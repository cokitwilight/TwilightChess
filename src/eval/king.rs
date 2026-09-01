use crate::bitboard::{Square, bit, file_mask, file_of, pop_lsb, rank_of, square};
use crate::board::Board;
use crate::eval::EvalInfo;
use crate::eval::eval::MAX_DANGER;
use crate::eval::lookup::{KING_DANGER_TABLE, KING_PAWN_SHIELD, KING_PAWN_TWO_SHIELD};
use crate::eval::scale_by_phase;
use crate::types::{Color, PieceType};

pub fn king_eval(board: &Board, info: &EvalInfo) -> i32 {
    king_eval_danger_raw(board, Color::White, info)
        - king_eval_danger_raw(board, Color::Black, info)
}

pub fn king_eval_danger_raw(board: &Board, color: Color, info: &EvalInfo) -> i32 {
    let mut danger = 0;

    let Some(king_sq) = pop_lsb(&mut board.pieces(color, PieceType::King)) else {
        panic!("No king in board.pieces in king_eval_danger_raw!");
    };

    danger += info.king_attack_weight(color);
    danger += pawn_shield_danger_score(board, color, king_sq, info);
    danger += open_file_danger_bonus(board, color, king_sq, info);
    danger += open_diagonal_danger_bonus(board, color, king_sq, info);
    danger += escape_score_danger_bonus(board, color, info);
    danger += defender_danger_bonus(board, color, info);

    let mut penalty = -KING_DANGER_TABLE[danger.clamp(0, MAX_DANGER as i32) as usize];
    penalty = scale_by_phase(penalty, info.phase(), 4, 12);
    penalty
}

// range: -17 <-> 8
pub(super) fn pawn_shield_danger_score(
    board: &Board,
    color: Color,
    king_sq: Square,
    info: &EvalInfo,
) -> i32 {
    if info.phase() < 10 {
        return 0; // no pawn shield evaluation in the endgame
    }

    if info.phase() > 20 && board.has_castling_rights() {
        match color {
            Color::White => {
                if king_sq == 4 {
                    return 0; // dont give a bonus if king hasn't castled in opening
                }
            }
            Color::Black => {
                if king_sq == 60 {
                    return 0;
                }
            }
        }
    }
    let mut danger = 0;

    let pawns = board.pieces(color, PieceType::Pawn);

    let mut first_row_shield = KING_PAWN_SHIELD[color.idx()][king_sq as usize];

    match color {
        Color::White => {
            if king_sq < 56 {
                let front_mask = bit(king_sq + 8);

                let enemy_pawn = board.pieces(color.opposite(), PieceType::Pawn) & front_mask;

                first_row_shield |= enemy_pawn;
            }
        }
        Color::Black => {
            if king_sq > 7 {
                let front_mask = bit(king_sq - 8);

                let enemy_pawn = board.pieces(color.opposite(), PieceType::Pawn) & front_mask;

                first_row_shield |= enemy_pawn;
            }
        }
    }

    let mut total_pawn_shield = 0;

    let pawn_count = (pawns & first_row_shield).count_ones() as i32;

    total_pawn_shield += pawn_count;

    danger -= pawn_count * 2;

    let second_row_shield = KING_PAWN_TWO_SHIELD[color.idx()][king_sq as usize];

    let second_pawn_count = (pawns & second_row_shield).count_ones() as i32;

    total_pawn_shield += second_pawn_count;

    danger -= second_pawn_count;

    if total_pawn_shield >= 3 {
        danger -= 8;
    } else if total_pawn_shield == 0 {
        danger += 8;
    } else {
        danger += 1;
    }
    danger
}

// range: 0 <-> 25
pub(super) fn open_file_danger_bonus(
    board: &Board,
    color: Color,
    king_sq: Square,
    info: &EvalInfo,
) -> i32 {
    if info.phase() < 10 {
        return 0; // ignore in endgames
    }

    let mut danger = 0;

    let king_file = file_of(king_sq);

    let enemy_sliders = board.pieces(color.opposite(), PieceType::Rook)
        | board.pieces(color.opposite(), PieceType::Queen);

    let friendly_pawns = board.pieces(color, PieceType::Pawn);

    for (offset, penalty) in [(-1, 1), (0, 2), (1, 1)] {
        let file = king_file as i8 + offset;

        if !(0..=7).contains(&file) {
            continue;
        }

        let file_bb = file_mask(file as u8);

        if file_bb & friendly_pawns == 0 {
            danger += penalty;
        }
        if file_bb & enemy_sliders != 0 {
            danger += 7;
        }
    }

    danger
}

// range: 0 <-> 24. Note in just the forward two it would likely be 0-12/14
// REWRITE FUNCTION!
pub(super) fn open_diagonal_danger_bonus(
    board: &Board,
    color: Color,
    king_sq: Square,
    info: &EvalInfo,
) -> i32 {
    if info.phase() < 10 {
        return 0;
    }

    let mut danger = 0;

    let occupied = board.all_occupancy();

    let enemy_sliders = board.pieces(color.opposite(), PieceType::Bishop)
        | board.pieces(color.opposite(), PieceType::Queen);

    for (df, dr) in [(1, 1), (1, -1), (-1, 1), (-1, -1)] {
        let mut file = file_of(king_sq) as i8 + df;
        let mut rank = rank_of(king_sq) as i8 + dr;

        let mut open_diagonal = true;

        while (0..8).contains(&file) && (0..8).contains(&rank) {
            let to = square(file as u8, rank as u8);
            let to_mask = bit(to);

            if occupied & to_mask != 0 {
                open_diagonal = false;
                if to_mask & enemy_sliders != 0 {
                    // break when you meet a bishop/queen else keep checking for potention weakness
                    danger += 7;
                    break;
                }
            }

            file += df;
            rank += dr;
        }
        if open_diagonal {
            danger += 6;
        }
    }

    danger
}

// range: -6 <-> 12
pub(super) fn escape_score_danger_bonus(board: &Board, color: Color, info: &EvalInfo) -> i32 {
    if info.phase() < 10 {
        return 0;
    }

    let all_attacks = info.all_attacks(color.opposite());

    let friends = board.occupancy_of(color);

    let king_ring = info.king_ring(color); // this includes the king itself

    let escape_squares = (king_ring & !all_attacks & !friends).count_ones() as i32;

    if escape_squares == 0 {
        12
    } else if escape_squares <= 1 {
        4
    } else if escape_squares <= 2 {
        1
    } else {
        -6
    }
}

// range: 0 <-> 40.  // reasonably more likely to be at max ~20
pub(super) fn defender_danger_bonus(board: &Board, color: Color, info: &EvalInfo) -> i32 {
    let defended_squares = info.king_ring(color) & info.attacked_by_two(color); // king is included in regular attacks

    let local_defenders =
        info.king_ring(color) & board.occupancy_of(color) & !board.pieces(color, PieceType::Pawn);

    let mut danger = 0;

    // since this represents king danger the bonus is negative
    danger -= defended_squares.count_ones() as i32 * 2;
    danger -= local_defenders.count_ones() as i32 * 3;

    danger
}
