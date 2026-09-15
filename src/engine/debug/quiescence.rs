use crate::board::{Board, Move, MoveType};
use crate::engine::config::{CHECKMATE_SCORE, MATE_THRESHOLD, NEG_INF};
use crate::engine::ordering::see;
use crate::engine::search::is_insufficient_material;
use crate::engine::tt::{TTEntry, TTFlag, TTNodeType, score_from_tt, score_to_tt};
use crate::engine::{Engine, MAX_PLY, PickerFrame, SearchContext};
use crate::eval::evaluation_for_turn;
use crate::moves::MoveGenInfo;
use crate::types::PieceType;

const MAX_CHECK_Q_PLIES: usize = 64;

pub(super) fn quiescence(
    engine: &mut Engine,
    board: &mut Board,
    context: &mut SearchContext,
    depth: u16,
    mut alpha: i32,
    mut beta: i32,
    ply: usize,
    picker_frame: PickerFrame,
    check_plies: usize,
) -> i32 {
    if context.stopped || (context.stats.total_nodes() & 2047 == 0 && context.should_stop()) {
        return 0;
    }

    context.stats.node_stats.quiescence += 1;

    if Engine::repetition_in_search(context, board.board_hash(), board.halfmove_clock() as usize) {
        return 0;
    }
    if board.halfmove_clock() >= 100 {
        return 0;
    }

    if board.phase() < 8 && is_insufficient_material(board) {
        return 0;
    }

    if ply >= MAX_PLY - 1 {
        return evaluation_for_turn(board);
    }

    let in_check = board.in_check(board.side_to_move());
    let original_alpha = alpha;
    let original_beta = beta;
    let hash = board.board_hash();
    let side_to_move = board.side_to_move();
    let mut tt_best_move: Option<Move> = None;

    if let Some(entry) = engine.tt.get(hash, TTNodeType::Quiescence) {
        tt_best_move = entry.best_move;
        let tt_score = score_from_tt(entry.eval, ply);

        if entry.depth >= depth {
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
    }

    let mut best_eval = NEG_INF;
    let mut stand_pat = NEG_INF;
    let mut best_move: Option<Move> = None;

    let mut move_picker = if in_check {
        engine.new_move_picker(
            0,
            context,
            side_to_move,
            ply,
            picker_frame,
            None,
            tt_best_move,
        )
    } else {
        engine.new_move_picker(
            1,
            context,
            side_to_move,
            ply,
            picker_frame,
            None,
            tt_best_move,
        )
    };

    let info = MoveGenInfo::calculate(board, side_to_move);
    let mut pending_move = move_picker.get_next(board, &info, context, &engine.history);

    if in_check {
        if pending_move.is_none() {
            let score = -CHECKMATE_SCORE + ply as i32;
            engine.tt.insert(
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

        if check_plies > MAX_CHECK_Q_PLIES {
            let fallback = 0.clamp(alpha, beta - 1);
            return fallback;
        }
    } else {
        if pending_move.is_none() {
            let mut legal_move_picker = engine.new_move_picker(
                0,
                context,
                side_to_move,
                ply,
                picker_frame.child(),
                None,
                None,
            );

            if legal_move_picker
                .get_next(board, &info, context, &engine.history)
                .is_none()
            {
                let score = 0;
                engine.tt.insert(
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
        }

        stand_pat = evaluation_for_turn(board);
        best_eval = stand_pat;

        if stand_pat >= beta {
            engine.tt.insert(
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
    }

    loop {
        let scored_mv = if let Some(scored_mv) = pending_move.take() {
            scored_mv
        } else if let Some(scored_mv) = move_picker.get_next(board, &info, context, &engine.history)
        {
            scored_mv
        } else {
            break;
        };

        let mv = &scored_mv.mv;
        let captured_value = match mv.kind() {
            MoveType::EnPassant => PieceType::Pawn.value(),
            _ => board.piece_at(mv.to()).map(|p| p.kind.value()).unwrap_or(0),
        };

        let gives_check = board.move_gives_check(mv);
        let can_prune = !in_check
            && board.phase > 8
            && mv.promotion().is_none()
            && alpha.abs() < MATE_THRESHOLD
            && Some(*mv) != tt_best_move
            && !gives_check;

        if can_prune {
            if engine.config.search.delta.enabled
                && stand_pat + captured_value + engine.config.search.delta.margin < alpha
            {
                continue;
            }

            if engine.config.search.see.enabled {
                let see_value = if let Some(value) = scored_mv.see {
                    value
                } else {
                    see(board, *mv, scored_mv.captured)
                };
                if see_value < -engine.config.search.see.margin {
                    continue;
                }
            }
        }

        let undo = board.make_move(*mv);
        let child_hash = board.board_hash();
        context.repetition_history.push(child_hash);
        let check_plies = if gives_check { check_plies + 1 } else { 0 };

        let eval = -quiescence(
            engine,
            board,
            context,
            depth,
            -beta,
            -alpha,
            ply + 1,
            picker_frame.child(),
            check_plies,
        );

        if context.stopped {
            context.repetition_history.pop();
            board.undo_move(undo);
            return 0;
        }

        context.repetition_history.pop();
        board.undo_move(undo);

        if eval > best_eval {
            best_eval = eval;
            best_move = Some(*mv);
        }

        alpha = alpha.max(eval);

        if eval >= beta {
            engine.tt.insert(
                hash,
                TTEntry {
                    depth,
                    eval: score_to_tt(eval, ply),
                    best_move,
                    flag: TTFlag::LowerBound,
                    node_type: TTNodeType::Quiescence,
                },
            );
            return eval;
        }
    }

    let flag = if best_eval <= original_alpha {
        TTFlag::UpperBound
    } else if best_eval >= original_beta {
        TTFlag::LowerBound
    } else {
        TTFlag::Exact
    };

    engine.tt.insert(
        hash,
        TTEntry {
            depth,
            eval: score_to_tt(best_eval, ply),
            best_move,
            flag,
            node_type: TTNodeType::Quiescence,
        },
    );

    best_eval
}
