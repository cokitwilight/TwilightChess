use crate::board::{Board, Move, MoveList, MoveType, null_move_reduction};
use crate::engine::config::{CHECKMATE_SCORE, NEG_INF};
use crate::engine::history::HistoryKey;
use crate::engine::ordering::see;
use crate::engine::pruning::lmr::LMR_SCALE_I32;
use crate::engine::search::search::is_insufficient_material;
use crate::engine::search_stats::{MAX_TRACKED_DEPTH, MOVE_INDEX_BUCKETS, REDUCTION_BUCKETS};
use crate::engine::tt::{TTEntry, TTFlag, TTNodeType, score_from_tt, score_to_tt};
use crate::engine::{Engine, MATE_THRESHOLD, MAX_PLY, SearchStackEntry};
use crate::engine::{SearchContext, SearchOptions};
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
        options: SearchOptions,
    ) -> i32 {
        // every 2048 nodes check if it should stop rather than expensively checking each time.
        if context.stopped || (context.stats.total_nodes() & 2047 == 0 && context.should_stop()) {
            context.stats.terminal_stats.stopped_returns += 1;
            return 0;
        }

        context.stats.node_stats.main += 1;

        // Classify the original window before the TT potentially narrows it.
        let is_pv = beta != alpha + 1;
        if is_pv {
            context.stats.node_stats.pv += 1;
        } else {
            context.stats.node_stats.non_pv += 1;
        }

        if ply >= MAX_PLY as usize - 1 {
            context.stats.terminal_stats.max_ply_returns += 1;
            return evaluation_for_turn(board);
        }

        if depth == 0 {
            // FOR BENCHMARKING
            // return evaluation_for_turn(board);
            return self.quiescence(
                board,
                context,
                context.limits.max_q_depth,
                alpha,
                beta,
                ply,
                0,
            );
        }

        // ADD DRAWING LOGIC HERE

        if Engine::repetition_in_search(context, board.hash(), board.halfmove_clock() as usize) {
            context.stats.draw_stats.repetition_returns += 1;
            return 0;
        }

        if board.halfmove_clock() >= 100 {
            // 50 move rule
            context.stats.draw_stats.fifty_move_returns += 1;
            return 0;
        }

        if board.phase() < 8 && is_insufficient_material(&board) {
            context.stats.draw_stats.insufficient_material_returns += 1;
            return 0;
        }

        let original_alpha = alpha;
        // let original_beta = beta;

        let in_check = board.in_check(board.side_to_move());
        if in_check {
            context.stats.node_stats.in_check += 1;
        }

        let mut tt_best_move: Option<Move> = None;

        let mut candidate_move: Option<(Move, i32)> = None;

        context.stats.tt_stats.main.probes += 1;

        if let Some(entry) = self.tt.get(board.hash, TTNodeType::Main) {
            context.stats.tt_stats.main.hits += 1;

            match entry.flag {
                TTFlag::Exact => context.stats.tt_stats.main.exact_hits += 1,
                TTFlag::LowerBound => context.stats.tt_stats.main.lower_bound_hits += 1,
                TTFlag::UpperBound => context.stats.tt_stats.main.upper_bound_hits += 1,
            }
            if entry.best_move.is_some() {
                context.stats.tt_stats.main.move_hits += 1;
            }
            if entry.depth < depth && options.excluded_move.is_none() {
                context.stats.tt_stats.main.depth_rejected_hits += 1;
            }

            let tt_score = score_from_tt(entry.eval, ply);

            tt_best_move = entry.best_move;

            if entry.depth >= depth && options.excluded_move.is_none() {
                context.stats.tt_stats.main.usable += 1;

                match entry.flag {
                    TTFlag::Exact => {
                        context.stats.tt_stats.main.exact_returns += 1;
                        return tt_score;
                    }
                    TTFlag::LowerBound => {
                        alpha = alpha.max(tt_score);
                    }
                    TTFlag::UpperBound => {
                        beta = beta.min(tt_score);
                    }
                }
                if alpha >= beta {
                    context.stats.tt_stats.main.bound_cutoffs += 1;
                    return tt_score;
                }
            }

            let suitable_for_singular = options.excluded_move.is_none()
                && self.config.search.singular.enabled
                && options.allow_singular
                && depth >= self.config.search.singular.minimum_depth
                && entry.depth >= depth.saturating_sub(2)
                && matches!(entry.flag, TTFlag::Exact | TTFlag::LowerBound)
                && tt_score.abs() < MATE_THRESHOLD;

            if suitable_for_singular {
                if let Some(mv) = entry.best_move {
                    candidate_move = Some((mv, tt_score));
                }
            }
        }

        let mut static_eval: Option<i32> = None;

        let can_rfp = !in_check
            && self.config.search.rfp.enabled
            && options.excluded_move.is_none()
            && !is_pv
            && beta < MATE_THRESHOLD
            && alpha > -MATE_THRESHOLD
            && ply > 0
            && board.phase() >= self.config.search.rfp.min_phase
            && depth <= self.config.search.rfp.max_depth;

        // reverse futility pruning
        if can_rfp {
            context.stats.rfp_stats.attempts += 1;
            let depth_bucket = usize::from(depth).min(MAX_TRACKED_DEPTH - 1);
            context.stats.rfp_stats.attempts_by_depth.bins[depth_bucket] += 1;

            let margin = self.config.search.rfp.margin_factor * depth as i32;
            // dynamic RFP margin

            let eval = evaluation_for_turn(board);

            if eval - margin >= beta {
                context.stats.rfp_stats.cutoffs += 1;
                context.stats.rfp_stats.cutoffs_by_depth.bins[depth_bucket] += 1;
                return beta;
            } else {
                static_eval = Some(eval);
            }
        }

        // null move here
        // 4 is a placeholder for now
        // use phase for now. Might not be viable though

        let mut can_null_prune = self.config.search.null_move.enabled
            && options.allow_null_move
            && options.excluded_move.is_none()
            && !in_check
            && depth >= self.config.search.null_move.minimum_depth
            && board.phase >= self.config.search.null_move.minimum_phase
            && beta.abs() < MATE_THRESHOLD
            && !is_pv;

        if can_null_prune {
            let eval = *static_eval.get_or_insert_with(|| evaluation_for_turn(board));

            if eval < beta {
                can_null_prune = false;
            }
        }

        if can_null_prune {
            context.stats.null_move_stats.attempts += 1;
            let depth_bucket = usize::from(depth).min(MAX_TRACKED_DEPTH - 1);
            context.stats.null_move_stats.attempts_by_depth.bins[depth_bucket] += 1;

            let reduction = null_move_reduction(depth);

            let undo = board.make_null_move();

            let mut search_options = options;

            search_options.allow_null_move = false;

            let score = -self.negamax(
                board,
                context,
                depth - 1 - reduction,
                -beta,
                -beta + 1,
                ply + 1,
                search_options,
            );

            board.undo_null_move(undo);

            if context.stopped {
                return 0;
            }

            if score >= beta {
                context.stats.null_move_stats.cutoffs += 1;
                context.stats.null_move_stats.cutoffs_by_depth.bins[depth_bucket] += 1;
                return beta;
            }
        }

        // for now call legal moves instead of pseudo moves since it is relatively fast since we added pin and check masks
        // NOTE: Legal moves will likely equal searched moves
        let mut moves = board.all_legal_moves();

        if moves.is_empty() {
            if in_check {
                context.stats.terminal_stats.checkmates += 1;
                return -CHECKMATE_SCORE + ply as i32;
            } else {
                context.stats.terminal_stats.stalemates += 1;
                return 0; // Stalemate
            }
        }
        let mut searched_moves = 0;

        let side_to_move = board.side_to_move();

        let ordering_tt_move = if options.excluded_move == tt_best_move {
            None
        } else {
            tt_best_move
        };

        self.order_moves(
            board,
            &mut moves,
            side_to_move,
            ply,
            context,
            None,
            ordering_tt_move,
        );

        let mut max_eval = NEG_INF;
        let mut best_move: Option<Move> = None;

        let mut did_cutoff = false;

        let can_fut = self.config.search.fut.enabled
            && depth <= self.config.search.fut.max_depth
            && options.excluded_move.is_none()
            && !in_check
            && alpha.abs() < MATE_THRESHOLD
            && depth > 0
            && !is_pv;

        if can_fut && static_eval.is_none() {
            let eval = evaluation_for_turn(board);
            static_eval = Some(eval);
        }

        let normal_child_options = SearchOptions {
            allow_null_move: true,
            allow_singular: options.allow_singular,
            excluded_move: None,
        };

        if let Some(eval) = static_eval {
            context.stack[ply].static_eval = eval;
        } else {
            let eval = evaluation_for_turn(board);
            context.stack[ply].static_eval = eval;
            static_eval = Some(eval);
        }

        let mut searched_quiets: MoveList = MoveList::new();

        for mv in moves.iter() {
            // singular extension before make move
            if Some(*mv) == options.excluded_move {
                continue;
            }

            let is_quiet = mv.kind() == MoveType::Normal && mv.promotion().is_none();
            let gives_check = board.move_gives_check(mv);

            let piece = board
                .piecetype_at(mv.from())
                .expect("No piece in board in negamax!");

            let curr_key = HistoryKey::new(side_to_move, piece, mv.to());

            let was_killer = context.killer_moves.contains(ply, *mv);
            let history_score = if is_quiet {
                self.history.get_quiet_score(&context.stack, ply, curr_key)
            } else {
                0
            };

            context.stack[ply + 1] = SearchStackEntry {
                mv: Some(*mv),
                piece: Some(piece),
                history_index: Some(curr_key),
                static_eval: 0,
            };

            let mut extension: u16 = 0;

            if let Some(candidate) = candidate_move {
                if candidate.0 == *mv {
                    context.stats.singular_stats.attempts += 1;

                    let margin = self.config.search.singular.base_margin
                        + self.config.search.singular.depth_margin * depth as i32;

                    let singular_beta = candidate.1 - margin;
                    let verification_depth = depth.saturating_sub(1) / 2;

                    let nodes_before = context.stats.total_nodes();

                    let verification_score = self.negamax(
                        board,
                        context,
                        verification_depth,
                        singular_beta - 1,
                        singular_beta,
                        ply, // Same position: do not increase ply
                        SearchOptions {
                            allow_null_move: false,
                            allow_singular: false,
                            excluded_move: Some(candidate.0),
                        },
                    );

                    let nodes_after = context.stats.total_nodes();

                    context.stats.singular_stats.verification_nodes += nodes_after - nodes_before;

                    if context.stopped {
                        return 0;
                    }

                    if verification_score < singular_beta {
                        extension = 1;
                        context.stats.singular_stats.extensions += 1;
                    } else {
                        context.stats.singular_stats.fail_highs += 1;
                    }
                }
            }

            let full_child_depth = depth.saturating_sub(1).saturating_add(extension);

            let undo = board.make_move(*mv);

            if can_fut
                && searched_moves > 0
                && is_quiet
                && !gives_check
                && extension == 0
                && Some(*mv) != tt_best_move
            {
                context.stats.fut_stats.attempts += 1;
                let depth_bucket = usize::from(depth).min(MAX_TRACKED_DEPTH - 1);
                context.stats.fut_stats.attempts_by_depth.bins[depth_bucket] += 1;
                let mut margin = depth * self.config.search.fut.margin;

                if self.config.search.fut.history_enabled {
                    if history_score > self.config.search.fut.good_history {
                        margin += depth * self.config.search.fut.history_margin;
                    } else if history_score < self.config.search.fut.bad_history {
                        margin += depth * self.config.search.fut.history_margin;
                    }
                }

                let eval = static_eval.expect("No available static eval in negamax!");

                if eval + margin as i32 <= alpha {
                    context.stats.fut_stats.pruned_moves += 1;
                    context.stats.fut_stats.pruned_moves_by_depth.bins[depth_bucket] += 1;
                    board.undo_move(undo);
                    continue;
                }
            }

            let child_hash = board.hash();
            context.repetition_history.push(child_hash); // only store if valid move

            let can_lmr = is_quiet
                && !in_check
                && !gives_check
                && self.config.search.lmr.enabled
                && options.excluded_move.is_none()
                && extension == 0
                && Some(*mv) != tt_best_move
                && depth >= 3
                && searched_moves >= 3;

            let reduction = if can_lmr {
                let mut current_reduction = self
                    .lmr_table
                    .get(depth as usize, searched_moves as usize + 1);

                if self.config.search.lmr.history_enabled {
                    let baseline_red = current_reduction / LMR_SCALE_I32;

                    let history_adjustement = history_score / self.config.search.lmr.history_scale;

                    current_reduction -= history_adjustement;

                    let history_red = current_reduction / LMR_SCALE_I32;

                    if history_red < baseline_red {
                        context.stats.lmr_stats.history_improvements += 1;
                    } else if history_red > baseline_red {
                        context.stats.lmr_stats.history_reductions += 1;
                    }
                }

                let red = current_reduction / LMR_SCALE_I32;

                red.clamp(0, depth as i32 - 2)
            } else {
                0
            };

            let reduced_child_depth = full_child_depth.saturating_sub(reduction as u16);

            let mut eval: i32;

            if searched_moves == 0 {
                eval = -self.negamax(
                    board,
                    context,
                    full_child_depth,
                    -beta,
                    -alpha,
                    ply + 1,
                    normal_child_options,
                );

                if context.stopped {
                    context.repetition_history.pop();
                    board.undo_move(undo);
                    return 0;
                }
            } else {
                if can_lmr {
                    context.stats.lmr_stats.attempts += 1;
                    context.stats.lmr_stats.eligible_moves += 1;
                    context.stats.lmr_stats.reduction_plies += reduction as u64;

                    let depth_bucket = usize::from(depth).min(MAX_TRACKED_DEPTH - 1);
                    context.stats.lmr_stats.attempts_by_depth.bins[depth_bucket] += 1;

                    let reduction_bucket = (reduction as usize).min(REDUCTION_BUCKETS - 1);
                    context.stats.lmr_stats.reduction_histogram.bins[reduction_bucket] += 1;

                    if reduction > 0 {
                        context.stats.lmr_stats.reduced_moves += 1;
                    } else {
                        context.stats.lmr_stats.zero_reduction_moves += 1;
                    }
                }

                // null window search reduced depth
                context.stats.pvs_stats.null_window_searches += 1;
                eval = -self.negamax(
                    board,
                    context,
                    reduced_child_depth,
                    -alpha - 1,
                    -alpha,
                    ply + 1,
                    normal_child_options,
                );

                if context.stopped {
                    context.repetition_history.pop();
                    board.undo_move(undo);
                    return 0;
                }

                if eval > alpha {
                    context.stats.pvs_stats.alpha_improvements += 1;

                    if reduction > 0 {
                        context.stats.lmr_stats.researches += 1;
                        let depth_bucket = usize::from(depth).min(MAX_TRACKED_DEPTH - 1);
                        context.stats.lmr_stats.researches_by_depth.bins[depth_bucket] += 1;

                        // null window search full depth
                        eval = -self.negamax(
                            board,
                            context,
                            full_child_depth,
                            -alpha - 1,
                            -alpha,
                            ply + 1,
                            normal_child_options,
                        );

                        if context.stopped {
                            context.repetition_history.pop();
                            board.undo_move(undo);
                            return 0;
                        }

                        if eval > alpha {
                            context.stats.lmr_stats.research_alpha_improvements += 1;
                        }
                        if eval >= beta {
                            context.stats.lmr_stats.research_cutoffs += 1;
                        }
                    }

                    if eval > alpha && eval < beta {
                        // this move might improve alpha, research it at full depth
                        // full window search, full depth
                        context.stats.pvs_stats.full_window_researches += 1;
                        eval = -self.negamax(
                            board,
                            context,
                            full_child_depth,
                            -beta,
                            -alpha,
                            ply + 1,
                            normal_child_options,
                        );

                        if context.stopped {
                            context.repetition_history.pop();
                            board.undo_move(undo);
                            return 0;
                        }

                        if eval >= beta {
                            context.stats.pvs_stats.research_cutoffs += 1;
                        }
                    }
                }
            }

            searched_moves += 1;

            context.stats.move_stats.main_searched += 1;

            context.repetition_history.pop();
            board.undo_move(undo);

            if eval > max_eval {
                max_eval = eval;
                best_move = Some(*mv);
            }

            alpha = alpha.max(eval);

            if alpha >= beta {
                context.stats.cutoff_stats.beta += 1;

                let cutoff_index = searched_moves as usize;
                let cutoff_bucket = cutoff_index.saturating_sub(1).min(MOVE_INDEX_BUCKETS - 1);
                context
                    .stats
                    .move_ordering_stats
                    .cutoff_move_index_histogram
                    .bins[cutoff_bucket] += 1;
                context.stats.move_ordering_stats.cutoff_move_index_sum += cutoff_index as u64;
                context.stats.move_ordering_stats.cutoff_move_index_max = context
                    .stats
                    .move_ordering_stats
                    .cutoff_move_index_max
                    .max(cutoff_index as u64);

                if Some(*mv) == tt_best_move {
                    context.stats.move_ordering_stats.tt_move_cutoffs += 1;
                } else if matches!(mv.kind(), MoveType::Capture | MoveType::EnPassant) {
                    if see(board, *mv) >= 0 {
                        context.stats.move_ordering_stats.winning_capture_cutoffs += 1;
                    } else {
                        context.stats.move_ordering_stats.losing_capture_cutoffs += 1;
                    }
                } else if was_killer {
                    context.stats.move_ordering_stats.killer_move_cutoffs += 1;
                } else if history_score > 0 {
                    context.stats.move_ordering_stats.history_move_cutoffs += 1;
                }

                if searched_moves == 1 {
                    context.stats.cutoff_stats.first_move_beta += 1;
                }

                did_cutoff = true;

                if is_quiet && options.excluded_move.is_none() {
                    if was_killer {
                        context.stats.history_stats.killer_cutoffs += 1;
                    } else if history_score > 0 {
                        context.stats.history_stats.history_cutoffs += 1;
                    }

                    context.killer_moves.add(ply, *mv);
                    self.history.add_quiet_bonus(context, ply, depth, curr_key);

                    for _ in searched_quiets.iter() {
                        self.history.add_quiet_malus(context, ply, depth, curr_key);
                    }
                }

                break;
            }

            if is_quiet {
                searched_quiets.push(*mv);
            }
        }

        if searched_moves == 0 {
            debug_assert!(options.excluded_move.is_some());

            context.stats.singular_stats.no_alternatives += 1;

            // This is the lower edge of the null window:
            // singular_beta - 1.
            return alpha;
        }

        // Store the best move in the search context for later use

        let flag = if max_eval <= original_alpha {
            TTFlag::UpperBound
        } else if did_cutoff {
            TTFlag::LowerBound
        } else {
            TTFlag::Exact
        };

        if options.excluded_move.is_none() {
            context.stats.tt_stats.main.stores += 1;
            let insert_result = self.tt.insert(
                board.hash,
                TTEntry {
                    depth,
                    eval: score_to_tt(max_eval, ply),
                    best_move,
                    flag,
                    node_type: TTNodeType::Main,
                },
            );
            context.stats.tt_stats.main.record_insert(insert_result);
        }

        max_eval
    }
}
