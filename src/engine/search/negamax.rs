use crate::board::{Board, Move, MoveType};
use crate::engine::history::HistoryKey;
use crate::engine::ordering::see;
use crate::engine::pruning::{
    calculate_eval_improvement, calculate_fut_margin, calculate_lmr_research_depth,
    calculate_nmp_margin, calculate_see_threshold, lmp_history_limit, lmp_threshold,
    null_move_reduction, rfp_margin, singular_margin, singular_verification_depth,
};
use crate::engine::search::is_insufficient_material;
use crate::engine::search::iterative::{MAX_CAPTURES, MAX_QUIETS};
use crate::engine::search::{NodeType, NonPV, PV};
use crate::engine::search_stats::track_stat;
#[cfg(feature = "search-stats")]
use crate::engine::search_stats::{
    CorrectionUpdateKind, MAX_TRACKED_DEPTH, MOVE_INDEX_BUCKETS, REDUCTION_BUCKETS,
};
use crate::engine::staged::ScoredMove;
use crate::engine::tt::{TTFlag, TTNodeType, score_from_tt, score_to_tt};
use crate::engine::{Engine, MAX_PLY, PickerFrame, SearchStackEntry};
use crate::engine::{SearchContext, SearchOptions};
use crate::eval::evaluation_for_turn;
use crate::moves::MoveGenInfo;
use crate::types::{PieceType, Score, is_decisive, is_loss, is_valid, is_win, mated_in};

impl Engine {
    // Implementation for negamax function
    pub(crate) fn negamax<NODE: NodeType>(
        &mut self,
        board: &mut Board,
        context: &mut SearchContext,
        depth: u16,
        mut alpha: i32,
        mut beta: i32,
        cut_node: bool,
        ply: usize,
        picker_frame: PickerFrame,
        options: SearchOptions,
    ) -> i32 {
        // every 2048 nodes check if it should stop rather than expensively checking each time.
        if context.stopped || context.stats.total_nodes() & 2047 == 0 && context.should_stop() {
            track_stat!(context.stats.terminal_stats.stopped_returns += 1);
            return Score::ZERO;
        }

        context.stats.node_stats.main += 1;

        if NODE::PV {
            context.seldepth = context.seldepth.max(ply);
        }

        // Classify the original window before the TT potentially narrows it.

        track_stat! {
        if cut_node {
            context.stats.node_stats.cut += 1;
        }
        if NODE::ROOT {
            context.stats.node_stats.pv += 1;
            context.stats.node_stats.root += 1;
        } else if NODE::PV {
            context.stats.node_stats.pv += 1;
        } else {
            context.stats.node_stats.non_pv += 1
        }
        }

        if ply >= MAX_PLY - 1 {
            track_stat!(context.stats.terminal_stats.max_ply_returns += 1);
            return evaluation_for_turn(board);
        }

        if depth == 0 {
            return self.quiescence::<NODE>(
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

        if !NODE::ROOT {
            if Engine::repetition_in_search(
                context,
                board.board_hash(),
                board.halfmove_clock() as usize,
            ) {
                track_stat!(context.stats.draw_stats.repetition_returns += 1);
                return Score::ZERO;
            }

            if board.halfmove_clock() >= 100 {
                track_stat!(context.stats.draw_stats.fifty_move_returns += 1);
                return Score::ZERO;
            }

            if board.phase() < 8 && is_insufficient_material(board) {
                track_stat!(context.stats.draw_stats.insufficient_material_returns += 1);
                return Score::ZERO;
            }
        }

        let original_alpha = alpha;

        let in_check = board.in_check(board.side_to_move());

        track_stat! {
            if in_check {
                context.stats.node_stats.in_check += 1;
            }
        }

        let mut tt_best_move: Option<Move> = None;

        let mut candidate_move: Option<(Move, i32)> = None;

        track_stat!(context.stats.tt_stats.main.probes += 1);

        let mut tt_eval = Score::NONE;

        if let Some(entry) = self.tt.get(board.board_hash, TTNodeType::Main) {
            track_stat!(context.stats.tt_stats.main.hits += 1);

            track_stat!(match entry.flag.bound_type() {
                TTFlag::Exact => context.stats.tt_stats.main.exact_hits += 1,
                TTFlag::LowerBound => {
                    context.stats.tt_stats.main.lower_bound_hits += 1;
                }
                TTFlag::UpperBound => {
                    context.stats.tt_stats.main.upper_bound_hits += 1;
                }
            });
            if entry.best_move.is_some() {
                track_stat!(context.stats.tt_stats.main.move_hits += 1);
            }
            if u16::from(entry.depth) < depth && options.excluded_move.is_none() {
                track_stat!(context.stats.tt_stats.main.depth_rejected_hits += 1);
            }

            let tt_score = score_from_tt(entry.eval as i32, ply);

            tt_best_move = entry.best_move;

            tt_eval = i32::from(entry.static_eval);

            if !NODE::ROOT
                && is_valid(tt_score)
                && u16::from(entry.depth) >= depth
                && options.excluded_move.is_none()
            {
                track_stat!(context.stats.tt_stats.main.usable += 1);

                match entry.flag.bound_type() {
                    TTFlag::Exact => {
                        track_stat!(context.stats.tt_stats.main.exact_returns += 1);
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
                    track_stat!(context.stats.tt_stats.main.bound_cutoffs += 1);
                    return tt_score;
                }
            }

            let suitable_for_singular = !NODE::ROOT
                && options.excluded_move.is_none()
                && self.config.search.singular.enabled
                && options.allow_singular
                && depth >= self.config.search.singular.minimum_depth
                && u16::from(entry.depth) >= depth.saturating_sub(2)
                && matches!(entry.flag.bound_type(), TTFlag::Exact | TTFlag::LowerBound)
                && is_valid(tt_score)
                && !is_decisive(tt_score);

            if suitable_for_singular && let Some(mv) = entry.best_move {
                candidate_move = Some((mv, tt_score));
            }
        } else if let Some(entry) = self.tt.get_any(board.board_hash) {
            // A quiescence score cannot supply a main-search cutoff, but its
            // move and static evaluation remain useful at any search depth.
            tt_best_move = entry.best_move;
            tt_eval = i32::from(entry.static_eval);
        }

        let raw_static_eval = if is_valid(tt_eval) {
            tt_eval
        } else {
            evaluation_for_turn(board)
        };

        let correction = if self.config.search.correction.enabled {
            let c = self.history.correction.get(board);
            track_stat!(context.stats.correction_stats.record_main_lookup(c));
            c
        } else {
            0
        };

        let corrected_static_eval = raw_static_eval + correction;

        let improvement =
            calculate_eval_improvement(raw_static_eval, &context.stack, ply, in_check);

        let is_improving_2_plies = improvement > 0;
        track_stat!(context.stats.improvement_stats.record(improvement));

        let can_rfp = !NODE::PV
            && !in_check
            && self.config.search.rfp.enabled
            && options.excluded_move.is_none()
            && !is_win(beta)
            && !is_loss(alpha)
            && ply > 0
            && board.phase() >= self.config.search.rfp.min_phase
            && depth <= self.config.search.rfp.max_depth;

        // reverse futility pruning
        if can_rfp {
            #[cfg(feature = "search-stats")]
            let depth_bucket = usize::from(depth).min(MAX_TRACKED_DEPTH - 1);

            track_stat! {
                context.stats.rfp_stats.attempts += 1;
                context.stats.rfp_stats.attempts_by_depth.bins[depth_bucket] += 1;
            }

            let margin = rfp_margin(depth as i32, improvement, &self.config.search.rfp);
            // dynamic RFP margin

            if corrected_static_eval - margin >= beta {
                track_stat! {
                    context.stats.rfp_stats.cutoffs += 1;
                    context.stats.rfp_stats.cutoffs_by_depth.bins[depth_bucket] += 1;
                }

                // integer lerp
                let result = (3000 * corrected_static_eval + 7000 * beta) / 10000;

                return result;
            }
        }

        let can_null_prune = !NODE::PV
            && cut_node
            && self.config.search.null_move.enabled
            && options.allow_null_move
            && options.excluded_move.is_none()
            && !in_check
            && depth >= self.config.search.null_move.minimum_depth
            && board.phase >= self.config.search.null_move.minimum_phase
            && !is_decisive(beta)
            && corrected_static_eval >= calculate_nmp_margin(beta);

        if can_null_prune {
            #[cfg(feature = "search-stats")]
            let depth_bucket = usize::from(depth).min(MAX_TRACKED_DEPTH - 1);

            track_stat!(context.stats.null_move_stats.attempts += 1);
            track_stat!(context.stats.null_move_stats.attempts_by_depth.bins[depth_bucket] += 1);

            let reduction = null_move_reduction(depth);

            let undo = board.make_null_move();

            let mut search_options = options;

            search_options.allow_null_move = false;

            let score = -self.negamax::<NonPV>(
                board,
                context,
                depth - 1 - reduction,
                -beta,
                -beta + 1,
                false,
                ply + 1,
                picker_frame.child(),
                search_options,
            );

            board.undo_null_move(undo);

            if context.stopped {
                return Score::ZERO;
            }

            if score >= beta {
                track_stat!(context.stats.null_move_stats.cutoffs += 1);
                track_stat!(context.stats.null_move_stats.cutoffs_by_depth.bins[depth_bucket] += 1);
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

        let pv_move = if NODE::ROOT {
            context.previous_best_move
        } else {
            None
        };

        let mut move_picker = self.new_move_picker(
            0,
            context,
            side_to_move,
            ply,
            picker_frame,
            pv_move,
            ordering_tt_move,
        );

        let mut max_eval = -Score::INFINITE;
        let mut best_move: Option<Move> = None;

        let mut did_cutoff = false;

        let can_fut = !NODE::PV
            && self.config.search.fut.enabled
            && depth <= self.config.search.fut.max_depth
            && options.excluded_move.is_none()
            && !in_check
            && !is_decisive(alpha)
            && depth > 0;

        let normal_child_options = SearchOptions {
            allow_null_move: true,
            allow_singular: options.allow_singular,
            excluded_move: None,
        };

        if !in_check {
            context.stack[ply].static_eval = raw_static_eval;
        }

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
                } else if mv.kind() == MoveType::EnPassant {
                    Some(PieceType::Pawn)
                } else {
                    board.piecetype_at(mv.to())
                }
            } else {
                None
            };

            #[cfg(feature = "search-stats")]
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

            if let Some(candidate) = candidate_move
                && candidate.0 == *mv
            {
                track_stat!(context.stats.singular_stats.attempts += 1);

                let margin = singular_margin(depth, &self.config.search.singular);

                let singular_beta = candidate.1 - margin;
                let verification_depth = singular_verification_depth(depth);

                #[cfg(feature = "search-stats")]
                let nodes_before = context.stats.total_nodes();

                let verification_score = self.negamax::<NonPV>(
                    board,
                    context,
                    verification_depth,
                    singular_beta - 1,
                    singular_beta,
                    cut_node,
                    ply, // Same position: do not increase ply
                    picker_frame.child(),
                    SearchOptions {
                        allow_null_move: false,
                        allow_singular: false,
                        excluded_move: Some(candidate.0),
                    },
                );

                #[cfg(feature = "search-stats")]
                let nodes_after = context.stats.total_nodes();

                track_stat!(
                    context.stats.singular_stats.verification_nodes += nodes_after - nodes_before
                );

                if context.stopped {
                    return Score::ZERO;
                }

                if verification_score < singular_beta {
                    extension = 1;
                    track_stat!(context.stats.singular_stats.extensions += 1);
                } else {
                    track_stat!(context.stats.singular_stats.fail_highs += 1);
                }
            }

            context.stack[ply + 1] = SearchStackEntry {
                mv: Some(*mv),
                piece: Some(piece),
                history_index: Some(curr_key),
                static_eval: Score::NONE,
            };

            let full_child_depth = depth.saturating_sub(1).saturating_add(extension);

            // pruning section

            let can_reduce = !NODE::ROOT
                && (is_quiet || is_capture)
                && !in_check
                && Some(*mv) != tt_best_move
                && extension == 0
                && options.excluded_move.is_none();

            // Also protects the first search: -INFINITE counts as a loss.
            // A losing mate found so far must not hide an unsearched escape.
            let can_prune_moves = !is_loss(max_eval);
            let can_prune = !NODE::PV
                && can_reduce
                && can_prune_moves
                && is_quiet
                && !gives_check
                && !is_decisive(alpha);

            let can_lmp = can_prune
                && self.config.search.lmp.enabled
                && depth <= self.config.search.lmp.max_depth;

            if can_lmp && quiet_moves_seen > lmp_threshold(depth as usize) {
                track_stat!(context.stats.lmp_stats.attempts += 1);

                if history_score >= lmp_history_limit(depth as usize) {
                    track_stat!(context.stats.lmp_stats.history_rejections += 1);
                } else {
                    track_stat!(context.stats.lmp_stats.pruned_moves += 1);
                    continue;
                }
            }

            if can_fut && can_prune && searched_moves > 0 {
                #[cfg(feature = "search-stats")]
                let depth_bucket = usize::from(depth).min(MAX_TRACKED_DEPTH - 1);

                track_stat!(context.stats.fut_stats.attempts += 1);
                track_stat!(context.stats.fut_stats.attempts_by_depth.bins[depth_bucket] += 1);

                let margin = calculate_fut_margin(depth, history_score, &self.config.search.fut);

                if corrected_static_eval + margin <= alpha {
                    track_stat!(context.stats.fut_stats.pruned_moves += 1);
                    track_stat!(
                        context.stats.fut_stats.pruned_moves_by_depth.bins[depth_bucket] += 1
                    );
                    continue;
                }
            }

            let can_see = !NODE::PV
                && can_reduce
                && can_prune_moves
                && is_capture
                && !gives_check
                && !is_decisive(alpha);

            if self.config.search.see.enabled && can_see && searched_moves > 0 {
                #[cfg(feature = "search-stats")]
                let depth_bucket = usize::from(depth).min(MAX_TRACKED_DEPTH - 1);
                track_stat! {
                    context.stats.main_see_stats.attempts += 1;
                    context.stats.main_see_stats.attempts_by_depth.bins[depth_bucket] += 1;
                }
                let see_value = if let Some(value) = scored_mv.see {
                    value
                } else {
                    see(board, *mv, scored_mv.captured)
                };
                if see_value < calculate_see_threshold(&self.config.search.see) * 2 {
                    track_stat! {
                        context.stats.main_see_stats.pruned_moves += 1;
                        context.stats.main_see_stats.pruned_moves_by_depth.bins[depth_bucket] += 1;
                    }
                    continue;
                }
            }

            let undo = board.make_move(*mv);

            let child_hash = board.board_hash();
            context.repetition_history.push(child_hash); // only store if valid move

            let can_lmr = can_reduce
                && self.config.search.lmr.enabled
                && depth >= self.config.search.lmr.min_depth
                && searched_moves >= self.config.search.lmr.min_move_number;

            let reduction = if can_lmr {
                self.lmr_table.calculate_reduction(
                    context,
                    depth as usize,
                    searched_moves as usize,
                    is_quiet,
                    is_capture,
                    is_improving_2_plies,
                    NODE::PV,
                    cut_node,
                    history_score,
                    captured_piece,
                    gives_check,
                    &self.config.search.lmr,
                )
            } else {
                0
            };

            let reduced_child_depth = full_child_depth.saturating_sub(reduction as u16);

            let mut eval: i32;

            if searched_moves == 0 && NODE::PV {
                eval = -self.negamax::<PV>(
                    board,
                    context,
                    full_child_depth,
                    -beta,
                    -alpha,
                    false,
                    ply + 1,
                    picker_frame.child(),
                    normal_child_options,
                );

                if context.stopped {
                    context.repetition_history.pop();
                    board.undo_move(undo);
                    return Score::ZERO;
                }
            } else {
                if can_lmr {
                    track_stat!(context.stats.lmr_stats.attempts += 1);
                    track_stat!(context.stats.lmr_stats.eligible_moves += 1);
                    track_stat!(context.stats.lmr_stats.reduction_plies += reduction as u64);

                    #[cfg(feature = "search-stats")]
                    let depth_bucket = usize::from(depth).min(MAX_TRACKED_DEPTH - 1);

                    track_stat!(context.stats.lmr_stats.attempts_by_depth.bins[depth_bucket] += 1);

                    #[cfg(feature = "search-stats")]
                    let reduction_bucket = (reduction as usize).min(REDUCTION_BUCKETS - 1);

                    track_stat!(
                        context.stats.lmr_stats.reduction_histogram.bins[reduction_bucket] += 1
                    );

                    track_stat! {
                        if reduction > 0 {
                            context.stats.lmr_stats.reduced_moves += 1;
                        } else {
                            context.stats.lmr_stats.zero_reduction_moves += 1;
                        }
                    }
                }

                let passed_cut_node = if reduction > 0 { true } else { false };

                // null window search reduced depth
                track_stat!(context.stats.pvs_stats.null_window_searches += 1);
                eval = -self.negamax::<NonPV>(
                    board,
                    context,
                    reduced_child_depth,
                    -alpha - 1,
                    -alpha,
                    passed_cut_node,
                    ply + 1,
                    picker_frame.child(),
                    normal_child_options,
                );

                if context.stopped {
                    context.repetition_history.pop();
                    board.undo_move(undo);
                    return Score::ZERO;
                }

                if eval > alpha {
                    let mut research_depth = full_child_depth;

                    track_stat!(context.stats.pvs_stats.alpha_improvements += 1);

                    if reduction > 0 {
                        #[cfg(feature = "search-stats")]
                        let depth_bucket = usize::from(depth).min(MAX_TRACKED_DEPTH - 1);

                        track_stat!(context.stats.lmr_stats.researches += 1);
                        track_stat!(
                            context.stats.lmr_stats.researches_by_depth.bins[depth_bucket] += 1
                        );

                        research_depth = calculate_lmr_research_depth(
                            depth,
                            full_child_depth,
                            reduced_child_depth,
                            eval,
                            max_eval,
                        );

                        track_stat! { match research_depth.cmp(&full_child_depth) {
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
                        }

                        // null window search full depth
                        if research_depth > reduced_child_depth {
                            track_stat! {
                                context.stats.lmr_stats.dynamic_searches += 1;
                                if research_depth != full_child_depth {
                                        context.stats.lmr_stats.dynamic_searches_at_modified_depth += 1;
                                }
                            }
                            eval = -self.negamax::<NonPV>(
                                board,
                                context,
                                research_depth,
                                -alpha - 1,
                                -alpha,
                                !cut_node,
                                ply + 1,
                                picker_frame.child(),
                                normal_child_options,
                            );
                        }

                        if context.stopped {
                            context.repetition_history.pop();
                            board.undo_move(undo);
                            return Score::ZERO;
                        }

                        track_stat! {
                            if eval > alpha {
                                context.stats.lmr_stats.research_alpha_improvements += 1;
                            }
                            if eval >= beta {
                                context.stats.lmr_stats.research_cutoffs += 1;
                            }
                        }
                    }

                    if NODE::PV && eval > alpha && eval < beta {
                        // this move might improve alpha, research it at full depth
                        // full window search, full depth
                        track_stat! {
                            context.stats.pvs_stats.full_window_researches += 1;
                            if reduction > 0 {
                                context.stats.lmr_stats.dynamic_searches += 1;
                                if research_depth != full_child_depth {
                                    context.stats.lmr_stats.dynamic_searches_at_modified_depth += 1;
                                }
                            }
                        }
                        eval = -self.negamax::<PV>(
                            board,
                            context,
                            research_depth,
                            -beta,
                            -alpha,
                            false,
                            ply + 1,
                            picker_frame.child(),
                            normal_child_options,
                        );

                        if context.stopped {
                            context.repetition_history.pop();
                            board.undo_move(undo);
                            return Score::ZERO;
                        }

                        track_stat! {
                            if eval >= beta {
                                context.stats.pvs_stats.research_cutoffs += 1;
                            }
                        }
                    }
                }
            }

            searched_moves += 1;

            track_stat!(context.stats.move_stats.main_searched += 1);

            context.repetition_history.pop();
            board.undo_move(undo);

            let mut improved_alpha = false;

            if eval > max_eval {
                max_eval = eval;
                best_move = Some(*mv);

                if NODE::ROOT {
                    context.root_best_move = best_move;
                }
            }

            if alpha < eval {
                alpha = eval;
                improved_alpha = true;
            }

            // alpha = alpha.max(eval);

            if alpha >= beta {
                track_stat! {
                    context.stats.cutoff_stats.beta += 1;

                    let cutoff_index = searched_moves as usize;
                    let cutoff_bucket = cutoff_index.saturating_sub(1).min(MOVE_INDEX_BUCKETS - 1);
                    context
                        .stats
                        .move_ordering_stats
                        .cutoff_move_index_histogram
                        .bins[cutoff_bucket] += 1;

                        context.stats.move_ordering_stats.cutoff_move_index_sum += cutoff_index as u64
                    ;

                        context.stats.move_ordering_stats.cutoff_move_index_max = context
                            .stats
                            .move_ordering_stats
                            .cutoff_move_index_max
                            .max(cutoff_index as u64)
                    ;


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
                }

                did_cutoff = true;

                if options.excluded_move.is_none() {
                    if is_quiet {
                        context.killer_moves.add(ply, *mv);
                        #[cfg(feature = "search-stats")]
                        self.history
                            .add_quiet_bonus_with_stats(curr_key, context, ply, depth);
                        #[cfg(not(feature = "search-stats"))]
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

                            #[cfg(feature = "search-stats")]
                            self.history.add_quiet_malus_with_stats(
                                history_key,
                                context,
                                ply,
                                depth,
                            );
                            #[cfg(not(feature = "search-stats"))]
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
                track_stat!(context.stats.singular_stats.no_alternatives += 1);

                // This is the lower edge of the null window:
                // singular_beta - 1.
                return alpha;
            }

            if in_check {
                track_stat!(context.stats.terminal_stats.checkmates += 1);
                return mated_in(ply);
            }

            track_stat!(context.stats.terminal_stats.stalemates += 1);
            return Score::ZERO;
        }

        // update correction history

        let can_update_correction = self.config.search.correction.enabled
            && !in_check
            && options.excluded_move.is_none()
            && !is_decisive(alpha)
            && !is_decisive(beta)
            && depth < 3;

        if can_update_correction {
            track_stat!(context.stats.correction_stats.record_update_opportunity());

            if max_eval > original_alpha && max_eval < beta {
                // Exact
                let delta = max_eval - corrected_static_eval;
                self.history.correction.update(board, delta, depth);
                track_stat!(
                    context
                        .stats
                        .correction_stats
                        .record_update(CorrectionUpdateKind::Exact, delta)
                );
            } else if max_eval >= beta && beta > corrected_static_eval {
                // Fail High
                let delta = beta - corrected_static_eval;
                self.history.correction.update(board, delta, depth);
                track_stat!(
                    context
                        .stats
                        .correction_stats
                        .record_update(CorrectionUpdateKind::FailHigh, delta)
                );
            } else if max_eval <= original_alpha && original_alpha < corrected_static_eval {
                // Fail Low
                let delta = original_alpha - corrected_static_eval;
                self.history.correction.update(board, delta, depth);
                track_stat!(
                    context
                        .stats
                        .correction_stats
                        .record_update(CorrectionUpdateKind::FailLow, delta)
                );
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
            track_stat!(context.stats.tt_stats.main.stores += 1);
            let adjusted_depth = u8::try_from(depth).unwrap_or(u8::MAX);
            let _insert_result = self.tt.insert(
                board.board_hash,
                NODE::PV,
                flag,
                adjusted_depth,
                score_to_tt(max_eval, ply),
                raw_static_eval as i16,
                best_move,
            );
            track_stat!(context.stats.tt_stats.main.record_insert(_insert_result));
        }

        max_eval
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bitboard::{H8, square};
    use crate::engine::SearchLimits;

    use crate::engine::configs::EngineConfig;

    fn test_engine() -> Engine {
        let mut config = EngineConfig::standard();
        config.tt_size = 1;
        Engine::new(config)
    }

    #[test]
    fn missing_tt_search_scores_cannot_cut_off_main_or_quiescence() {
        for quiescence in [false, true] {
            for flag in [TTFlag::Exact, TTFlag::LowerBound, TTFlag::UpperBound] {
                let mut engine = test_engine();
                let mut board = Board::from_fen("7k/6Q1/6K1/8/8/8/8/8 b - - 0 1").unwrap();
                engine.tt.insert(
                    board.board_hash(),
                    false,
                    flag,
                    10,
                    Score::NONE as i16,
                    Score::NONE as i16,
                    None,
                );
                let mut context =
                    SearchContext::new(SearchLimits::depth(1, 0), vec![board.board_hash()]);
                let score = if quiescence {
                    engine.quiescence::<NonPV>(
                        &mut board,
                        &mut context,
                        0,
                        -1,
                        0,
                        0,
                        PickerFrame::ROOT,
                        0,
                    )
                } else {
                    engine.negamax::<NonPV>(
                        &mut board,
                        &mut context,
                        1,
                        -1,
                        0,
                        false,
                        0,
                        PickerFrame::ROOT,
                        SearchOptions::NORMAL,
                    )
                };
                assert_eq!(score, mated_in(0), "qsearch={quiescence}, flag={flag:?}");
            }
        }
    }

    #[test]
    fn losing_mate_does_not_allow_futility_to_hide_an_escape() {
        let mut engine = test_engine();
        engine.config.search.fut.enabled = true;
        engine.config.search.fut.max_depth = 2;
        engine.config.search.lmp.enabled = false;
        engine.config.search.lmr.enabled = false;
        engine.config.search.correction.enabled = false;
        let mut board = Board::from_fen(crate::board::STARTPOS_FEN).unwrap();
        let hash = board.board_hash();
        let moves = board.all_legal_moves();
        let first_move = moves.iter().next().copied().unwrap();
        // Force a losing first move, then fail-low escapes. A low cached static
        // eval would futility-prune every escape while alpha remains at 100.
        for &mv in moves.iter() {
            let undo = board.make_move(mv);
            let child_score = if mv == first_move { Score::MATE - 3 } else { 0 };
            engine.tt.insert(
                board.board_hash(),
                false,
                TTFlag::Exact,
                1,
                score_to_tt(child_score, 1),
                Score::NONE as i16,
                None,
            );
            board.undo_move(undo);
        }
        engine.tt.insert(
            hash,
            false,
            TTFlag::UpperBound,
            1,
            -100,
            -5000,
            Some(first_move),
        );
        let mut context = SearchContext::new(SearchLimits::depth(2, 0), vec![hash]);
        let score = engine.negamax::<NonPV>(
            &mut board,
            &mut context,
            2,
            100,
            101,
            false,
            0,
            PickerFrame::ROOT,
            SearchOptions::NORMAL,
        );
        assert_eq!(score, 0, "a legal escape must replace the losing mate");
        assert_eq!(board.board_hash(), hash);
        assert_eq!(context.repetition_history, vec![hash]);
    }

    #[test]
    fn first_capture_is_searched_even_when_see_would_prune_it() {
        let mut engine = test_engine();
        engine.config.search.see.enabled = true;
        engine.config.search.see.margin = -10_000;
        let mut board = Board::from_fen("7k/8/8/8/8/4p3/3Q4/4K3 w - - 0 1").unwrap();
        let hash = board.board_hash();
        let moves = board.all_legal_moves();
        // Exact child entries isolate main-search pruning from qsearch.
        for &mv in moves.iter() {
            let undo = board.make_move(mv);
            engine.tt.insert(
                board.board_hash(),
                false,
                TTFlag::Exact,
                1,
                0,
                Score::NONE as i16,
                None,
            );
            board.undo_move(undo);
        }
        engine.config.search.fut.enabled = false;
        engine.config.search.lmp.enabled = false;
        engine.config.search.lmr.enabled = false;
        let mut context = SearchContext::new(SearchLimits::depth(2, 0), vec![hash]);
        let score = engine.negamax::<NonPV>(
            &mut board,
            &mut context,
            2,
            1000,
            1001,
            false,
            0,
            PickerFrame::ROOT,
            SearchOptions::NORMAL,
        );
        assert_eq!(score, 0);
        assert_eq!(
            context.stats.node_stats.main,
            1 + moves.iter().count() as u64
        );
        assert_eq!(context.stats.main_see_stats.pruned_moves, 0);
    }

    #[test]
    fn main_see_pruning_has_dedicated_depth_counters() {
        let mut engine = test_engine();
        engine.config.search.see.enabled = true;
        // Force eligible captures below the threshold to exercise pruning.
        engine.config.search.see.margin = -10_000;
        // Two captures: the first must be searched, and the later capture can
        // exercise SEE pruning after a non-losing score has been established.
        let mut board = Board::from_fen("7k/8/8/3p4/8/4p3/3Q4/4K3 w - - 0 1").unwrap();
        let mut context = SearchContext::new(SearchLimits::depth(1, 0), vec![board.board_hash()]);
        engine.negamax::<NonPV>(
            &mut board,
            &mut context,
            1,
            -2000,
            2000,
            false,
            0,
            PickerFrame::ROOT,
            SearchOptions::NORMAL,
        );
        let stats = context.stats.main_see_stats;
        #[cfg(feature = "search-stats")]
        {
            assert!(stats.attempts > 0);
            assert_eq!(stats.attempts, stats.pruned_moves);
            assert_eq!(stats.attempts_by_depth.bins[1], stats.attempts);
            assert_eq!(stats.pruned_moves_by_depth.bins[1], stats.pruned_moves);
            // Qsearch stops at depth zero before any SEE pruning.
            assert_eq!(context.stats.q_pruning_stats.see_attempts, 0);
        }
        #[cfg(not(feature = "search-stats"))]
        assert_eq!(stats.attempts, 0);

        let mut combined = crate::engine::search_stats::SearchStats::default();
        combined += context.stats;
        combined += context.stats;
        let delta = combined - context.stats;
        assert_eq!(delta.main_see_stats.attempts, stats.attempts);
        assert_eq!(delta.main_see_stats.pruned_moves, stats.pruned_moves);
        assert_eq!(
            delta.main_see_stats.attempts_by_depth,
            stats.attempts_by_depth
        );
        assert_eq!(
            delta.main_see_stats.pruned_moves_by_depth,
            stats.pruned_moves_by_depth
        );
    }

    #[test]
    fn records_cut_flag_and_improvement_at_the_improving_check() {
        for improvement in [-40, 0, 25] {
            let mut engine = test_engine();
            // A stalemate reaches the improving check without searching children.
            let mut board = Board::from_fen("7k/5Q2/6K1/8/8/8/8/8 b - - 0 1").unwrap();
            let mut context =
                SearchContext::new(SearchLimits::depth(1, 1), vec![board.board_hash()]);
            context.stack[0].static_eval = evaluation_for_turn(&board) - improvement;
            engine.negamax::<NonPV>(
                &mut board,
                &mut context,
                1,
                -1,
                0,
                true,
                2,
                PickerFrame::ROOT,
                SearchOptions::NORMAL,
            );

            assert_eq!(context.stats.node_stats.main, 1);
            assert_eq!(
                context.stats.node_stats.cut,
                u64::from(cfg!(feature = "search-stats"))
            );
            let stats = context.stats.improvement_stats;
            #[cfg(feature = "search-stats")]
            {
                assert_eq!(stats.improving, u64::from(improvement > 0));
                assert_eq!(stats.non_improving, u64::from(improvement <= 0));
                assert_eq!(stats.average_improvement(), Some(f64::from(improvement)));
                assert_eq!(stats.highest_improvement, Some(improvement.unsigned_abs()));
                assert_eq!(stats.lowest_improvement, Some(improvement));
            }
            #[cfg(not(feature = "search-stats"))]
            assert_eq!(stats.samples(), 0);
        }
    }

    #[test]
    fn staged_negamax_detects_checkmate_and_stalemate() {
        let cases = [
            ("7k/6Q1/6K1/8/8/8/8/8 b - - 0 1", -Score::MATE),
            ("7k/5Q2/6K1/8/8/8/8/8 b - - 0 1", 0),
        ];

        for (fen, expected) in cases {
            let mut engine = test_engine();
            let mut board = Board::from_fen(fen).expect("valid terminal FEN");
            let mut context =
                SearchContext::new(SearchLimits::depth(1, 1), vec![board.board_hash()]);

            let score = engine.negamax::<PV>(
                &mut board,
                &mut context,
                1,
                -Score::INFINITE,
                Score::INFINITE,
                false,
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

        let score = engine.negamax::<NonPV>(
            &mut board,
            &mut context,
            1,
            alpha,
            alpha + 1,
            false,
            0,
            PickerFrame::ROOT,
            SearchOptions::singular_verification(only_legal_move),
        );

        assert_eq!(score, alpha);
        #[cfg(feature = "search-stats")]
        {
            assert_eq!(context.stats.singular_stats.no_alternatives, 1);
            assert_eq!(context.stats.move_stats.main_searched, 0);
            assert_eq!(context.stats.terminal_stats.checkmates, 0);
        }
        #[cfg(not(feature = "search-stats"))]
        assert_eq!(context.stats.singular_stats.no_alternatives, 0);
    }

    fn assert_static_eval_is_available(cached: Option<(u8, i16)>) {
        let mut engine = test_engine();
        let mut board = Board::from_fen("7k/8/8/8/8/8/4Q3/4K3 w - - 0 1").unwrap();
        let computed_eval = evaluation_for_turn(&board);
        assert_ne!(
            computed_eval, 0,
            "the fixture must expose a missing evaluation"
        );
        let expected_eval = match cached {
            Some((_, static_eval)) if static_eval != Score::NONE as i16 => i32::from(static_eval),
            _ => computed_eval,
        };
        if let Some((depth, static_eval)) = cached {
            engine.tt.insert(
                board.board_hash(),
                false,
                TTFlag::LowerBound,
                depth,
                -20_000,
                static_eval,
                None,
            );
        }
        let mut context = SearchContext::new(SearchLimits::depth(1, 1), vec![board.board_hash()]);
        engine.negamax::<PV>(
            &mut board,
            &mut context,
            1,
            -Score::INFINITE,
            Score::INFINITE,
            false,
            0,
            PickerFrame::ROOT,
            SearchOptions::NORMAL,
        );

        // Static evaluation is valid regardless of the stored search depth.
        // It must feed the search stack and survive the subsequent TT store.
        assert_eq!(
            (
                context.stack[0].static_eval,
                i32::from(
                    engine
                        .tt
                        .get(board.board_hash(), TTNodeType::Main)
                        .unwrap()
                        .static_eval
                ),
            ),
            (expected_eval, expected_eval),
            "cached entry: {cached:?}",
        );
    }

    #[test]
    fn tt_miss_computes_and_stores_static_evaluation() {
        assert_static_eval_is_available(None);
    }

    #[test]
    fn shallow_tt_hit_reuses_cached_static_evaluation() {
        assert_static_eval_is_available(Some((0, 321)));
    }

    #[test]
    fn shallow_tt_hit_with_missing_static_evaluation_recomputes_it() {
        assert_static_eval_is_available(Some((0, Score::NONE as i16)));
    }

    #[test]
    fn usable_tt_bound_reuses_cached_static_evaluation() {
        assert_static_eval_is_available(Some((1, 321)));
    }

    #[test]
    fn quiescence_entry_cannot_supply_a_main_search_score() {
        let mut engine = test_engine();
        let mut board = Board::from_fen("7k/6Q1/6K1/8/8/8/8/8 b - - 0 1").unwrap();
        engine.tt.insert(
            board.board_hash(),
            false,
            TTFlag::Exact,
            0,
            123,
            Score::NONE as i16,
            None,
        );
        let mut context = SearchContext::new(SearchLimits::depth(1, 1), vec![board.board_hash()]);

        let score = engine.negamax::<PV>(
            &mut board,
            &mut context,
            1,
            -Score::INFINITE,
            Score::INFINITE,
            false,
            0,
            PickerFrame::ROOT,
            SearchOptions::NORMAL,
        );

        assert_eq!(score, -Score::MATE);
        assert_eq!(context.stats.tt_stats.main.hits, 0);
        assert_eq!(context.stats.tt_stats.main.depth_rejected_hits, 0);
        assert_eq!(context.stats.tt_stats.main.exact_returns, 0);
    }

    #[test]
    fn singular_eligibility_does_not_wrap_requested_depth() {
        let mut engine = test_engine();
        let mut board = Board::from_fen("7k/6Q1/4K3/8/8/8/8/8 b - - 0 1").unwrap();
        let only_legal_move = Move::new(H8, square(6, 6), MoveType::Capture, None);
        engine.tt.insert(
            board.board_hash(),
            false,
            TTFlag::LowerBound,
            1,
            -123,
            Score::NONE as i16,
            Some(only_legal_move),
        );
        let mut context = SearchContext::new(SearchLimits::depth(258, 1), vec![board.board_hash()]);
        let score = engine.negamax::<PV>(
            &mut board,
            &mut context,
            258,
            -Score::INFINITE,
            Score::INFINITE,
            false,
            0,
            PickerFrame::ROOT,
            SearchOptions::NORMAL,
        );
        assert_eq!(score, 0);
        assert_eq!(context.stats.singular_stats.attempts, 0);
        assert_eq!(
            engine
                .tt
                .get(board.board_hash(), TTNodeType::Main)
                .unwrap()
                .depth,
            255
        );
    }

    #[test]
    fn requested_depth_256_rejects_a_255_depth_entry() {
        let mut engine = test_engine();
        let mut board = Board::from_fen("7k/6Q1/6K1/8/8/8/8/8 b - - 0 1").unwrap();
        engine.tt.insert(
            board.board_hash(),
            false,
            TTFlag::Exact,
            255,
            123,
            Score::NONE as i16,
            None,
        );
        let mut context = SearchContext::new(SearchLimits::depth(256, 1), vec![board.board_hash()]);

        let score = engine.negamax::<PV>(
            &mut board,
            &mut context,
            256,
            -Score::INFINITE,
            Score::INFINITE,
            false,
            0,
            PickerFrame::ROOT,
            SearchOptions::NORMAL,
        );

        // Saturating a requested depth still overstates the cached result's
        // coverage. Widen the stored depth for comparison instead.
        assert_eq!(score, -Score::MATE);
        assert_eq!(context.stats.tt_stats.main.exact_returns, 0);
        #[cfg(feature = "search-stats")]
        assert_eq!(context.stats.tt_stats.main.depth_rejected_hits, 1);
        #[cfg(not(feature = "search-stats"))]
        assert_eq!(context.stats.tt_stats.main.depth_rejected_hits, 0);
    }

    #[test]
    fn requested_depth_above_u8_range_does_not_wrap_to_quiescence_depth() {
        // Regression: `depth as u8` turns 256 into zero, accepting a qsearch
        // exact score as a 256-ply result. A terminal board keeps this bounded.
        let mut engine = test_engine();
        let mut board = Board::from_fen("7k/6Q1/6K1/8/8/8/8/8 b - - 0 1").unwrap();
        engine.tt.insert(
            board.board_hash(),
            false,
            TTFlag::Exact,
            0,
            123,
            Score::NONE as i16,
            None,
        );
        let depth = u16::from(u8::MAX) + 1;
        let mut context =
            SearchContext::new(SearchLimits::depth(depth, 1), vec![board.board_hash()]);

        let score = engine.negamax::<PV>(
            &mut board,
            &mut context,
            depth,
            -Score::INFINITE,
            Score::INFINITE,
            false,
            0,
            PickerFrame::ROOT,
            SearchOptions::NORMAL,
        );

        assert_eq!(score, -Score::MATE);
        assert_eq!(context.stats.tt_stats.main.exact_returns, 0);
        assert_eq!(context.stats.tt_stats.main.depth_rejected_hits, 0);
        assert_eq!(context.stats.tt_stats.main.hits, 0);
    }
}
