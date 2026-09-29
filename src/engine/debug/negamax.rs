use crate::board::{Board, Move, MoveType, null_move_reduction};
use crate::engine::config::{CHECKMATE_SCORE, NEG_INF};
use crate::engine::debug::quiescence::quiescence;
use crate::engine::history::{HistoryKey, HistoryTables};
use crate::engine::pruning::lmr::LMR_CAPTURE_VALUE;
use crate::engine::pruning::{LMR_SCALE_I32, lmp_history_limit, lmp_threshold};
use crate::engine::search::is_insufficient_material;
use crate::engine::search::iterative::{MAX_CAPTURES, MAX_QUIETS};
use crate::engine::staged::ScoredMove;
use crate::engine::tt::{TTEntry, TTFlag, TTNodeType, score_from_tt, score_to_tt};
use crate::engine::{
    Engine, MATE_THRESHOLD, MAX_PLY, PickerFrame, SearchContext, SearchOptions, SearchStackEntry,
};
use crate::eval::evaluation_for_turn;
use crate::moves::MoveGenInfo;
use crate::types::PieceType;

pub(super) fn negamax(
    engine: &mut Engine,
    board: &mut Board,
    context: &mut SearchContext,
    depth: u16,
    mut alpha: i32,
    mut beta: i32,
    ply: usize,
    picker_frame: PickerFrame,
    options: SearchOptions,
) -> i32 {
    if context.stopped || (context.stats.total_nodes() & 2047 == 0 && context.should_stop()) {
        return 0;
    }

    context.stats.node_stats.main += 1;

    let is_pv = beta != alpha + 1;

    if ply >= MAX_PLY - 1 {
        return evaluation_for_turn(board);
    }

    if depth == 0 {
        return quiescence(
            engine,
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

    if Engine::repetition_in_search(context, board.board_hash(), board.halfmove_clock() as usize) {
        return 0;
    }

    if board.halfmove_clock() >= 100 {
        return 0;
    }

    if board.phase() < 8 && is_insufficient_material(board) {
        return 0;
    }

    let original_alpha = alpha;
    let in_check = board.in_check(board.side_to_move());
    let mut tt_best_move: Option<Move> = None;
    let mut candidate_move: Option<(Move, i32)> = None;

    if let Some(entry) = engine.tt.get(board.board_hash, TTNodeType::Main) {
        let tt_score = score_from_tt(entry.eval, ply);
        tt_best_move = entry.best_move;

        if entry.depth >= depth && options.excluded_move.is_none() {
            match entry.flag {
                TTFlag::Exact => return tt_score,
                TTFlag::LowerBound => {
                    alpha = alpha.max(tt_score);
                }
                TTFlag::UpperBound => {
                    beta = beta.min(tt_score);
                }
            }
            if alpha >= beta {
                return tt_score;
            }
        }

        let suitable_for_singular = options.excluded_move.is_none()
            && engine.config.search.singular.enabled
            && options.allow_singular
            && depth >= engine.config.search.singular.minimum_depth
            && entry.depth >= depth.saturating_sub(2)
            && matches!(entry.flag, TTFlag::Exact | TTFlag::LowerBound)
            && tt_score.abs() < MATE_THRESHOLD;

        if suitable_for_singular && let Some(mv) = entry.best_move {
            candidate_move = Some((mv, tt_score));
        }
    }

    let raw_static_eval = evaluation_for_turn(board);
    let corrected_static_eval = if engine.config.search.correction.enabled {
        raw_static_eval + engine.history.correction.get(board)
    } else {
        raw_static_eval
    };
    let can_rfp = !in_check
        && engine.config.search.rfp.enabled
        && options.excluded_move.is_none()
        && !is_pv
        && beta < MATE_THRESHOLD
        && alpha > -MATE_THRESHOLD
        && ply > 0
        && board.phase() >= engine.config.search.rfp.min_phase
        && depth <= engine.config.search.rfp.max_depth;

    if can_rfp {
        let margin = engine.config.search.rfp.margin_factor * depth as i32;
        if corrected_static_eval - margin >= beta {
            return beta;
        }
    }

    let mut can_null_prune = engine.config.search.null_move.enabled
        && options.allow_null_move
        && options.excluded_move.is_none()
        && !in_check
        && depth >= engine.config.search.null_move.minimum_depth
        && board.phase >= engine.config.search.null_move.minimum_phase
        && beta.abs() < MATE_THRESHOLD
        && !is_pv;

    if can_null_prune {
        if corrected_static_eval < beta {
            can_null_prune = false;
        }
    }

    if can_null_prune {
        let reduction = null_move_reduction(depth);
        let undo = board.make_null_move();
        let mut search_options = options;
        search_options.allow_null_move = false;

        let score = -negamax(
            engine,
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

    let mut move_picker = engine.new_move_picker(
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

    let can_fut = engine.config.search.fut.enabled
        && depth <= engine.config.search.fut.max_depth
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
            Some(previous_eval) => raw_static_eval > previous_eval,
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

    while let Some(scored_mv) = move_picker.get_next(board, &info, context, &engine.history) {
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
            }
        } else {
            None
        };

        let history_score = if is_quiet {
            engine
                .history
                .get_quiet_score(&context.stack, ply, curr_key)
        } else if is_capture && Some(*mv) != tt_best_move {
            engine
                .history
                .get_capture_score(curr_key, captured_piece.unwrap())
        } else {
            0
        };

        let mut extension: u16 = 0;

        if let Some(candidate) = candidate_move {
            if candidate.0 == *mv {
                let margin = engine.config.search.singular.base_margin
                    + engine.config.search.singular.depth_margin * depth as i32;
                let singular_beta = candidate.1 - margin;
                let verification_depth = depth.saturating_sub(1) / 2;

                let verification_score = negamax(
                    engine,
                    board,
                    context,
                    verification_depth,
                    singular_beta - 1,
                    singular_beta,
                    ply,
                    picker_frame.child(),
                    SearchOptions {
                        allow_null_move: false,
                        allow_singular: false,
                        excluded_move: Some(candidate.0),
                    },
                );

                if context.stopped {
                    return 0;
                }

                if verification_score < singular_beta {
                    extension = 1;
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

        let can_reduce = (is_quiet || is_capture)
            && !in_check
            && Some(*mv) != tt_best_move
            && extension == 0
            && options.excluded_move.is_none();

        let can_prune =
            can_reduce && is_quiet && !gives_check && !is_pv && alpha.abs() < MATE_THRESHOLD;

        let can_lmp = can_prune
            && engine.config.search.lmp.enabled
            && depth <= engine.config.search.lmp.max_depth;

        if can_lmp
            && quiet_moves_seen > lmp_threshold(depth as usize)
            && history_score < lmp_history_limit(depth as usize)
        {
            continue;
        }

        if can_fut && can_prune && searched_moves > 0 {
            let mut margin = depth * engine.config.search.fut.margin;

            if engine.config.search.fut.history_enabled {
                if history_score > engine.config.search.fut.good_history {
                    margin += depth * engine.config.search.fut.history_margin;
                } else if history_score < engine.config.search.fut.bad_history {
                    margin -= depth * engine.config.search.fut.history_margin;
                }
            }

            if corrected_static_eval + margin as i32 <= alpha {
                continue;
            }
        }

        let undo = board.make_move(*mv);
        let child_hash = board.board_hash();
        context.repetition_history.push(child_hash);

        let can_lmr =
            can_reduce && engine.config.search.lmr.enabled && depth >= 3 && searched_moves >= 3;

        let reduction = if can_lmr {
            let base_reduction = engine
                .lmr_table
                .get(depth as usize, searched_moves as usize + 1);
            let mut current_reduction = base_reduction;

            if engine.config.search.lmr.history_enabled {
                let history_adjustement = if is_quiet {
                    history_score / engine.config.search.lmr.history_scale
                } else if is_capture {
                    // Capture history is less than max quiet history.
                    history_score * 2 / engine.config.search.lmr.history_scale
                } else {
                    0
                };
                current_reduction += -history_adjustement;
            }

            if !is_improving {
                current_reduction += base_reduction / 3;
            }

            if is_pv {
                current_reduction += -(base_reduction / 3);
            }

            if is_capture {
                current_reduction +=
                    -(LMR_CAPTURE_VALUE[captured_piece.unwrap().idx()] + LMR_SCALE_I32 / 2);
            }

            if gives_check {
                current_reduction += -(LMR_SCALE_I32 / 2);
            }

            let reduction_in_plies = |scaled_reduction: i32| {
                (scaled_reduction / LMR_SCALE_I32).clamp(0, depth as i32 - 2)
            };
            let red = reduction_in_plies(current_reduction);

            red
        } else {
            0
        };

        let reduced_child_depth = full_child_depth.saturating_sub(reduction as u16);
        let mut eval: i32;

        if searched_moves == 0 {
            eval = -negamax(
                engine,
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
            eval = -negamax(
                engine,
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
                if reduction > 0 {
                    eval = -negamax(
                        engine,
                        board,
                        context,
                        full_child_depth,
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
                }

                if eval > alpha && eval < beta {
                    eval = -negamax(
                        engine,
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
                }
            }
        }

        searched_moves += 1;
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

        if alpha >= beta {
            did_cutoff = true;

            if options.excluded_move.is_none() {
                if is_quiet {
                    context.killer_moves.add(ply, *mv);
                    add_quiet_bonus(&mut engine.history, curr_key, context, ply, depth);

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

                        add_quiet_malus(&mut engine.history, history_key, context, ply, depth);
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

                    engine
                        .history
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

                    engine
                        .history
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
            return alpha;
        }

        if in_check {
            return -CHECKMATE_SCORE + ply as i32;
        }

        return 0;
    }

    let can_update_correction = engine.config.search.correction.enabled
        && !in_check
        && options.excluded_move.is_none()
        && alpha.abs() < MATE_THRESHOLD
        && beta.abs() < MATE_THRESHOLD
        && depth < 3;

    if can_update_correction {
        if max_eval > original_alpha && max_eval < beta {
            let delta = max_eval - corrected_static_eval;
            engine.history.correction.update(board, delta, depth);
        } else if max_eval >= beta && beta > corrected_static_eval {
            let delta = beta - corrected_static_eval;
            engine.history.correction.update(board, delta, depth);
        } else if max_eval <= original_alpha && original_alpha < corrected_static_eval {
            let delta = original_alpha - corrected_static_eval;
            engine.history.correction.update(board, delta, depth);
        }
    }

    let flag = if max_eval <= original_alpha {
        TTFlag::UpperBound
    } else if did_cutoff {
        TTFlag::LowerBound
    } else {
        TTFlag::Exact
    };

    if options.excluded_move.is_none() {
        engine.tt.insert(
            board.board_hash,
            TTEntry {
                depth,
                eval: score_to_tt(max_eval, ply),
                best_move,
                flag,
                node_type: TTNodeType::Main,
            },
        );
    }

    max_eval
}

fn add_quiet_bonus(
    history: &mut HistoryTables,
    curr_key: HistoryKey,
    context: &SearchContext,
    ply: usize,
    depth: u16,
) {
    history.main.add_bonus(curr_key, depth);

    if let Some(prev_key) = context.stack[ply].history_index {
        history.continuation.add_bonus(1, prev_key, curr_key, depth);
    }

    if ply > 0 {
        if let Some(prev_key) = context.stack[ply - 1].history_index {
            history.continuation.add_bonus(2, prev_key, curr_key, depth);
        }

        if ply > 2
            && let Some(prev_key) = context.stack[ply - 3].history_index
        {
            history.continuation.add_bonus(4, prev_key, curr_key, depth);
        }
    }
}

fn add_quiet_malus(
    history: &mut HistoryTables,
    curr_key: HistoryKey,
    context: &SearchContext,
    ply: usize,
    depth: u16,
) {
    history.main.add_malus(curr_key, depth);

    if let Some(prev_key) = context.stack[ply].history_index {
        history.continuation.add_malus(1, prev_key, curr_key, depth);
    }

    if ply > 0 {
        if let Some(prev_key) = context.stack[ply - 1].history_index {
            history.continuation.add_malus(2, prev_key, curr_key, depth);
        }

        if ply > 2
            && let Some(prev_key) = context.stack[ply - 3].history_index
        {
            history.continuation.add_malus(4, prev_key, curr_key, depth);
        }
    }
}
