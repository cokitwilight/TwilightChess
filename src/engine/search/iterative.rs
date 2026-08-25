use std::time::{Duration, Instant};

use crate::board::{Board, Move, MoveType};
use crate::engine::config::{CHECKMATE_SCORE, NEG_INF, POS_INF};
use crate::engine::history::HistoryKey;
use crate::engine::search_stats::{SearchStats, fmt_nps, median_f64};
use crate::engine::tt::entry::TTNodeType;
use crate::engine::tt::{TTEntry, TTFlag, score_to_tt};
use crate::engine::{Engine, SearchContext, SearchOptions, SearchResult};

// for debug printing
use thousands::Separable;

impl Engine {
    pub fn iterative_deepening(
        &mut self,
        board: &mut Board,
        ctx: &mut SearchContext,
        can_print: bool,
    ) -> SearchResult {
        let full_start = Instant::now();
        let mut best_result = SearchResult {
            best_move: None,
            eval: 0,
            depth_reached: 0,
            stats: SearchStats::default(),
            pv: Vec::new(),
            elapsed: full_start.elapsed(),
        };

        let mut total_time: f64 = 0.0;
        let mut total_nodes: u64 = 0;
        let mut nps_samples: Vec<f64> = Vec::new();

        let aspiration_start = self.config.search.aspiration.initial_window;
        let window_growth = self.config.search.aspiration.growth_factor;
        let aspiration_max = self.config.search.aspiration.max_window;
        let mate_margin = self.config.search.aspiration.mate_margin;

        // const _MAX_ASPIRATION_RESEARCHES: usize = 8; // use later if needed

        'depth_loop: for depth in 1..=ctx.limits.max_depth {
            if ctx.should_stop() {
                break;
            }

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

            if use_aspiration {
                ctx.stats.aspiration_stats.searches += 1;
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

            let mut aspiration_attempt = 0;

            let result = loop {
                let nodes_before_attempt = ctx.stats.total_nodes();
                let result =
                    self.search_root(board, ctx, best_result.best_move, depth, alpha, beta);
                let attempt_nodes = ctx.stats.total_nodes() - nodes_before_attempt;

                if use_aspiration && aspiration_attempt > 0 {
                    ctx.stats.aspiration_stats.research_nodes += attempt_nodes;
                }
                aspiration_attempt += 1;

                if ctx.should_stop() {
                    break 'depth_loop;
                }

                let result_is_mate_score = result.eval.abs() >= CHECKMATE_SCORE - mate_margin;

                // Exact score.
                if result.eval > alpha && result.eval < beta {
                    if use_aspiration && aspiration_attempt == 1 {
                        ctx.stats.aspiration_stats.successful_first_windows += 1;
                    }
                    break result;
                }

                // If we were already searching full-width and still failed,
                // something is wrong with score bounds, so accept to avoid infinite loop.
                if alpha == full_alpha && beta == full_beta {
                    break result;
                }

                if result.eval <= alpha {
                    ctx.stats.aspiration_stats.fail_low += 1;
                } else if result.eval >= beta {
                    ctx.stats.aspiration_stats.fail_high += 1;
                }

                // Mate scores can jump far outside the aspiration window.
                // Immediately fall back to full window instead of slowly widening.
                if result_is_mate_score {
                    ctx.stats.aspiration_stats.full_window_fallbacks += 1;
                    alpha = full_alpha;
                    beta = full_beta;
                    continue;
                }

                // Avoid infinite widening if something behaves unexpectedly.
                if window >= aspiration_max {
                    ctx.stats.aspiration_stats.full_window_fallbacks += 1;
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
            let depth_nodes = depth_stats.total_nodes();

            let depth_nps = if elapsed_secs > 0.0 {
                depth_nodes as f64 / elapsed_secs
            } else {
                0.0
            };

            total_time += elapsed_secs;
            total_nodes += depth_nodes;

            if depth_nps.is_finite() && depth_nps > 0.0 {
                nps_samples.push(depth_nps);
            }

            let avg_nps = if nps_samples.is_empty() {
                0.0
            } else {
                nps_samples.iter().sum::<f64>() / nps_samples.len() as f64
            };

            let median_nps = median_f64(&nps_samples);

            let weighted_avg_nps = if total_time > 0.0 {
                total_nodes as f64 / total_time
            } else {
                0.0
            };
            if can_print {
                depth_stats.print_all(depth, elapsed_secs);

                println!(
                    "Eval: {}. Time: {:.3}s. NPS: {} | Avg: {} | Median: {} | Weighted: {}",
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
            best_result.elapsed = full_start.elapsed()
        }

        total_time = full_start.elapsed().as_secs_f64();

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
        }

        best_result.stats = ctx.stats;
        best_result
    }

    fn search_root(
        &mut self,
        board: &mut Board,
        ctx: &mut SearchContext,
        previous_best_move: Option<Move>,
        depth: u16,
        mut alpha: i32,
        beta: i32,
    ) -> SearchResult {
        ctx.stats.node_stats.main += 1;
        ctx.stats.node_stats.pv += 1;
        ctx.stats.node_stats.root += 1;

        let original_alpha = alpha;
        let root_hash = board.hash();
        let side_to_move = board.side_to_move();
        if board.in_check(side_to_move) {
            ctx.stats.node_stats.in_check += 1;
        }

        let mut best_eval = NEG_INF;
        let mut best_move = None;
        let mut best_pv = Vec::new();

        let mut all_moves = board.all_legal_moves(); // this returns mostly legal moves except for pawn and king legality(TODO)

        let mut legal_moves = 0; // counter of legal moves since pseudo moves might not flag checkmate

        let mut stopped = false;

        let tt_best_move = self
            .tt
            .get(board.hash(), TTNodeType::Main)
            .and_then(|entry| entry.best_move)
            .or_else(|| {
                self.tt
                    .get_any(board.hash())
                    .and_then(|entry| entry.best_move)
            });

        // SELECTOR TESTING
        //

        // let expected = board.all_legal_moves();
        // let mut selector = self.new_staged_move_selecter(
        //     &board,
        //     &mut expected.clone(),
        //     board.side_to_move(),
        //     0,
        //     ctx,
        //     previous_best_move,
        //     tt_best_move,
        // );
        // let mut returned = Vec::new();

        // while let Some(scored) = selector.get_next(
        //     &board,
        //     board.side_to_move(),
        //     0,
        //     ctx,
        //     &self.history,
        //     previous_best_move,
        //     tt_best_move,
        // ) {
        //     returned.push(scored.mv);
        // }

        // assert_eq!(returned.len(), expected.len());

        // for mv in expected.iter() {
        //     assert_eq!(
        //         returned.iter().filter(|&&found| found == *mv).count(),
        //         1,
        //         "move must be returned exactly once: {mv:?}",
        //     );
        // }

        // SELECTOR TESTING

        self.order_moves(
            board,
            &mut all_moves,
            board.side_to_move(),
            0,
            ctx,
            previous_best_move,
            tt_best_move,
        );

        for mv in all_moves.iter() {
            if ctx.should_stop() {
                stopped = true;
                break;
            }

            let piece = board
                .piecetype_at(mv.from())
                .expect("No piece in board in Search Root!");

            let undo = board.make_move(*mv);

            let child_hash = board.hash();

            // if board.in_check(side_to_move) {
            //     board.undo_move(undo);
            //     continue;
            // }
            legal_moves += 1;

            ctx.repetition_history.push(child_hash);

            let eval = -self.negamax(
                board,
                ctx,
                depth - 1,
                -beta,
                -alpha,
                1,
                SearchOptions::NORMAL,
            );

            if stopped {
                break;
            }

            ctx.repetition_history.pop();

            board.undo_move(undo);

            if eval > best_eval {
                best_eval = eval;
                best_move = Some(*mv);

                best_pv.clear();
                best_pv.push(*mv);
            }

            if eval > alpha {
                alpha = eval;
            }

            if alpha >= beta {
                if (mv.kind() == MoveType::Normal || mv.kind() == MoveType::Castle)
                    && mv.promotion().is_none()
                {
                    self.history
                        .main
                        .add_bonus(HistoryKey::new(side_to_move, piece, mv.to()), depth);
                }
                break;
            }
        }
        if stopped {
            ctx.stats.terminal_stats.stopped_returns += 1;
            // return early before affecting logic with incomplete results
            return SearchResult {
                best_move,
                eval: best_eval,
                depth_reached: depth,
                stats: ctx.stats,
                pv: best_pv,
                elapsed: Duration::ZERO,
            };
        }

        if legal_moves == 0 {
            let eval = if board.in_check(side_to_move) {
                ctx.stats.terminal_stats.checkmates += 1;
                -CHECKMATE_SCORE
            } else {
                ctx.stats.terminal_stats.stalemates += 1;
                0
            };
            return SearchResult {
                best_move: None,
                eval,
                depth_reached: depth,
                stats: ctx.stats,
                pv: best_pv,
                elapsed: Duration::ZERO,
            };
        }

        let flag = if best_eval <= original_alpha {
            TTFlag::UpperBound
        } else if best_eval >= beta {
            TTFlag::LowerBound
        } else {
            TTFlag::Exact
        };
        ctx.stats.tt_stats.main.stores += 1;
        let insert_result = self.tt.insert(
            root_hash,
            TTEntry {
                eval: score_to_tt(best_eval, 1),
                depth: depth as u16,
                flag,
                best_move,
                node_type: TTNodeType::Main,
            },
        );
        ctx.stats.tt_stats.main.record_insert(insert_result);

        SearchResult {
            best_move,
            eval: best_eval,
            depth_reached: depth,
            stats: ctx.stats,
            pv: best_pv, // TODO: Implement principal variation
            elapsed: Duration::ZERO,
        }
    }
}
