use std::time::{Duration, Instant};

use crate::board::{Board, Move, MoveType};
use crate::engine::config::{CHECKMATE_SCORE, NEG_INF, POS_INF};
use crate::engine::debug::negamax::negamax;
use crate::engine::history::HistoryKey;
use crate::engine::search::iterative::{MAX_CAPTURES, MAX_QUIETS};
use crate::engine::search_stats::{SearchStats, fmt_nps, median_f64};
use crate::engine::staged::ScoredMove;
use crate::engine::tt::{TTEntry, TTFlag, TTNodeType, score_to_tt};
use crate::engine::{
    Engine, MAX_PV, PickerFrame, SearchContext, SearchLimits, SearchOptions, SearchResult,
    SearchTermination,
};
use crate::moves::MoveGenInfo;
use crate::types::PieceType;
use thousands::Separable;

impl Engine {
    /// Drop-in search entry point for benchmarking without detailed search
    /// instrumentation. Only main and quiescence node counts are populated.
    pub fn search_without_stats(
        &mut self,
        board: &Board,
        limits: SearchLimits,
        repetition_history: &Vec<u64>,
        opening_allowed: bool,
        can_print: bool,
    ) -> SearchResult {
        search_without_stats(
            self,
            board,
            limits,
            repetition_history,
            opening_allowed,
            can_print,
        )
    }
}

/// Runs the normal engine search without collecting instrumentation other than
/// main-search and quiescence node counts.
pub fn search_without_stats(
    engine: &mut Engine,
    board: &Board,
    limits: SearchLimits,
    repetition_history: &Vec<u64>,
    opening_allowed: bool,
    can_print: bool,
) -> SearchResult {
    let mut context = SearchContext::new(limits, repetition_history.clone());
    let mut board = board.clone();

    if opening_allowed && let Some(book_mv) = engine.get_book_move(&board) {
        if can_print {
            println!("Book Move. {}", book_mv);
        }

        return SearchResult {
            best_move: Some(book_mv),
            eval: 0,
            depth_reached: 0,
            stats: context.stats,
            pv: [None; MAX_PV],
            elapsed: context.elapsed(),
            termination: SearchTermination::BookMove,
        };
    }

    engine.tt.new_search();
    iterative_deepening(engine, &mut board, &mut context, can_print)
}

fn iterative_deepening(
    engine: &mut Engine,
    board: &mut Board,
    ctx: &mut SearchContext,
    can_print: bool,
) -> SearchResult {
    let side_to_move = board.side_to_move();
    let info = MoveGenInfo::calculate(board, side_to_move);
    let fallback_move = {
        let mut move_picker =
            engine.new_move_picker(0, ctx, side_to_move, 0, PickerFrame::ROOT, None, None);

        move_picker
            .get_next(board, &info, ctx, &engine.history)
            .map(|scored_move| scored_move.mv)
    };

    let mut best_result = SearchResult {
        best_move: fallback_move,
        eval: 0,
        depth_reached: 0,
        stats: SearchStats::default(),
        pv: [None; MAX_PV],
        elapsed: Duration::ZERO,
        termination: SearchTermination::DepthLimit,
    };

    let mut total_time: f64 = 0.0;
    let mut total_nodes: u64 = 0;
    let mut nps_samples = [0.0f64; 128];
    let mut nps_index = 0usize;

    let aspiration_start = engine.config.search.aspiration.initial_window;
    let window_growth = engine.config.search.aspiration.growth_factor;
    let aspiration_max = engine.config.search.aspiration.max_window;
    let mate_margin = engine.config.search.aspiration.mate_margin;

    'depth_loop: for depth in 1..=ctx.limits.max_depth {
        if ctx.should_stop() {
            break;
        }

        let nodes_before_depth = ctx.stats.total_nodes();
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
            depth > 1 && !previous_is_mate_score && engine.config.search.aspiration.enabled;

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

        let result = loop {
            let result = search_root(
                engine,
                board,
                ctx,
                best_result.best_move,
                depth,
                alpha,
                beta,
            );
            if ctx.should_stop() {
                break 'depth_loop;
            }

            let result_is_mate_score = result.eval.abs() >= CHECKMATE_SCORE - mate_margin;

            if result.eval > alpha && result.eval < beta {
                break result;
            }

            if alpha == full_alpha && beta == full_beta {
                break result;
            }

            if result_is_mate_score {
                alpha = full_alpha;
                beta = full_beta;
                continue;
            }

            if window >= aspiration_max {
                alpha = full_alpha;
                beta = full_beta;
                continue;
            }
            window = window.saturating_mul(window_growth).min(aspiration_max);

            if result.eval <= alpha {
                alpha = result.eval.saturating_sub(window).max(full_alpha);

                if window >= aspiration_max {
                    beta = full_beta;
                }

                continue;
            }

            if result.eval >= beta {
                beta = result.eval.saturating_add(window).min(full_beta);

                if window >= aspiration_max {
                    alpha = full_alpha;
                }

                continue;
            }

            unreachable!("aspiration result was neither exact nor fail-high/low");
        };

        let elapsed = start.elapsed();
        let elapsed_secs = elapsed.as_secs_f64();
        let depth_nodes = ctx.stats.total_nodes() - nodes_before_depth;
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
            println!(
                "Depth: {}. Nodes: {}. Eval: {}. Time: {:.3}s. NPS: {} | Avg: {} | Median: {} | Weighted: {}",
                depth,
                depth_nodes,
                result.eval,
                elapsed_secs,
                fmt_nps(depth_nps),
                fmt_nps(avg_nps),
                fmt_nps(median_nps),
                fmt_nps(weighted_avg_nps),
            );
        }

        if ctx.should_stop() {
            break;
        }

        best_result = result;
        best_result.depth_reached = depth;
    }

    total_time = ctx.elapsed().as_secs_f64();

    let total_nps = if total_time > 0.0 {
        ctx.stats.total_nodes() as f64 / total_time
    } else {
        0.0
    };

    if can_print {
        println!(
            "\nFinal Eval: {}. Total Time: {:.3}. Total Nodes: {}. Total NPS: {}\n",
            best_result.eval,
            total_time,
            ctx.stats.total_nodes(),
            format!("{:.2}", total_nps).separate_with_commas()
        );
    }

    best_result.stats = ctx.stats;
    best_result.elapsed = ctx.elapsed();
    best_result.termination = ctx.stop_reason.unwrap_or(SearchTermination::DepthLimit);
    best_result
}

fn search_root(
    engine: &mut Engine,
    board: &mut Board,
    ctx: &mut SearchContext,
    previous_best_move: Option<Move>,
    depth: u16,
    mut alpha: i32,
    beta: i32,
) -> SearchResult {
    ctx.stats.node_stats.main += 1;

    let original_alpha = alpha;
    let root_hash = board.hash();
    let side_to_move = board.side_to_move();

    let mut best_eval = NEG_INF;
    let mut best_move = None;
    let mut best_pv = [None; MAX_PV];
    let mut legal_moves = 0;
    let mut stopped = false;

    let tt_best_move = engine
        .tt
        .get(board.hash(), TTNodeType::Main)
        .and_then(|entry| entry.best_move)
        .or_else(|| {
            engine
                .tt
                .get_any(board.hash())
                .and_then(|entry| entry.best_move)
        });

    let mut move_picker = engine.new_move_picker(
        0,
        ctx,
        side_to_move,
        0,
        PickerFrame::ROOT,
        previous_best_move,
        tt_best_move,
    );

    let mut searched_quiets = [ScoredMove::new(); MAX_QUIETS];
    let mut q = 0usize;
    let mut searched_captures = [ScoredMove::new(); MAX_CAPTURES];
    let mut c = 0usize;

    let info = MoveGenInfo::calculate(board, side_to_move);

    while let Some(scored_mv) = move_picker.get_next(board, &info, ctx, &engine.history) {
        if ctx.should_stop() {
            stopped = true;
            break;
        }

        let mv = &scored_mv.mv;
        let is_capture = mv.kind() == MoveType::Capture || mv.kind() == MoveType::EnPassant;
        let is_quiet = !is_capture && mv.promotion().is_none();
        let mut improved_alpha = false;

        let piece = board
            .piecetype_at(mv.from())
            .expect("No piece in board in Search Root!");

        let undo = board.make_move(*mv);
        let child_hash = board.hash();
        legal_moves += 1;
        ctx.repetition_history.push(child_hash);

        let eval = -negamax(
            engine,
            board,
            ctx,
            depth - 1,
            -beta,
            -alpha,
            1,
            PickerFrame::ROOT.child(),
            SearchOptions::NORMAL,
        );

        ctx.repetition_history.pop();
        board.undo_move(undo);

        if ctx.stopped {
            stopped = true;
            break;
        }

        if eval > best_eval {
            best_eval = eval;
            best_move = Some(*mv);
            best_pv[0] = Some(*mv);
        }

        if eval > alpha {
            alpha = eval;
            improved_alpha = true;
        }

        if alpha >= beta {
            let history_key = if let Some(key) = scored_mv.history_key {
                key
            } else {
                HistoryKey::new(side_to_move, piece, mv.to())
            };

            if is_quiet {
                engine.history.main.add_bonus(history_key, depth);

                for scored_mv in &searched_quiets[..q] {
                    let key = if let Some(k) = scored_mv.history_key {
                        k
                    } else {
                        let mv = &scored_mv.mv;
                        let piece = board
                            .piecetype_at(mv.from())
                            .expect("No piece in board in negamax!");
                        HistoryKey::new(side_to_move, piece, mv.to())
                    };

                    engine.history.main.add_malus(key, depth);
                }
            } else if is_capture {
                let captured_piece = if let Some(piece) = scored_mv.captured {
                    piece
                } else {
                    match mv.kind() {
                        MoveType::EnPassant => PieceType::Pawn,
                        MoveType::Capture => board.piecetype_at(mv.to()).unwrap(),
                        _ => unreachable!(
                            "Somehow non capture move in searched captures in negamax! {:?}",
                            mv.kind()
                        ),
                    }
                };

                engine
                    .history
                    .add_capture_bonus(history_key, captured_piece, depth);
            }

            for scored_mv in &searched_captures[..c] {
                let mv = &scored_mv.mv;
                let history_key = if let Some(key) = scored_mv.history_key {
                    key
                } else {
                    let piece = board
                        .piecetype_at(mv.from())
                        .expect("No piece in board in negamax!");
                    HistoryKey::new(side_to_move, piece, mv.to())
                };

                let captured_piece = if let Some(piece) = scored_mv.captured {
                    piece
                } else {
                    match mv.kind() {
                        MoveType::EnPassant => PieceType::Pawn,
                        MoveType::Capture => board.piecetype_at(mv.to()).unwrap(),
                        _ => unreachable!(
                            "Somehow non capture move in searched captures in negamax! {:?}",
                            mv.kind()
                        ),
                    }
                };

                engine
                    .history
                    .add_capture_malus(history_key, captured_piece, depth);
            }
            break;
        }

        if !improved_alpha {
            if is_quiet && q < MAX_QUIETS {
                searched_quiets[q] = scored_mv;
                q += 1;
            } else if is_capture && c < MAX_CAPTURES {
                searched_captures[c] = scored_mv;
                c += 1;
            }
        }
    }

    if stopped {
        return SearchResult {
            best_move,
            eval: best_eval,
            depth_reached: depth,
            stats: ctx.stats,
            pv: best_pv,
            elapsed: Duration::ZERO,
            termination: SearchTermination::DepthLimit,
        };
    }

    if legal_moves == 0 {
        let eval = if board.in_check(side_to_move) {
            -CHECKMATE_SCORE
        } else {
            0
        };
        return SearchResult {
            best_move: None,
            eval,
            depth_reached: depth,
            stats: ctx.stats,
            pv: best_pv,
            elapsed: Duration::ZERO,
            termination: SearchTermination::DepthLimit,
        };
    }

    let flag = if best_eval <= original_alpha {
        TTFlag::UpperBound
    } else if best_eval >= beta {
        TTFlag::LowerBound
    } else {
        TTFlag::Exact
    };
    engine.tt.insert(
        root_hash,
        TTEntry {
            eval: score_to_tt(best_eval, 1),
            depth,
            flag,
            best_move,
            node_type: TTNodeType::Main,
        },
    );

    SearchResult {
        best_move,
        eval: best_eval,
        depth_reached: depth,
        stats: ctx.stats,
        pv: best_pv,
        elapsed: Duration::ZERO,
        termination: SearchTermination::DepthLimit,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::STARTPOS_FEN;
    use crate::engine::configs::EngineConfig;

    #[test]
    fn search_without_stats_only_populates_node_counts() {
        let mut config = EngineConfig::standard();
        config.tt_size = 1;
        let mut engine = Engine::new(config);
        let board = Board::from_fen(STARTPOS_FEN).expect("valid start position");

        let result = search_without_stats(
            &mut engine,
            &board,
            SearchLimits::depth(2, 1),
            &vec![board.hash()],
            false,
            false,
        );

        let nodes = result.stats.node_stats;
        assert!(nodes.main > 0);
        assert!(nodes.quiescence > 0);
        assert_eq!(nodes.pv, 0);
        assert_eq!(nodes.non_pv, 0);
        assert_eq!(nodes.in_check, 0);
        assert_eq!(nodes.root, 0);

        let expected = SearchStats {
            node_stats: nodes,
            ..SearchStats::default()
        };
        assert_eq!(format!("{:?}", result.stats), format!("{:?}", expected));
    }

    #[test]
    fn search_without_stats_matches_normal_search_logic() {
        let mut config = EngineConfig::default();
        config.tt_size = 1;
        config.search.lmp.enabled = true;
        let board = Board::from_fen(STARTPOS_FEN).expect("valid start position");
        let limits = SearchLimits::depth(3, 3);
        let repetition_history = vec![board.hash()];
        let mut normal_engine = Engine::new(config);
        let mut uninstrumented_engine = Engine::new(config);

        let normal = normal_engine.search(&board, limits, &repetition_history, false, false);
        let uninstrumented = uninstrumented_engine.search_without_stats(
            &board,
            limits,
            &repetition_history,
            false,
            false,
        );

        assert_eq!(uninstrumented.best_move, normal.best_move);
        assert_eq!(uninstrumented.eval, normal.eval);
        assert_eq!(uninstrumented.depth_reached, normal.depth_reached);
        assert_eq!(uninstrumented.termination, normal.termination);
        assert_eq!(
            uninstrumented.stats.node_stats.main,
            normal.stats.node_stats.main
        );
        assert_eq!(
            uninstrumented.stats.node_stats.quiescence,
            normal.stats.node_stats.quiescence
        );
    }
}
