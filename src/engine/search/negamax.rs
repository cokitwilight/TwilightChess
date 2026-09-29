use crate::board::{Board, Move, MoveType, null_move_reduction};
use crate::engine::config::{CHECKMATE_SCORE, NEG_INF};
use crate::engine::history::HistoryKey;
use crate::engine::ordering::see;
use crate::engine::pruning::lmr::LMR_CAPTURE_VALUE;
use crate::engine::pruning::{LMR_SCALE_I32, lmp_history_limit, lmp_threshold};
use crate::engine::search::is_insufficient_material;
use crate::engine::search::iterative::{MAX_CAPTURES, MAX_QUIETS};
use crate::engine::search_stats::{
    CorrectionUpdateKind, MAX_TRACKED_DEPTH, MOVE_INDEX_BUCKETS, REDUCTION_BUCKETS,
};
use crate::engine::staged::ScoredMove;
use crate::engine::tt::{TTEntry, TTFlag, TTNodeType, score_from_tt, score_to_tt};
use crate::engine::{Engine, MATE_THRESHOLD, MAX_PLY, PickerFrame, SearchStackEntry};
use crate::engine::{SearchContext, SearchOptions};
use crate::eval::evaluation_for_turn;
use crate::moves::MoveGenInfo;
use crate::types::PieceType;

impl Engine {
    // Implementation for negamax function
    pub(crate) fn negamax(
        &mut self,
        board: &mut Board,
        context: &mut SearchContext,
        depth: u16,
        mut alpha: i32,
        mut beta: i32,
        ply: usize,
        picker_frame: PickerFrame,
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

        if ply >= MAX_PLY - 1 {
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
                picker_frame,
                0,
            );
        }

        // ADD DRAWING LOGIC HERE

        if Engine::repetition_in_search(
            context,
            board.board_hash(),
            board.halfmove_clock() as usize,
        ) {
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

        if let Some(entry) = self.tt.get(board.board_hash, TTNodeType::Main) {
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

        let raw_static_eval = evaluation_for_turn(board);

        let corrected_static_eval = if self.config.search.correction.enabled {
            let correction = self.history.correction.get(board);
            context
                .stats
                .correction_stats
                .record_main_lookup(correction);
            raw_static_eval + correction
        } else {
            raw_static_eval
        };

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

            if corrected_static_eval - margin >= beta {
                context.stats.rfp_stats.cutoffs += 1;
                context.stats.rfp_stats.cutoffs_by_depth.bins[depth_bucket] += 1;
                return beta;
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

        if can_null_prune && corrected_static_eval < beta {
            can_null_prune = false;
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
                picker_frame.child(),
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

        let mut searched_moves = 0;

        let side_to_move = board.side_to_move();

        let ordering_tt_move = if options.excluded_move == tt_best_move {
            None
        } else {
            tt_best_move
        };

        let mut move_picker = self.new_move_picker(
            0,
            context,
            side_to_move,
            ply,
            picker_frame,
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

        let normal_child_options = SearchOptions {
            allow_null_move: true,
            allow_singular: options.allow_singular,
            excluded_move: None,
        };

        if !in_check {
            context.stack[ply].static_eval = Some(raw_static_eval);
        }

        let is_improving = if ply >= 2 && !in_check {
            match context.stack[ply - 2].static_eval {
                Some(previous_eval) => raw_static_eval > previous_eval, // since stored value is raw as well only compare to the raw static eval
                None => false,
            }
        } else {
            false
        };

        let mut searched_quiets = [ScoredMove::new(); MAX_QUIETS];
        let mut q = 0usize;
        let mut quiet_moves_seen = 0usize;

        let mut searched_captures = [ScoredMove::new(); MAX_CAPTURES];
        let mut c = 0usize;

        let info = MoveGenInfo::calculate(board, side_to_move);

        while let Some(scored_mv) = move_picker.get_next(board, &info, context, &self.history) {
            // singular extension before make move
            let mv = &scored_mv.mv;
            if Some(*mv) == options.excluded_move {
                continue;
            }

            let is_quiet = mv.kind() == MoveType::Normal && mv.promotion().is_none();
            let is_capture = mv.kind() == MoveType::Capture || mv.kind() == MoveType::EnPassant;
            let gives_check = board.move_gives_check(mv);

            if is_quiet {
                quiet_moves_seen += 1;
            }

            let piece = board
                .piecetype_at(mv.from())
                .expect("No piece in board in negamax!");

            let curr_key = if let Some(key) = scored_mv.history_key {
                key
            } else {
                HistoryKey::new(side_to_move, piece, mv.to())
            };

            let captured_piece = if is_capture {
                if let Some(piece) = scored_mv.captured {
                    Some(piece)
                } else {
                    board.piecetype_at(mv.to())
                    // match mv.kind() {
                    //     MoveType::Capture => board.piecetype_at(mv.to()),
                    //     MoveType::EnPassant => Some(PieceType::Pawn),
                    //     _ => None,
                    // }
                }
            } else {
                None
            };

            let was_killer = context.killer_moves.contains(ply, *mv);
            let history_score = if is_quiet {
                self.history.get_quiet_score(&context.stack, ply, curr_key)
            } else if is_capture && Some(*mv) != tt_best_move {
                // tt best move could be a capture that does not have a capture piece
                self.history.get_capture_score(
                    curr_key,
                    captured_piece.expect("No piece in capture move for history lookup!"),
                )
            } else {
                0
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
                        picker_frame.child(),
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

            context.stack[ply + 1] = SearchStackEntry {
                mv: Some(*mv),
                piece: Some(piece),
                history_index: Some(curr_key),
                static_eval: None,
            };

            let full_child_depth = depth.saturating_sub(1).saturating_add(extension);

            // pruning section

            let can_reduce = (is_quiet || is_capture)
                && !in_check
                && Some(*mv) != tt_best_move
                && extension == 0
                && options.excluded_move.is_none();

            let can_prune =
                can_reduce && is_quiet && !gives_check && !is_pv && alpha.abs() < MATE_THRESHOLD;

            let can_lmp = can_prune
                && self.config.search.lmp.enabled
                && depth <= self.config.search.lmp.max_depth;

            if can_lmp && quiet_moves_seen > lmp_threshold(depth as usize) {
                context.stats.lmp_stats.attempts += 1;

                if history_score >= lmp_history_limit(depth as usize) {
                    context.stats.lmp_stats.history_rejections += 1;
                } else {
                    context.stats.lmp_stats.pruned_moves += 1;
                    continue;
                }
            }

            if can_fut && can_prune && searched_moves > 0 {
                context.stats.fut_stats.attempts += 1;
                let depth_bucket = usize::from(depth).min(MAX_TRACKED_DEPTH - 1);
                context.stats.fut_stats.attempts_by_depth.bins[depth_bucket] += 1;
                let mut margin = depth * self.config.search.fut.margin;

                if self.config.search.fut.history_enabled {
                    if history_score > self.config.search.fut.good_history {
                        margin += depth * self.config.search.fut.history_margin;
                    } else if history_score < self.config.search.fut.bad_history {
                        margin -= depth * self.config.search.fut.history_margin;
                    }
                }

                if corrected_static_eval + margin as i32 <= alpha {
                    context.stats.fut_stats.pruned_moves += 1;
                    context.stats.fut_stats.pruned_moves_by_depth.bins[depth_bucket] += 1;
                    continue;
                }
            }

            let undo = board.make_move(*mv);

            let child_hash = board.board_hash();
            context.repetition_history.push(child_hash); // only store if valid move

            let can_lmr =
                can_reduce && self.config.search.lmr.enabled && depth >= 3 && searched_moves >= 3;

            let reduction = if can_lmr {
                let base_reduction = self
                    .lmr_table
                    .get(depth as usize, searched_moves as usize + 1);
                let mut current_reduction = base_reduction;
                let mut history_delta = 0;
                let mut non_improving_delta = 0;
                let mut pv_delta = 0;
                let mut capture_delta = 0;
                let mut check_delta = 0;

                if self.config.search.lmr.history_enabled {
                    let baseline_red = current_reduction / LMR_SCALE_I32;

                    let history_adjustement = if is_quiet {
                        history_score / self.config.search.lmr.history_scale
                    } else if is_capture {
                        // Capture history is less than max quiet history.
                        history_score * 2 / self.config.search.lmr.history_scale
                    } else {
                        0
                    };

                    history_delta = -history_adjustement;
                    current_reduction += history_delta;

                    if history_delta != 0 {
                        context.stats.lmr_stats.history_adjustments += 1;
                    }

                    let history_red = current_reduction / LMR_SCALE_I32;

                    if history_red < baseline_red {
                        context.stats.lmr_stats.history_improvements += 1;
                    } else if history_red > baseline_red {
                        context.stats.lmr_stats.history_reductions += 1;
                    }
                }

                if !is_improving {
                    non_improving_delta = base_reduction / 3;
                    current_reduction += non_improving_delta;
                    if non_improving_delta != 0 {
                        context.stats.lmr_stats.non_improving_adjustments += 1;
                    }
                }

                if is_pv {
                    pv_delta = -(base_reduction / 3);
                    current_reduction += pv_delta;
                    if pv_delta != 0 {
                        context.stats.lmr_stats.pv_adjustments += 1;
                    }
                }

                if is_capture {
                    capture_delta =
                        -(LMR_CAPTURE_VALUE[captured_piece.unwrap().idx()] + LMR_SCALE_I32 / 2);
                    current_reduction += capture_delta;
                    context.stats.lmr_stats.capture_adjustments += 1;
                }

                if gives_check {
                    check_delta = -(LMR_SCALE_I32 / 2);
                    current_reduction += check_delta;
                    context.stats.lmr_stats.check_adjustments += 1;
                }

                let reduction_in_plies = |scaled_reduction: i32| {
                    (scaled_reduction / LMR_SCALE_I32).clamp(0, depth as i32 - 2)
                };
                let red = reduction_in_plies(current_reduction);

                if history_delta != 0
                    && reduction_in_plies(current_reduction - history_delta) != red
                {
                    context.stats.lmr_stats.history_effects += 1;
                }
                if non_improving_delta != 0
                    && reduction_in_plies(current_reduction - non_improving_delta) != red
                {
                    context.stats.lmr_stats.non_improving_effects += 1;
                }
                if pv_delta != 0 && reduction_in_plies(current_reduction - pv_delta) != red {
                    context.stats.lmr_stats.pv_effects += 1;
                }
                if capture_delta != 0
                    && reduction_in_plies(current_reduction - capture_delta) != red
                {
                    context.stats.lmr_stats.capture_effects += 1;
                }
                if check_delta != 0 && reduction_in_plies(current_reduction - check_delta) != red {
                    context.stats.lmr_stats.check_effects += 1;
                }

                red
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
                    picker_frame.child(),
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
                    picker_frame.child(),
                    normal_child_options,
                );

                if context.stopped {
                    context.repetition_history.pop();
                    board.undo_move(undo);
                    return 0;
                }

                if eval > alpha {
                    let mut research_depth = full_child_depth;

                    context.stats.pvs_stats.alpha_improvements += 1;

                    if reduction > 0 {
                        context.stats.lmr_stats.researches += 1;
                        let depth_bucket = usize::from(depth).min(MAX_TRACKED_DEPTH - 1);
                        context.stats.lmr_stats.researches_by_depth.bins[depth_bucket] += 1;

                        let deeper_search = reduced_child_depth < depth && eval > max_eval + 50;
                        let shallow_search = eval < max_eval + 10;

                        research_depth = if deeper_search {
                            research_depth + 1
                        } else if shallow_search {
                            research_depth - 1
                        } else {
                            research_depth
                        };

                        match research_depth.cmp(&full_child_depth) {
                            std::cmp::Ordering::Greater => {
                                context.stats.lmr_stats.dynamic_depth_increases += 1;
                            }
                            std::cmp::Ordering::Less => {
                                context.stats.lmr_stats.dynamic_depth_decreases += 1;
                            }
                            std::cmp::Ordering::Equal => {
                                context.stats.lmr_stats.dynamic_depth_unchanged += 1;
                            }
                        }

                        // null window search full depth
                        if research_depth > reduced_child_depth {
                            context.stats.lmr_stats.dynamic_searches += 1;
                            if research_depth != full_child_depth {
                                context.stats.lmr_stats.dynamic_searches_at_modified_depth += 1;
                            }
                            eval = -self.negamax(
                                board,
                                context,
                                research_depth,
                                -alpha - 1,
                                -alpha,
                                ply + 1,
                                picker_frame.child(),
                                normal_child_options,
                            );
                        }

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
                        if reduction > 0 {
                            context.stats.lmr_stats.dynamic_searches += 1;
                            if research_depth != full_child_depth {
                                context.stats.lmr_stats.dynamic_searches_at_modified_depth += 1;
                            }
                        }
                        eval = -self.negamax(
                            board,
                            context,
                            research_depth,
                            -beta,
                            -alpha,
                            ply + 1,
                            picker_frame.child(),
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

            let mut improved_alpha = false;

            if eval > max_eval {
                max_eval = eval;
                best_move = Some(*mv);
            }

            if alpha < eval {
                alpha = eval;
                improved_alpha = true;
            }

            // alpha = alpha.max(eval);

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
                    let see_value = if let Some(value) = scored_mv.see {
                        value
                    } else {
                        see(board, *mv, captured_piece)
                    };
                    if see_value >= 0 {
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

                if options.excluded_move.is_none() {
                    if is_quiet {
                        context.killer_moves.add(ply, *mv);
                        self.history.add_quiet_bonus(curr_key, context, ply, depth);

                        for scored_mv in &searched_quiets[..q] {
                            let history_key = if let Some(key) = scored_mv.history_key {
                                key
                            } else {
                                let mv = &scored_mv.mv;
                                let piece = board
                                    .piecetype_at(mv.from())
                                    .expect("No piece in board in negamax!");
                                HistoryKey::new(side_to_move, piece, mv.to())
                            };

                            self.history
                                .add_quiet_malus(history_key, context, ply, depth);
                        }
                    } else if is_capture {
                        let captured_piece = if let Some(piece) = captured_piece {
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

                        self.history
                            .add_capture_bonus(curr_key, captured_piece, depth);
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

                        self.history
                            .add_capture_malus(history_key, captured_piece, depth);
                    }
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

        if searched_moves == 0 {
            if options.excluded_move.is_some() {
                context.stats.singular_stats.no_alternatives += 1;

                // This is the lower edge of the null window:
                // singular_beta - 1.
                return alpha;
            }

            if in_check {
                context.stats.terminal_stats.checkmates += 1;
                return -CHECKMATE_SCORE + ply as i32;
            }

            context.stats.terminal_stats.stalemates += 1;
            return 0;
        }

        // update correction history

        let can_update_correction = self.config.search.correction.enabled
            && !in_check
            && options.excluded_move.is_none()
            && alpha.abs() < MATE_THRESHOLD
            && beta.abs() < MATE_THRESHOLD
            && depth < 3;

        if can_update_correction {
            context.stats.correction_stats.record_update_opportunity();

            if max_eval > original_alpha && max_eval < beta {
                // Exact
                let delta = max_eval - corrected_static_eval;
                self.history.correction.update(board, delta, depth);
                context
                    .stats
                    .correction_stats
                    .record_update(CorrectionUpdateKind::Exact, delta);
            } else if max_eval >= beta && beta > corrected_static_eval {
                // Fail High
                let delta = beta - corrected_static_eval;
                self.history.correction.update(board, delta, depth);
                context
                    .stats
                    .correction_stats
                    .record_update(CorrectionUpdateKind::FailHigh, delta);
            } else if max_eval <= original_alpha && original_alpha < corrected_static_eval {
                // Fail Low
                let delta = original_alpha - corrected_static_eval;
                self.history.correction.update(board, delta, depth);
                context
                    .stats
                    .correction_stats
                    .record_update(CorrectionUpdateKind::FailLow, delta);
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

        if options.excluded_move.is_none() {
            context.stats.tt_stats.main.stores += 1;
            let insert_result = self.tt.insert(
                board.board_hash,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bitboard::{H8, square};
    use crate::engine::SearchLimits;
    use crate::engine::config::POS_INF;
    use crate::engine::configs::EngineConfig;

    fn test_engine() -> Engine {
        let mut config = EngineConfig::standard();
        config.tt_size = 1;
        Engine::new(config)
    }

    #[test]
    fn staged_negamax_detects_checkmate_and_stalemate() {
        let cases = [
            ("7k/6Q1/6K1/8/8/8/8/8 b - - 0 1", -CHECKMATE_SCORE),
            ("7k/5Q2/6K1/8/8/8/8/8 b - - 0 1", 0),
        ];

        for (fen, expected) in cases {
            let mut engine = test_engine();
            let mut board = Board::from_fen(fen).expect("valid terminal FEN");
            let mut context =
                SearchContext::new(SearchLimits::depth(1, 1), vec![board.board_hash()]);

            let score = engine.negamax(
                &mut board,
                &mut context,
                1,
                NEG_INF + 1,
                POS_INF - 1,
                0,
                PickerFrame::ROOT,
                SearchOptions::NORMAL,
            );

            assert_eq!(score, expected, "wrong terminal score for {fen}");
        }
    }

    #[test]
    fn singular_verification_does_not_count_the_excluded_move_as_searched() {
        let mut engine = test_engine();
        let mut board = Board::from_fen("7k/6Q1/4K3/8/8/8/8/8 b - - 0 1").expect("valid FEN");
        let mut context = SearchContext::new(SearchLimits::depth(1, 1), vec![board.board_hash()]);
        let only_legal_move = Move::new(H8, square(6, 6), MoveType::Capture, None);
        let alpha = -100;

        let score = engine.negamax(
            &mut board,
            &mut context,
            1,
            alpha,
            alpha + 1,
            0,
            PickerFrame::ROOT,
            SearchOptions::singular_verification(only_legal_move),
        );

        assert_eq!(score, alpha);
        assert_eq!(context.stats.singular_stats.no_alternatives, 1);
        assert_eq!(context.stats.move_stats.main_searched, 0);
        assert_eq!(context.stats.terminal_stats.checkmates, 0);
    }
}
