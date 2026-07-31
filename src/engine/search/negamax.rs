use crate::board::{Board, Move, MoveType, null_move_reduction};
use crate::engine::SearchContext;
use crate::engine::config::{CHECKMATE_SCORE, NEG_INF};
use crate::engine::pruning::lmr_reduction;
use crate::engine::search::search::is_insufficient_material;
use crate::engine::tt::{TTEntry, TTFlag, TTNodeType, score_from_tt, score_to_tt};
use crate::engine::{Engine, MATE_THRESHOLD};
use crate::eval::evaluation_for_turn;

impl Engine {
    // Implementation for negamax function
    pub fn negamax(
        &mut self,
        board: &mut Board,
        context: &mut SearchContext,
        depth: u16,
        mut alpha: i32,
        mut beta: i32,
        ply: usize,
        allow_null_move: bool,
    ) -> i32 {
        context.stats.nodes += 1;

        if depth == 0 {
            return self.quiescence(board, context, context.limits.max_q_depth, alpha, beta, ply);
        }

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

        if board.phase() < 8 && is_insufficient_material(&board) {
            context.stats.insufficient_returns += 1;
            return 0;
        }

        let original_alpha = alpha;
        // let original_beta = beta;

        let in_check = board.in_check(board.side_to_move());

        let mut tt_best_move: Option<Move> = None;

        context.stats.tt.probes += 1;

        if let Some(entry) = self.tt.get(board.hash) {
            context.stats.tt.hits += 1;

            let tt_score = score_from_tt(entry.eval, ply);

            tt_best_move = entry.best_move;

            if entry.depth >= depth && entry.node_type == TTNodeType::Main {
                context.stats.tt.usable += 1;

                match entry.flag {
                    TTFlag::Exact => {
                        context.stats.tt.exact_returns += 1;
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
                    context.stats.tt.bound_cutoffs += 1;
                    return tt_score;
                }
            }
        }

        let mut static_eval: Option<i32> = None;

        let can_rfp = !in_check
            && self.config.search.rfp.enabled
            && beta == alpha + 1
            && beta < MATE_THRESHOLD
            && ply > 0
            && board.phase() >= self.config.search.rfp.min_phase
            && depth <= self.config.search.rfp.max_depth;

        // reverse futility pruning
        if can_rfp {
            context.stats.rfp_attempts += 1;

            let margin = self.config.search.rfp.margin_factor * depth as i32;
            // dynamic RFP margin

            let eval = evaluation_for_turn(board);

            if eval - margin >= beta {
                context.stats.rfp_cutoffs += 1;
                return beta;
            } else {
                static_eval = Some(eval);
            }
        }

        // null move here
        // 4 is a placeholder for now
        // use phase for now. Might not be viable though

        let can_null_prune = self.config.search.null_move.enabled
            && allow_null_move
            && !in_check
            && depth >= self.config.search.null_move.minimum_depth
            && board.phase >= self.config.search.null_move.minimum_phase
            && beta < MATE_THRESHOLD;

        if can_null_prune {
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
        let mut searched_moves = 0;

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

        let can_rfp = self.config.search.fut.enabled
            && depth <= self.config.search.fut.max_depth
            && !in_check
            && alpha.abs() < MATE_THRESHOLD;

        if can_rfp && static_eval.is_none() {
            let eval = evaluation_for_turn(board);
            static_eval = Some(eval);
        }

        for mv in moves.iter() {
            let is_quiet = mv.kind == MoveType::Normal && mv.promotion.is_none();

            let undo = board.make_move(*mv);

            legal_moves += 1;

            let gives_check = board.in_check(side_to_move.opposite());

            if can_rfp && searched_moves > 0 && is_quiet && !gives_check {
                context.stats.fut_attempts += 1;
                let margin = depth * self.config.search.fut.margin;

                let eval = static_eval.expect("No available static eval in negamax!");

                if eval + margin as i32 <= alpha {
                    context.stats.fut_cutoffs += 1;
                    board.undo_move(undo);
                    continue;
                }
            }

            searched_moves += 1;
            context.stats.moves_searched += 1;

            let child_hash = board.hash();
            context.repetition_history.push(child_hash); // only store if valid move

            // for stats debugging
            let was_killer = context.killer_moves.contains(ply, *mv);
            let history_score = self.history.get(side_to_move, mv.from, mv.to);

            let reduction =
                if is_quiet && !in_check && !gives_check && self.config.search.lmr.enabled {
                    lmr_reduction(depth, searched_moves)
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
            if in_check {
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
                depth,
                eval: score_to_tt(max_eval, ply),
                best_move,
                flag,
                node_type: TTNodeType::Main,
            },
        );

        max_eval
    }
}
