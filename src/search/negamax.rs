use crate::{
    board::{Board, Move, MoveType, null_move::null_move_reduction},
    search::{
        engine::{Engine, SearchContext},
        lmr::lmr_reduction,
        tt::{TTEntry, TTFlag},
    },
};

pub const CHECKMATE_SCORE: i32 = 100000;

pub const NEG_INF: i32 = -1_000_000_000;
pub const POS_INF: i32 = 1_000_000_000;

pub const MAX_Q_DEPTH: usize = 5;

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
            // Placeholder for quiescence search or evaluation function
            return self.quiescence(board, context, MAX_Q_DEPTH, alpha, beta, ply);
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

        // null move here
        // 4 is a placeholder for now
        // use phase for now might not be viable though
        if allow_null_move
            && !in_check
            && depth >= 4
            && board.phase > 8
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

        let mut moves = board.all_pseudo_moves();

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

            if board.in_check(side_to_move) {
                // illegal move
                context.stats.illegal_moves += 1;
                board.undo_move(undo);
                continue;
            }
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
