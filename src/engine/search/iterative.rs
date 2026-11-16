use std::time::Duration;
use std::time::Instant;

use crate::board::Board;
use crate::engine::config::{CHECKMATE_SCORE, NEG_INF, POS_INF};
use crate::engine::search::nodetype::Root;
use crate::engine::search_stats::{SearchStats, fmt_nps, median_f64, track_stat};
use crate::engine::{
    Engine, MAX_PV, PickerFrame, SearchContext, SearchOptions, SearchResult, SearchTermination,
};
use crate::moves::MoveGenInfo;

// for debug printing
use thousands::Separable;

pub const MAX_QUIETS: usize = 45;
pub const MAX_CAPTURES: usize = 45;

impl Engine {
    pub(super) fn iterative_deepening(
        &mut self,
        board: &mut Board,
        ctx: &mut SearchContext,
        can_print: bool,
    ) -> SearchResult {
        let side_to_move = board.side_to_move();
        let info = MoveGenInfo::calculate(board, side_to_move);
        let fallback_move = {
            let mut move_picker =
                self.new_move_picker(0, ctx, side_to_move, 0, PickerFrame::ROOT, None, None);

            move_picker
                .get_next(board, &info, ctx, &self.history)
                .map(|scored_move| scored_move.mv)
        };

        let mut best_result = SearchResult {
            // A zero-duration/node limit can expire before depth one completes.
            // Keep a legal fallback so callers such as run_game can still move.
            best_move: fallback_move,
            eval: 0,
            depth_reached: 0,
            seldepth: 0,
            stats: SearchStats::default(),
            pv: [None; MAX_PV],
            elapsed: Duration::ZERO,
            termination: SearchTermination::DepthLimit,
            #[cfg(feature = "debug-uci")]
            iterations: Vec::new(),
        };

        let mut cumulative_stats = ctx.stats;

        let mut total_time: f64 = 0.0;

        let mut total_nodes: u64 = 0;

        let mut nps_samples = [0.0f64; 128];

        let mut nps_index = 0usize;

        let aspiration_start = self.config.search.aspiration.initial_window;
        let window_growth = self.config.search.aspiration.growth_factor;
        let aspiration_max = self.config.search.aspiration.max_window;
        let mate_margin = self.config.search.aspiration.mate_margin;

        // const _MAX_ASPIRATION_RESEARCHES: usize = 8; // use later if needed

        'depth_loop: for depth in 1..=ctx.limits.max_depth {
            if ctx.should_stop() {
                break;
            }

            ctx.seldepth = 0;

            let stats_before = ctx.stats;

            let start = Instant::now();

            let full_alpha = NEG_INF + 1;
            let full_beta = POS_INF - 1;

            debug_assert!(
                CHECKMATE_SCORE + mate_margin < full_beta,
                "POS_INF must be much larger than CHECKMATE_SCORE"
            );

            let previous_eval = best_result.eval;

            let previous_is_mate_score = previous_eval.abs() >= CHECKMATE_SCORE - mate_margin;

            let use_aspiration =
                depth > 1 && !previous_is_mate_score && self.config.search.aspiration.enabled;

            track_stat! {
                if use_aspiration {
                    ctx.stats.aspiration_stats.searches += 1;
                }
            }

            let mut window = aspiration_start;

            let mut alpha = if use_aspiration {
                previous_eval.saturating_sub(window).max(full_alpha)
            } else {
                full_alpha
            };

            let mut beta = if use_aspiration {
                previous_eval.saturating_add(window).min(full_beta)
            } else {
                full_beta
            };

            #[cfg(feature = "search-stats")]
            let mut aspiration_attempt = 0;

            let result = loop {
                #[cfg(feature = "search-stats")]
                let nodes_before_attempt = ctx.stats.total_nodes();

                ctx.previous_best_move = best_result.best_move;
                ctx.root_best_move = None;

                let eval = self.negamax::<Root>(
                    board,
                    ctx,
                    depth,
                    alpha,
                    beta,
                    0,
                    PickerFrame::ROOT,
                    SearchOptions::NORMAL,
                );

                let mut pv = [None; MAX_PV];
                pv[0] = ctx.root_best_move;

                let result = SearchResult {
                    best_move: ctx.root_best_move,
                    eval,
                    depth_reached: depth,
                    seldepth: ctx.seldepth,
                    stats: ctx.stats,
                    pv,
                    elapsed: ctx.elapsed(),
                    termination: SearchTermination::DepthLimit,
                    #[cfg(feature = "debug-uci")]
                    iterations: Vec::new(),
                };

                track_stat! {
                    let attempt_nodes = ctx.stats.total_nodes() - nodes_before_attempt;
                    if use_aspiration && aspiration_attempt > 0 {
                        ctx.stats.aspiration_stats.research_nodes += attempt_nodes;
                    }
                    aspiration_attempt += 1;
                }

                if ctx.should_stop() {
                    break 'depth_loop;
                }

                let result_is_mate_score = result.eval.abs() >= CHECKMATE_SCORE - mate_margin;

                // Exact score.
                if result.eval > alpha && result.eval < beta {
                    track_stat! {
                        if use_aspiration && aspiration_attempt == 1 {
                            ctx.stats.aspiration_stats.successful_first_windows += 1;
                        }
                    }
                    break result;
                }

                // If we were already searching full-width and still failed,
                // something is wrong with score bounds, so accept to avoid infinite loop.
                if alpha == full_alpha && beta == full_beta {
                    break result;
                }

                track_stat! {
                    if result.eval <= alpha {
                        ctx.stats.aspiration_stats.fail_low += 1;
                    } else if result.eval >= beta {
                        ctx.stats.aspiration_stats.fail_high += 1;
                    }
                }

                // Mate scores can jump far outside the aspiration window.
                // Immediately fall back to full window instead of slowly widening.
                if result_is_mate_score {
                    track_stat!(ctx.stats.aspiration_stats.full_window_fallbacks += 1);
                    alpha = full_alpha;
                    beta = full_beta;
                    continue;
                }

                // Avoid infinite widening if something behaves unexpectedly.
                if window >= aspiration_max {
                    track_stat!(ctx.stats.aspiration_stats.full_window_fallbacks += 1);
                    alpha = full_alpha;
                    beta = full_beta;
                    continue;
                }
                window = window.saturating_mul(window_growth).min(aspiration_max);

                if result.eval <= alpha {
                    // Widen downward.
                    alpha = result.eval.saturating_sub(window).max(full_alpha);

                    // Optional: keep beta near previous expectation unless window is maxed.
                    if window >= aspiration_max {
                        beta = full_beta;
                    }

                    continue;
                }

                if result.eval >= beta {
                    // Widen upward.
                    beta = result.eval.saturating_add(window).min(full_beta);

                    // Optional: keep alpha near previous expectation unless window is maxed.
                    if window >= aspiration_max {
                        alpha = full_alpha;
                    }

                    continue;
                }

                unreachable!("aspiration result was neither exact nor fail-high/low");
            };

            let elapsed = start.elapsed();

            let elapsed_secs = elapsed.as_secs_f64();
            let depth_stats = ctx.stats - stats_before;

            cumulative_stats += depth_stats;

            let depth_nodes = depth_stats.total_nodes();

            cumulative_stats
                .iterative_deepening_stats
                .record_iteration(depth, depth_nodes);

            ctx.stats
                .iterative_deepening_stats
                .record_iteration(depth, depth_nodes);

            let depth_nps = if elapsed_secs > 0.0 {
                depth_nodes as f64 / elapsed_secs
            } else {
                0.0
            };

            total_time += elapsed_secs;
            total_nodes += depth_nodes;

            if depth_nps.is_finite() && depth_nps > 0.0 && nps_index < 128 {
                nps_samples[nps_index] = depth_nps;
                nps_index += 1;
            }

            let avg_nps = if nps_index == 0 {
                0.0
            } else {
                nps_samples.iter().sum::<f64>() / (nps_index + 1) as f64
            };

            let median_nps = median_f64(&nps_samples);

            let weighted_avg_nps = if total_time > 0.0 {
                total_nodes as f64 / total_time
            } else {
                0.0
            };
            if can_print {
                let estimated_node_growth = ctx
                    .stats
                    .iterative_deepening_stats
                    .estimated_growth_factor(depth)
                    .map_or_else(|| "n/a".to_string(), |factor| format!("{factor:.2}x"));

                depth_stats.print_all(depth, elapsed_secs);

                println!(
                    "Iteration Nodes: {}. Estimated Increase: {}",
                    depth_nodes.separate_with_commas(),
                    estimated_node_growth,
                );

                println!(
                    "Depth: {}. Eval: {}. Time: {:.3}s. NPS: {} | Avg: {} | Median: {} | Weighted: {}",
                    depth,
                    result.eval,
                    elapsed_secs,
                    fmt_nps(depth_nps),
                    fmt_nps(avg_nps),
                    fmt_nps(median_nps),
                    fmt_nps(weighted_avg_nps),
                );
            }

            if ctx.should_stop() {
                // dont corrupt the result with a potentially half searched result
                break;
            }

            best_result = result;
            best_result.depth_reached = depth;
            best_result.stats = ctx.stats;
            best_result.elapsed = ctx.elapsed();
            #[cfg(feature = "debug-uci")]
            if ctx.record_iteration(&best_result) {
                ctx.stop_reason = Some(SearchTermination::IterationStop);
                break;
            }
        }

        let total_time = ctx.elapsed().as_secs_f64();

        let total_nps = if total_time > 0.0 {
            ctx.stats.total_nodes() as f64 / total_time
        } else {
            0.0
        };

        if can_print {
            println!(
                "\nFinal Eval: {}. Total Time: {:.3}. Total NPS: {}\n",
                best_result.eval,
                total_time,
                format!("{:.2}", total_nps).separate_with_commas()
            );
            track_stat! {
                println!();
                println!("Cumulative Stats:");
                cumulative_stats.print_all(best_result.depth_reached, total_time);
            }
        }

        best_result.stats = ctx.stats;
        best_result.elapsed = ctx.elapsed();
        best_result.termination = ctx.stop_reason.unwrap_or(SearchTermination::DepthLimit);
        #[cfg(feature = "debug-uci")]
        {
            best_result.iterations = std::mem::take(&mut ctx.iterations);
        }
        best_result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::MoveType;
    use crate::engine::SearchLimits;
    use crate::engine::configs::EngineConfig;
    use crate::engine::tt::{TTFlag, TTNodeType, score_from_tt};

    #[test]
    fn node_limits_work_with_or_without_statistics() {
        let config = EngineConfig {
            tt_size: 1,
            ..EngineConfig::default()
        };
        let board = Board::from_fen(crate::board::STARTPOS_FEN).unwrap();
        for max_nodes in [0, 2048] {
            let mut limits = SearchLimits::depth(20, 3);
            limits.max_nodes = Some(max_nodes);
            let mut context = SearchContext::new(limits, vec![board.board_hash()]);
            let result = Engine::new(config).search(&board, &mut context, false, false);

            assert_eq!(result.termination, SearchTermination::NodeLimit);
            assert!(result.best_move.is_some());
            assert!(result.stats.total_nodes() >= max_nodes);
            assert!(result.stats.total_nodes() < max_nodes + 2048);
            if max_nodes == 0 {
                assert_eq!(result.stats.total_nodes(), 0);
                assert_eq!(result.depth_reached, 0);
            }
        }
    }

    #[cfg(feature = "debug-uci")]
    #[test]
    fn completed_iteration_history_preserves_each_depth() {
        let config = EngineConfig {
            tt_size: 1,
            ..EngineConfig::default()
        };
        let board = Board::from_fen(crate::board::STARTPOS_FEN).unwrap();
        let result = Engine::new(config).search(
            &board,
            &mut SearchContext::new(SearchLimits::depth(3, 2), vec![board.board_hash()]),
            false,
            false,
        );

        assert_eq!(result.iterations.len(), 3);
        for (index, iteration) in result.iterations.iter().enumerate() {
            assert_eq!(iteration.depth_reached as usize, index + 1);
            assert!(iteration.best_move.is_some());
            assert!(iteration.main_nodes > 0);
            assert!(iteration.quiescence_nodes > 0);
            assert!(iteration.elapsed <= result.elapsed);
        }
        for pair in result.iterations.windows(2) {
            assert!(pair[0].main_nodes < pair[1].main_nodes);
            assert!(pair[0].quiescence_nodes <= pair[1].quiescence_nodes);
            assert!(pair[0].elapsed <= pair[1].elapsed);
        }
        let last = result.iterations.last().unwrap();
        assert_eq!(last.best_move, result.best_move);
        assert_eq!(last.eval, result.eval);
        assert_eq!(last.seldepth, result.seldepth);
        assert_eq!(last.pv, result.pv);
        assert_eq!(last.main_nodes, result.stats.node_stats.main);
        assert_eq!(last.quiescence_nodes, result.stats.node_stats.quiescence);
    }

    #[cfg(feature = "debug-uci")]
    #[test]
    fn interrupted_depth_is_excluded_from_iteration_history() {
        let config = EngineConfig {
            tt_size: 1,
            ..EngineConfig::default()
        };
        let board = Board::from_fen(crate::board::STARTPOS_FEN).unwrap();
        let history = vec![board.board_hash()];
        let completed = Engine::new(config).search(
            &board,
            &mut SearchContext::new(SearchLimits::depth(1, 2), history.clone()),
            false,
            false,
        );
        let mut limits = SearchLimits::depth(3, 2);
        limits.max_nodes = Some(completed.stats.total_nodes() + 1);
        let result = Engine::new(config).search(
            &board,
            &mut SearchContext::new(limits, history),
            false,
            false,
        );

        assert_eq!(result.termination, SearchTermination::NodeLimit);
        assert_eq!(result.iterations.len(), 1);
        let last = &result.iterations[0];
        assert_eq!(last.depth_reached, 1);
        assert_eq!(last.best_move, completed.best_move);
        assert_eq!(last.eval, completed.eval);
        assert_eq!(
            last.main_nodes + last.quiescence_nodes,
            completed.stats.total_nodes()
        );
        assert!(result.stats.total_nodes() > last.main_nodes + last.quiescence_nodes);
    }

    #[test]
    fn root_stores_large_depths_without_wrapping() {
        for depth in [255, 256, 257, u16::MAX] {
            let mut config = EngineConfig::standard();
            config.tt_size = 1;
            let mut engine = Engine::new(config);
            // Black's only move captures the queen, leaving insufficient
            // material. Even a huge requested depth finishes immediately.
            let mut board = Board::from_fen("7k/6Q1/4K3/8/8/8/8/8 b - - 0 1").unwrap();
            let mut context =
                SearchContext::new(SearchLimits::depth(depth, 1), vec![board.board_hash()]);
            let eval = engine.negamax::<Root>(
                &mut board,
                &mut context,
                depth,
                NEG_INF + 1,
                POS_INF - 1,
                0,
                PickerFrame::ROOT,
                SearchOptions::NORMAL,
            );
            assert!(context.root_best_move.is_some());
            assert_eq!(eval, 0);
            assert_eq!(
                engine
                    .tt
                    .get(board.board_hash(), TTNodeType::Main)
                    .unwrap()
                    .depth,
                255
            );
        }
    }

    #[test]
    fn staged_root_detects_checkmate_and_stalemate() {
        let cases = [
            ("7k/6Q1/6K1/8/8/8/8/8 b - - 0 1", -CHECKMATE_SCORE),
            ("7k/5Q2/6K1/8/8/8/8/8 b - - 0 1", 0),
        ];

        for (fen, expected) in cases {
            let mut config = EngineConfig::standard();
            config.tt_size = 1;
            let mut engine = Engine::new(config);
            let mut board = Board::from_fen(fen).expect("valid terminal FEN");
            let mut context =
                SearchContext::new(SearchLimits::depth(1, 1), vec![board.board_hash()]);

            let eval = engine.negamax::<Root>(
                &mut board,
                &mut context,
                1,
                NEG_INF + 1,
                POS_INF - 1,
                0,
                PickerFrame::ROOT,
                SearchOptions::NORMAL,
            );

            assert_eq!(eval, expected, "wrong terminal score for {fen}");
            assert_eq!(context.root_best_move, None);
            assert_eq!(context.stats.node_stats.main, 1);
            assert_eq!(
                context.stats.node_stats.root,
                u64::from(cfg!(feature = "search-stats"))
            );
            assert_eq!(
                context.stats.terminal_stats.checkmates + context.stats.terminal_stats.stalemates,
                u64::from(cfg!(feature = "search-stats"))
            );
        }
    }

    #[test]
    fn root_search_ignores_tt_score_returns_and_window_narrowing() {
        for (flag, cached_score) in [
            (TTFlag::Exact, 123),
            (TTFlag::LowerBound, 123),
            (TTFlag::UpperBound, -123),
        ] {
            let config = EngineConfig {
                tt_size: 1,
                ..EngineConfig::standard()
            };
            let mut engine = Engine::new(config);
            // The only legal move captures the queen and draws.
            let mut board = Board::from_fen("7k/6Q1/4K3/8/8/8/8/8 b - - 0 1").unwrap();
            let root_hash = board.board_hash();
            engine
                .tt
                .insert(root_hash, true, flag, 6, cached_score, i16::MIN, None);
            let mut context = SearchContext::new(SearchLimits::depth(1, 1), vec![root_hash]);

            let eval = engine.negamax::<Root>(
                &mut board,
                &mut context,
                1,
                -1,
                1,
                0,
                PickerFrame::ROOT,
                SearchOptions::NORMAL,
            );

            assert_eq!(eval, 0, "root accepted the cached {flag:?} score");
            assert!(context.root_best_move.is_some());
            assert!(context.stats.total_nodes() > 1);
            assert_eq!(context.repetition_history, vec![root_hash]);
            assert_eq!(board.board_hash(), root_hash);
            board.assert_hash();
        }
    }

    #[test]
    fn root_search_selects_a_move_despite_root_draw_conditions() {
        for (fen, repeated) in [
            ("7k/6Q1/4K3/8/8/8/8/8 b - - 4 1", true),
            ("7k/6Q1/4K3/8/8/8/8/8 b - - 100 1", false),
            ("7k/8/8/8/8/8/8/K7 w - - 0 1", false),
        ] {
            let config = EngineConfig {
                tt_size: 1,
                ..EngineConfig::standard()
            };
            let mut engine = Engine::new(config);
            let mut board = Board::from_fen(fen).unwrap();
            let root_hash = board.board_hash();
            let history = vec![root_hash; if repeated { 3 } else { 1 }];
            let mut context = SearchContext::new(SearchLimits::depth(1, 1), history.clone());
            let legal_moves = board.all_legal_moves();

            let eval = engine.negamax::<Root>(
                &mut board,
                &mut context,
                1,
                NEG_INF + 1,
                POS_INF - 1,
                0,
                PickerFrame::ROOT,
                SearchOptions::NORMAL,
            );

            assert_eq!(eval, 0, "wrong draw score for {fen}");
            let best_move = context.root_best_move.expect("root must search a move");
            assert!(legal_moves.iter().any(|&mv| mv == best_move));
            assert_eq!(context.repetition_history, history);
            assert_eq!(board.board_hash(), root_hash);
            board.assert_hash();
        }
    }

    #[test]
    fn root_fail_low_keeps_the_preferred_best_move() {
        let config = EngineConfig {
            tt_size: 1,
            ..EngineConfig::standard()
        };
        let mut engine = Engine::new(config);
        // All children draw, so the first searched move wins the tie.
        let mut board = Board::from_fen("7k/8/8/8/8/8/8/K7 w - - 0 1").unwrap();
        let preferred = *board.all_legal_moves().as_slice().last().unwrap();
        let mut context = SearchContext::new(SearchLimits::depth(1, 1), vec![board.board_hash()]);
        context.previous_best_move = Some(preferred);

        let eval = engine.negamax::<Root>(
            &mut board,
            &mut context,
            1,
            1,
            2,
            0,
            PickerFrame::ROOT,
            SearchOptions::NORMAL,
        );

        assert_eq!(eval, 0);
        assert_eq!(context.root_best_move, Some(preferred));
        let entry = engine.tt.get(board.board_hash(), TTNodeType::Main).unwrap();
        assert_eq!(entry.flag.bound_type(), TTFlag::UpperBound);
        assert_eq!(entry.best_move, Some(preferred));
    }

    #[test]
    fn root_stores_mate_scores_at_ply_zero() {
        let config = EngineConfig {
            tt_size: 1,
            ..EngineConfig::standard()
        };
        let mut engine = Engine::new(config);
        let mut board = Board::from_fen("7k/5Q2/6K1/8/8/8/8/8 w - - 0 1").unwrap();
        let mut context = SearchContext::new(SearchLimits::depth(1, 1), vec![board.board_hash()]);

        let eval = engine.negamax::<Root>(
            &mut board,
            &mut context,
            1,
            NEG_INF + 1,
            POS_INF - 1,
            0,
            PickerFrame::ROOT,
            SearchOptions::NORMAL,
        );

        assert_eq!(eval, CHECKMATE_SCORE - 1);
        let entry = engine.tt.get(board.board_hash(), TTNodeType::Main).unwrap();
        assert_eq!(score_from_tt(i32::from(entry.eval), 0), eval);
        assert_eq!(entry.best_move, context.root_best_move);
    }

    #[test]
    fn root_search_handles_an_en_passant_fallback_move() {
        let config = EngineConfig {
            tt_size: 1,
            ..EngineConfig::standard()
        };
        let mut engine = Engine::new(config);
        let mut board = Board::from_fen("7k/8/8/3pP3/8/8/8/K7 w - d6 0 1").unwrap();
        let root_hash = board.board_hash();
        let legal_moves = board.all_legal_moves();
        let mut context = SearchContext::new(SearchLimits::depth(1, 1), vec![root_hash]);

        let result = engine.search(&board, &mut context, false, false);

        assert_eq!(
            context.previous_best_move.unwrap().kind(),
            MoveType::EnPassant
        );
        assert_eq!(result.depth_reached, 1);
        let best_move = result
            .best_move
            .expect("completed root search must select a move");
        assert!(legal_moves.iter().any(|&mv| mv == best_move));
        assert_eq!(result.pv[0], Some(best_move));
        assert_eq!(result.best_move, context.root_best_move);
        assert_eq!(context.repetition_history, vec![root_hash]);
    }
}
