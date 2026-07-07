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
                context.stats.tt.usable_hits += 1;

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
            context.stats.null_moves += 1;
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

        let side_to_move = board.side_to_move();

        if moves.is_empty() {
            if in_check {
                return -CHECKMATE_SCORE + ply as i32;
            } else {
                return 0; // Stalemate
            }
        }

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
            // TODO: move index for lmr

            let parent_hash = board.hash;
            let currently_in_check = board.in_check(side_to_move);

            let undo = board.make_move(*mv);

            if board.in_check(side_to_move) {
                // illegal move
                board.undo_move(undo);
                continue;
            }

            context.repetition_history.push(parent_hash); // only store if valid move

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
                context.stats.lmr_nodes += 1;
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

            board.undo_move(undo);
            context.repetition_history.pop();

            if eval > max_eval {
                max_eval = eval;
                best_move = Some(*mv);
            }

            alpha = alpha.max(eval);

            if alpha >= beta {
                // ADD KILLER MOVE AND HISTORY HEURISTIC HERE
                did_cutoff = true;
                if mv.kind == MoveType::Normal || mv.kind == MoveType::Castle {
                    // quiet move
                    self.history.add_bonus(side_to_move, mv.from, mv.to, depth);
                    context.killer_moves.add(ply, *mv);
                }
                break;
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

// pub fn negamax(
//         &mut self,
//         board: &mut Board,
//         depth: usize,
//         mut alpha: i32,
//         mut beta: i32,
//         search_history: &mut Vec<u64>,
//         ply: usize,
//     ) -> i32 {
//         self.nodes += 1;

//         // if self.is_repetition_in_search(search_history, board.hash) {
//         //     self.repetition_returns += 1;
//         //     return 0;
//         // }

//         // if board.is_fifty_move_draw() {
//         //     self.fifty_returns += 1;
//         //     return 0;
//         // }

//         if depth == 0 {
//             return 0;
//             // return self.quiescence(board, alpha, beta, MAX_Q_DEPTH, search_history, ply);
//         }

//         let mut moves = board.all_legal_moves();

//         if moves.is_empty() {
//             if board.in_check() {
//                 return -CHECKMATE_SCORE + ply as i32;
//             } else {
//                 return 0;
//             }
//         }

//         let side_to_move = board.get_turn();

//         self.order_moves(board, &mut moves, side_to_move, ply, None, tt_best_move);

//         let mut max_eval = NEG_INF;
//         let mut best_move: Option<Move> = None;

//         // println!("Line 787: negamax");

//         for (move_index, mv) in moves.iter().enumerate() {
//             let parent_hash = board.hash;

//             let in_check = board.in_check();

//             let undo = board.make_move(*mv);
//             search_history.push(parent_hash);

//             let quiet = Engine::is_quiet_move(*mv);

//             let gives_check = board.in_check();

//             // let reduction = 0; // compare with regular

//             let reduction = if quiet && !in_check && !gives_check {
//                 Self::lmr_reduction(depth, move_index)
//             } else {
//                 0
//             };

//             let mut eval;

//             if reduction > 0 {
//                 // Reduced null-window search.
//                 eval = -self.negamax(
//                     board,
//                     depth - 1 - reduction,
//                     -alpha - 1,
//                     -alpha,
//                     search_history,
//                     ply + 1,
//                 );

//                 // If the reduced search says this move may improve alpha,
//                 // re-search it at full depth with a full window.
//                 if eval > alpha {
//                     eval = -self.negamax(board, depth - 1, -beta, -alpha, search_history, ply + 1);
//                 }
//             } else {
//                 // Normal full-depth full-window search.
//                 eval = -self.negamax(board, depth - 1, -beta, -alpha, search_history, ply + 1);
//             }

//             search_history.pop();
//             board.undo_move(undo);

//             if eval > max_eval {
//                 max_eval = eval;
//                 best_move = Some(*mv);
//             }

//             alpha = alpha.max(eval);

//             if alpha >= beta {
//                 if Engine::is_quiet_move(*mv) {
//                     self.store_killer_move(ply, *mv);
//                     self.add_history_bonus(side_to_move, *mv, depth);
//                 }
//                 break;
//             }
//         }

//         let flag = if max_eval <= original_alpha {
//             TTFlag::UpperBound
//         } else if max_eval >= original_beta {
//             TTFlag::LowerBound
//         } else {
//             TTFlag::Exact
//         };

//         self.tt.insert(
//             board.hash,
//             TTEntry {
//                 hash: board.hash,
//                 depth,
//                 eval: max_eval,
//                 best_move,
//                 flag,
//             },
//         );

//         max_eval
//     }
