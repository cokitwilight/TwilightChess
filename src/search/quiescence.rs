use crate::{
    board::{Board, Move, MoveType},
    eval::eval::evaluation_for_turn,
    moves::{legal, see::see},
    search::{
        Engine,
        engine::SearchContext,
        negamax::{CHECKMATE_SCORE, NEG_INF},
        tt::{TTEntry, TTFlag},
    },
    types::PieceType,
};

const DELTA_MARGIN: i32 = 200; // safe defualt for now

impl Engine {
    pub fn quiescence(
        &mut self,
        board: &mut Board,
        context: &mut SearchContext,
        depth: usize,
        mut alpha: i32,
        mut beta: i32,
        ply: usize,
    ) -> i32 {
        context.stats.qnodes += 1;

        if Engine::repetition_in_search(context, board.hash(), board.halfmove_clock() as usize) {
            context.stats.repetition_returns += 1;
            return 0;
        }
        if board.halfmove_clock() >= 100 {
            context.stats.fifty_returns += 1;
            return 0;
        }

        let original_alpha = alpha;
        let original_beta = beta;
        let hash = board.hash();
        let side_to_move = board.side_to_move();

        let mut tt_best_move: Option<Move> = None;

        context.stats.qtt.probes += 1;

        if let Some(entry) = self.qtt.get(hash) {
            debug_assert_eq!(
                entry.hash, hash,
                "QTT hash mismatch: key matched but entry.hash differed"
            );

            context.stats.qtt.hits += 1;
            tt_best_move = entry.best_move;

            if entry.depth >= depth {
                context.stats.qtt.usable += 1;

                match entry.flag {
                    TTFlag::Exact => {
                        context.stats.qtt.exact_returns += 1;
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
                    context.stats.qtt.bound_cutoffs += 1;
                    return entry.eval;
                }
            }
        }

        let in_check = board.in_check(board.side_to_move());

        let mut best_score = NEG_INF;
        let mut stand_pat = NEG_INF;
        let mut best_move: Option<Move> = None;

        let mut raw_moves = if in_check {
            let evasions = board.all_legal_moves();

            if evasions.is_empty() {
                let score = -CHECKMATE_SCORE + ply as i32;
                context.stats.qtt.stores += 1;
                self.qtt.insert(
                    hash,
                    TTEntry {
                        hash,
                        depth,
                        eval: score,
                        best_move: None,
                        flag: TTFlag::Exact,
                    },
                );
                return score;
            }

            if depth == 0 {
                // TODO: Later keep searching regardless since in check.
                let score = evaluation_for_turn(board);

                let flag = if score <= original_alpha {
                    TTFlag::UpperBound
                } else if score >= original_beta {
                    TTFlag::LowerBound
                } else {
                    TTFlag::Exact
                };

                context.stats.qtt.stores += 1;
                self.qtt.insert(
                    hash,
                    TTEntry {
                        hash,
                        depth,
                        eval: score,
                        best_move: None,
                        flag,
                    },
                );
                return score;
            }

            evasions
        } else {
            stand_pat = evaluation_for_turn(board);
            best_score = stand_pat;

            if stand_pat >= beta {
                context.stats.stand_pat_cutoffs += 1;
                context.stats.qtt.stores += 1;
                self.qtt.insert(
                    hash,
                    TTEntry {
                        hash,
                        depth,
                        eval: stand_pat,
                        best_move: None,
                        flag: TTFlag::LowerBound,
                    },
                );

                return stand_pat;
            }

            if alpha < stand_pat {
                alpha = stand_pat;
            }

            if depth == 0 {
                context.stats.qtt.stores += 1;
                self.qtt.insert(
                    hash,
                    TTEntry {
                        hash,
                        depth,
                        eval: stand_pat,
                        best_move: None,
                        flag: TTFlag::Exact,
                    },
                );
                return stand_pat;
            }
            board.all_pseudo_capture_moves()
            // includes promotions and quiet promotions
        };

        // do move ordering here
        if in_check {
            self.order_moves(
                // includes quiet moves and history heuristics
                board,
                &mut raw_moves,
                side_to_move,
                ply,
                context,
                None,
                tt_best_move,
            );
        } else {
            // only tt and see ordering
            self.q_order_moves(board, &mut raw_moves, tt_best_move);
        }

        for mv in raw_moves.iter() {
            // add see pruning and delta pruning here
            let can_prune = !in_check
                && board.phase > 8
                && mv.promotion.is_none()
                && alpha.abs() > CHECKMATE_SCORE - 1000;

            if can_prune {
                let captured_value = match mv.kind {
                    MoveType::EnPassant => PieceType::Pawn.value(),

                    _ => board.piece_at(mv.to).map(|p| p.kind.value()).unwrap_or(0),
                };
                if stand_pat + captured_value + DELTA_MARGIN < alpha {
                    context.stats.delta_prunes += 1;
                    continue;
                }

                if see(board, *mv) <= -500 {
                    context.stats.see_prunes += 1;
                    // less agressive pruning since see doesn't check legality yet
                    continue;
                }
            }

            let undo = board.make_move(*mv);

            let child_hash = board.hash();

            if board.in_check(side_to_move) {
                // illegal move
                context.stats.qillegal_moves += 1;
                board.undo_move(undo);
                continue;
            }

            context.repetition_history.push(child_hash);

            context.stats.qmoves_searched += 1;

            let score = -self.quiescence(board, context, depth - 1, -beta, -alpha, ply + 1);

            context.repetition_history.pop();

            board.undo_move(undo);

            if score > best_score {
                best_score = score;
                best_move = Some(*mv);
            }

            if score > alpha {
                alpha = score;
            }

            if score >= beta {
                context.stats.qtt.stores += 1;
                self.qtt.insert(
                    hash,
                    TTEntry {
                        hash,
                        depth,
                        eval: score,
                        best_move,
                        flag: TTFlag::LowerBound,
                    },
                );
                return score;
            }
        }

        let flag = if best_score <= original_alpha {
            TTFlag::UpperBound
        } else if best_score >= original_beta {
            TTFlag::LowerBound
        } else {
            TTFlag::Exact
        };

        context.stats.qtt.stores += 1;
        self.qtt.insert(
            hash,
            TTEntry {
                hash,
                depth,
                eval: best_score,
                best_move,
                flag,
            },
        );

        best_score
    }
}
