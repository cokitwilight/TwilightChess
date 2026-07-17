use crate::board::{Board, Move, MoveType, null_move_reduction};
use crate::engine::Engine;
use crate::engine::SearchContext;
use crate::engine::config::{CHECKMATE_SCORE, NEG_INF, RFP_MAX_DEPTH};
use crate::engine::pruning::lmr_reduction;
use crate::engine::tt::{TTEntry, TTFlag};
use crate::eval::evaluation_for_turn;

impl Engine {
    // Implementation for negamax function
    pub fn negamax(
        &mut self,
        board: &mut Board,
        context: &mut SearchContext,
        depth: usize,
        mut alpha: i32,
        mut beta: i32,
        ply: usize,
        allow_null_move: bool,
    ) -> i32 {
        context.stats.nodes += 1;

        // ADD DRAWING LOGIC HERE

        if Engine::repetition_in_search(context, board.hash(), board.halfmove_clock() as usize) {
            context.stats.repetition_returns += 1;
            return 0;
        }

        if board.halfmove_clock() >= 100 {
            // 50 move rule
            context.stats.fifty_returns += 1;
            return 0;
        }

        // add insufficient material check here

        if depth == 0 {
            return self.quiescence(board, context, context.limits.max_q_depth, alpha, beta, ply);
        }

        let original_alpha = alpha;
        // let original_beta = beta;

        let in_check = board.in_check(board.side_to_move());

        let mut tt_best_move: Option<Move> = None;

        context.stats.tt.probes += 1;

        if let Some(entry) = self.tt.get(board.hash) {
            debug_assert_eq!(
                entry.hash, board.hash,
                "TT hash mismatch: key matched but entry.hash differed. negamax.rs"
            );
            context.stats.tt.hits += 1;

            tt_best_move = entry.best_move;

            if entry.depth >= depth {
                context.stats.tt.usable += 1;

                match entry.flag {
                    TTFlag::Exact => {
                        context.stats.tt.exact_returns += 1;
                        return entry.eval;
                    }
                    TTFlag::LowerBound => {
                        alpha = alpha.max(entry.eval);
                    }
                    TTFlag::UpperBound => {
                        beta = beta.min(entry.eval);
                    }
                }
                if alpha >= beta {
                    context.stats.tt.bound_cutoffs += 1;
                    return entry.eval;
                }
            }
        }

        // reverse futility pruning
        if !in_check
            && beta == alpha + 1
            && depth <= RFP_MAX_DEPTH
            && beta.abs() < CHECKMATE_SCORE - 1000
            && ply > 0
            && board.phase() >= 6
        {
            context.stats.rfp_attempts += 1;

            let margin = 80 * depth as i32;
            // dynamic RFP margin

            let static_eval = evaluation_for_turn(board);

            if static_eval - margin >= beta {
                context.stats.rfp_cutoffs += 1;
                return beta;
            }
        }

        // null move here
        // 4 is a placeholder for now
        // use phase for now. Might not be viable though
        let null_enabled = self.config.search.null_move.enabled;
        let min_null_depth = self.config.search.null_move.minimum_depth;
        let min_null_phase = self.config.search.null_move.minimum_phase;

        if allow_null_move
            && null_enabled
            && !in_check
            && depth >= min_null_depth
            && board.phase >= min_null_phase
            && beta < CHECKMATE_SCORE - 1000
        {
            context.stats.null_attempts += 1;
            let reduction = null_move_reduction(depth);

            let undo = board.make_null_move();

            let score = -self.negamax(
                board,
                context,
                depth - 1 - reduction,
                -beta,
                -beta + 1,
                ply + 1,
                false,
            );

            board.undo_null_move(undo);

            if score >= beta {
                context.stats.null_cutoffs += 1;
                return beta;
            }
        }

        let mut moves = board.all_legal_moves();

        let mut legal_moves = 0;

        let side_to_move = board.side_to_move();

        self.order_moves(
            board,
            &mut moves,
            side_to_move,
            ply,
            context,
            None,
            tt_best_move,
        );

        let mut max_eval = NEG_INF;
        let mut best_move: Option<Move> = None;

        let mut did_cutoff = false;

        for (move_index, mv) in moves.iter().enumerate() {
            let currently_in_check = board.in_check(side_to_move);

            let undo = board.make_move(*mv);

            let child_hash = board.hash();

            // for stats debugging
            let was_killer = context.killer_moves.contains(ply, *mv);
            let history_score = self.history.get(side_to_move, mv.from, mv.to);

            context.stats.moves_searched += 1;
            legal_moves += 1;

            context.repetition_history.push(child_hash); // only store if valid move

            let gives_check = board.in_check(side_to_move.opposite());

            let reduction = if (mv.kind == MoveType::Normal || mv.kind == MoveType::Castle)
                && !currently_in_check
                && !gives_check
            {
                lmr_reduction(depth, move_index)
            } else {
                0
            };

            let mut eval: i32;

            if reduction > 0 {
                context.stats.lmr_attempts += 1;
                // null window search
                eval = -self.negamax(
                    board,
                    context,
                    depth - 1 - reduction,
                    -alpha - 1,
                    -alpha,
                    ply + 1,
                    true,
                );

                if eval > alpha {
                    context.stats.lmr_researched += 1;
                    // this move might improve alpha, research it at full depth
                    eval = -self.negamax(board, context, depth - 1, -beta, -alpha, ply + 1, true);
                }
            } else {
                eval = -self.negamax(board, context, depth - 1, -beta, -alpha, ply + 1, true);
            }

            context.repetition_history.pop();

            board.undo_move(undo);

            if eval > max_eval {
                max_eval = eval;
                best_move = Some(*mv);
            }

            alpha = alpha.max(eval);

            if alpha >= beta {
                context.stats.beta_cutoffs += 1;
                did_cutoff = true;

                let is_quiet = (mv.kind == MoveType::Normal || mv.kind == MoveType::Castle)
                    && mv.promotion.is_none();

                if is_quiet {
                    if was_killer {
                        context.stats.killer_cutoffs += 1;
                    } else if history_score > 0 {
                        context.stats.history_cutoffs += 1;
                    }

                    self.history.add_bonus(side_to_move, mv.from, mv.to, depth);
                    context.killer_moves.add(ply, *mv);
                }
                break;
            }
        }

        if legal_moves == 0 {
            if board.in_check(side_to_move) {
                return -CHECKMATE_SCORE + ply as i32;
            } else {
                return 0; // Stalemate
            }
        }

        // Store the best move in the search context for later use

        let flag = if max_eval <= original_alpha {
            TTFlag::UpperBound
        } else if did_cutoff {
            TTFlag::LowerBound
        } else {
            TTFlag::Exact
        };

        context.stats.tt.stores += 1;
        self.tt.insert(
            board.hash,
            TTEntry {
                hash: board.hash,
                depth,
                eval: max_eval,
                best_move,
                flag,
            },
        );

        max_eval
    }
}
