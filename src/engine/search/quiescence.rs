use crate::board::{Board, Move, MoveType};
use crate::engine::Engine;
use crate::engine::SearchContext;
use crate::engine::config::{CHECKMATE_SCORE, MATE_THRESHOLD, NEG_INF};
use crate::engine::ordering::see;
use crate::engine::search::search::is_insufficient_material;
use crate::engine::tt::{TTEntry, TTFlag, TTNodeType, score_from_tt, score_to_tt};
use crate::eval::{evaluation_for_turn, lazy_eval_for_turn};
use crate::types::{Color, PieceType};

const DELTA_MARGIN: i32 = 200; // safe defualt for now

const LAZY_MARGIN: i32 = 300;

const MAX_CHECK_Q_PLIES: usize = 16;

impl Engine {
    pub fn quiescence(
        &mut self,
        board: &mut Board,
        context: &mut SearchContext,
        depth: u16,
        mut alpha: i32,
        mut beta: i32,
        ply: usize,
        check_plies: usize,
    ) -> i32 {
        if context.stats.nodes + context.stats.qnodes & 2047 == 0 {
            if context.should_stop() {
                return 0;
            }
        }

        context.stats.qnodes += 1;

        // NOTE: These will never be true in the first call to quiescence since negamax does this check first before calling if depth == 0 -> quiescence
        // this is why quiescence qtt probes always equals qnodes.
        if Engine::repetition_in_search(context, board.hash(), board.halfmove_clock() as usize) {
            context.stats.repetition_returns += 1;
            return 0;
        }
        if board.halfmove_clock() >= 100 {
            context.stats.fifty_returns += 1;
            return 0;
        }

        if board.phase() < 8 && is_insufficient_material(&board) {
            context.stats.insufficient_returns += 1;
            return 0;
        }

        let original_alpha = alpha;
        let original_beta = beta;
        let hash = board.hash();
        let side_to_move = board.side_to_move();

        let mut tt_best_move: Option<Move> = None;

        context.stats.qtt.probes += 1;

        if let Some(entry) = self.tt.get(hash) {
            context.stats.qtt.hits += 1;
            tt_best_move = entry.best_move;

            let tt_score = score_from_tt(entry.eval, ply);

            if entry.depth >= depth && entry.node_type == TTNodeType::Quiescence {
                context.stats.qtt.usable += 1;

                match entry.flag {
                    TTFlag::Exact => {
                        context.stats.qtt.exact_returns += 1;
                        return tt_score;
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
                    return tt_score;
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
                self.tt.insert(
                    hash,
                    TTEntry {
                        depth,
                        eval: score_to_tt(score, ply),
                        best_move: None,
                        flag: TTFlag::Exact,
                        node_type: TTNodeType::Quiescence,
                    },
                );
                return score;
            }

            // too many consecutive checks. Most likely a draw anyways.
            if check_plies > MAX_CHECK_Q_PLIES {
                // TODO: Later keep searching regardless since in check.
                let mut score = evaluation_for_turn(board);

                // since this is an awkward node do not store in tt.
                // Additionally conservatively make the position worse
                // maybe return alpha instead
                match board.side_to_move() {
                    Color::White => score = score - 100,
                    Color::Black => score = score + 100,
                }

                // let flag = if score <= original_alpha {
                //     TTFlag::UpperBound
                // } else if score >= original_beta {
                //     TTFlag::LowerBound
                // } else {
                //     TTFlag::Exact
                // };

                // context.stats.qtt.stores += 1;
                // self.tt.insert(
                //     hash,
                //     TTEntry {
                //         depth,
                //         eval: score_to_tt(score, ply),
                //         best_move: None,
                //         flag,
                //         node_type: TTNodeType::Quiescence,
                //     },
                // );
                return score;
            }

            evasions
        } else {
            if !in_check {
                let lazy_eval = lazy_eval_for_turn(board);

                if lazy_eval - LAZY_MARGIN >= beta || lazy_eval + LAZY_MARGIN <= alpha {
                    // TODO: Add this to the statistics tracker
                    return lazy_eval;
                }
            }

            stand_pat = evaluation_for_turn(board);

            best_score = stand_pat;

            if stand_pat >= beta {
                context.stats.stand_pat_cutoffs += 1;
                context.stats.qtt.stores += 1;
                self.tt.insert(
                    hash,
                    TTEntry {
                        depth,
                        eval: score_to_tt(stand_pat, ply),
                        best_move: None,
                        flag: TTFlag::LowerBound,
                        node_type: TTNodeType::Quiescence,
                    },
                );

                return stand_pat;
            }

            if alpha < stand_pat {
                alpha = stand_pat;
            }

            board.all_legal_capture_moves()
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
                && alpha.abs() > MATE_THRESHOLD;

            if can_prune {
                let captured_value = match mv.kind {
                    MoveType::EnPassant => PieceType::Pawn.value(),

                    _ => board.piece_at(mv.to).map(|p| p.kind.value()).unwrap_or(0),
                };
                if stand_pat + captured_value + DELTA_MARGIN < alpha {
                    context.stats.delta_prunes += 1;
                    continue;
                }

                // For now this is too expensive relative to node cutoffs(since its not legal the margin is too large)
                if self.config.search.see.enabled
                    && see(board, *mv) <= self.config.search.see.margin
                {
                    context.stats.see_prunes += 1;
                    // less agressive pruning since see doesn't check legality yet
                    continue;
                }
            }

            let undo = board.make_move(*mv);

            let child_hash = board.hash();

            context.repetition_history.push(child_hash);

            context.stats.qmoves_searched += 1;

            // let reduction = if in_check { 0 } else { 1 };

            let check_plies = if in_check { check_plies + 1 } else { 0 };

            let score =
                -self.quiescence(board, context, depth, -beta, -alpha, ply + 1, check_plies);

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
                self.tt.insert(
                    hash,
                    TTEntry {
                        depth,
                        eval: score_to_tt(score, ply),
                        best_move,
                        flag: TTFlag::LowerBound,
                        node_type: TTNodeType::Quiescence,
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
        self.tt.insert(
            hash,
            TTEntry {
                depth,
                eval: score_to_tt(best_score, ply),
                best_move,
                flag,
                node_type: TTNodeType::Quiescence,
            },
        );

        best_score
    }
}
