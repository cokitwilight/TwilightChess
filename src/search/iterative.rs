use std::time::Instant;

use crate::bitboard::print_all_bitboards;
use crate::board::{Board, Move, MoveType};
use crate::search::engine::{Engine, SearchContext, SearchResult};
use crate::search::negamax::{CHECKMATE_SCORE, NEG_INF, POS_INF};
use crate::search::stats::{SearchStats, fmt_nps, median_f64};
use crate::search::tt::{TTEntry, TTFlag};

// for debug printing
use thousands::Separable;

impl Engine {
    pub fn iterative_deepening(
        &mut self,
        board: &mut Board,
        ctx: &mut SearchContext,
    ) -> SearchResult {
        let mut best_result = SearchResult {
            best_move: None,
            eval: 0,
            depth_reached: 0,
            stats: SearchStats::default(),
            pv: Vec::new(),
        };

        let mut total_time: f64 = 0.0;
        let mut total_nodes: u64 = 0;
        let mut nps_samples: Vec<f64> = Vec::new();

        const ASPIRATION_START: i32 = 25;
        const ASPIRATION_MAX: i32 = 800;
        const MATE_MARGIN: i32 = 1000;
        const MAX_ASPIRATION_RESEARCHES: usize = 8;

        'depth_loop: for depth in 1..=ctx.limits.max_depth {
            if ctx.should_stop() {
                break;
            }

            let stats_before = ctx.stats;
            let start = Instant::now();

            let full_alpha = NEG_INF + 1;
            let full_beta = POS_INF - 1;

            debug_assert!(
                CHECKMATE_SCORE + MATE_MARGIN < full_beta,
                "POS_INF must be much larger than CHECKMATE_SCORE"
            );

            let previous_eval = best_result.eval;

            let previous_is_mate_score = previous_eval.abs() >= CHECKMATE_SCORE - MATE_MARGIN;

            let use_aspiration = depth > 1 && !previous_is_mate_score;

            let mut window = ASPIRATION_START;

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
                let result =
                    self.search_root(board, ctx, best_result.best_move, depth, alpha, beta);

                if ctx.should_stop() {
                    break 'depth_loop;
                }

                let result_is_mate_score = result.eval.abs() >= CHECKMATE_SCORE - MATE_MARGIN;

                // Exact score.
                if result.eval > alpha && result.eval < beta {
                    break result;
                }

                // If we were already searching full-width and still failed,
                // something is wrong with score bounds, so accept to avoid infinite loop.
                if alpha == full_alpha && beta == full_beta {
                    break result;
                }

                // Mate scores can jump far outside the aspiration window.
                // Immediately fall back to full window instead of slowly widening.
                if result_is_mate_score {
                    alpha = full_alpha;
                    beta = full_beta;
                    continue;
                }

                // Avoid infinite widening if something behaves unexpectedly.
                if window >= ASPIRATION_MAX {
                    alpha = full_alpha;
                    beta = full_beta;
                    continue;
                }

                window = window.saturating_mul(2).min(ASPIRATION_MAX);

                if result.eval <= alpha {
                    ctx.stats.aspiration_w_fail_low += 1;

                    // Widen downward.
                    alpha = result.eval.saturating_sub(window).max(full_alpha);

                    // Optional: keep beta near previous expectation unless window is maxed.
                    if window >= ASPIRATION_MAX {
                        beta = full_beta;
                    }

                    continue;
                }

                if result.eval >= beta {
                    ctx.stats.aspiration_w_fail_high += 1;

                    // Widen upward.
                    beta = result.eval.saturating_add(window).min(full_beta);

                    // Optional: keep alpha near previous expectation unless window is maxed.
                    if window >= ASPIRATION_MAX {
                        alpha = full_alpha;
                    }

                    continue;
                }

                unreachable!("aspiration result was neither exact nor fail-high/low");
            };

            let elapsed = start.elapsed();

            let elapsed_secs = elapsed.as_secs_f64();
            let depth_stats = ctx.stats - stats_before;
            let depth_nodes = depth_stats.nodes + depth_stats.qnodes;

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

            if ctx.should_stop() {
                break;
            }

            best_result = result;
            best_result.depth_reached = depth;
        }

        let total_nps = if total_time > 0.0 {
            (ctx.stats.nodes + ctx.stats.qnodes) as f64 / total_time
        } else {
            0.0
        };

        println!(
            "\nFinal Eval: {}. Total Time: {:.3}. Total NPS: {}\n",
            best_result.eval,
            total_time,
            format!("{:.2}", total_nps).separate_with_commas()
        );

        best_result.stats = ctx.stats;
        best_result
    }

    fn search_root(
        &mut self,
        board: &mut Board,
        ctx: &mut SearchContext,
        previous_best_move: Option<Move>,
        depth: usize,
        mut alpha: i32,
        beta: i32,
    ) -> SearchResult {
        ctx.stats.nodes += 1;
        if Engine::repetition_in_search(ctx, board.hash(), board.halfmove_clock() as usize)
            || board.halfmove_clock() >= 100
        {}
        if Engine::repetition_in_search(ctx, board.hash(), board.halfmove_clock() as usize) {
            ctx.stats.repetition_returns += 1;
            return SearchResult {
                best_move: None,
                eval: 0,
                depth_reached: depth,
                stats: ctx.stats.clone(),
                pv: Vec::new(),
            };
        }

        if board.halfmove_clock() >= 100 {
            ctx.stats.fifty_returns += 1;
            return SearchResult {
                best_move: None,
                eval: 0,
                depth_reached: depth,
                stats: ctx.stats.clone(),
                pv: Vec::new(),
            };
        }

        let original_alpha = alpha;
        let root_hash = board.hash();
        let side_to_move = board.side_to_move();

        let mut best_eval = NEG_INF;
        let mut best_move = None;
        let mut best_pv = Vec::new();

        let mut all_moves = board.all_legal_moves(); // this returns mostly legal moves except for pawn and king legality(TODO)

        let mut legal_moves = 0; // counter of legal moves since pseudo moves might not flag checkmate

        let mut stopped = false;

        let tt_best_move = self.tt.get(board.hash()).and_then(|entry| entry.best_move);

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

            let undo = board.make_move(*mv);

            let child_hash = board.hash();

            if board.in_check(side_to_move) {
                board.undo_move(undo);
                continue;
            }
            legal_moves += 1;

            ctx.repetition_history.push(child_hash);

            let eval = -self.negamax(board, ctx, depth - 1, -beta, -alpha, 1, true);

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
                if (mv.kind == MoveType::Normal || mv.kind == MoveType::Castle)
                    && mv.promotion.is_none()
                {
                    self.history.add_bonus(side_to_move, mv.from, mv.to, depth);
                }
                break;
            }
        }
        if stopped {
            // return early before affecting logic with incomplete results
            return SearchResult {
                best_move,
                eval: best_eval,
                depth_reached: depth,
                stats: ctx.stats.clone(),
                pv: best_pv,
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
            };
        }

        let flag = if best_eval <= original_alpha {
            TTFlag::UpperBound
        } else if best_eval >= beta {
            TTFlag::LowerBound
        } else {
            TTFlag::Exact
        };
        self.tt.insert(
            root_hash,
            TTEntry {
                hash: root_hash,
                eval: best_eval,
                depth,
                flag,
                best_move,
            },
        );

        SearchResult {
            best_move,
            eval: best_eval,
            depth_reached: depth,
            stats: ctx.stats.clone(),
            pv: best_pv, // TODO: Implement principal variation
        }
    }
}
