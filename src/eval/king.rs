use crate::bitboard::{
    Bitboard, NOT_FILE_A, NOT_FILE_H, Square, bit, file_mask, file_of, pop_lsb, rank_of, square,
};
use crate::board::Board;
use crate::eval::eval::EvalInfo;
use crate::types::{Color, PieceType};

pub fn king_eval(board: &Board, info: &EvalInfo) -> i32 {
    king_eval_raw(board, Color::White, info) - king_eval_raw(board, Color::Black, info)
}

pub fn king_eval_raw(board: &Board, color: Color, info: &EvalInfo) -> i32 {
    let mut score = 0;

    let Some(king_sq) = pop_lsb(&mut board.pieces(color, PieceType::King)) else {
        panic!("No king in board.pieces in king_eval_raw!");
    };

    score += king_ring_safety(board, color, info);
    score += pawn_shield_score(board, color, king_sq, info);
    score += open_file_bonus(board, color, king_sq, info);
    score += open_diagonal_bonus(board, color, king_sq, info);
    score += escape_score_bonus(board, color, info);

    score
}

fn king_ring_safety(_board: &Board, color: Color, info: &EvalInfo) -> i32 {
    let mut score = 0;

    // TODO: Add a king danger table

    let enemy = color.opposite();

    let king_ring = info.king_ring(color);

    let pawn_attacks = (king_ring & info.attacks(enemy, PieceType::Pawn)).count_ones();

    let knight_attacks = (king_ring & info.attacks(enemy, PieceType::Knight)).count_ones();

    let bishop_attacks = (king_ring & info.attacks(enemy, PieceType::Bishop)).count_ones();

    let rook_attacks = (king_ring & info.attacks(enemy, PieceType::Rook)).count_ones();

    let queen_attacks = (king_ring & info.attacks(enemy, PieceType::Queen)).count_ones();

    let num_attacks = pawn_attacks + knight_attacks + bishop_attacks + rook_attacks + queen_attacks;

    let defended_squares = (king_ring & info.attacked_by_two(color)).count_ones();

    let defender_bonus = match defended_squares {
        0 => -20,
        1 => -10,
        2 => 10,
        3 => 30,
        4 => 60,
        5 => 100,
        _ => 150,
    };

    let attack_penalty = match num_attacks {
        0 => -20,
        1 => 5,
        2 => 20,
        3 => 60,
        4 => 120,
        5 => 250,
        6 => 600,
        _ => 1200,
    };

    score -= attack_penalty;

    score += defender_bonus;

    score
}

fn pawn_shield_score(board: &Board, color: Color, king_sq: Square, info: &EvalInfo) -> i32 {
    if info.phase() < 10 {
        return 0; // no pawn shield evaluation in the endgame
    }

    if info.phase() > 20 {
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
    let mut score = 0;

    let pawns = board.pieces(color, PieceType::Pawn);

    let first_row_shield = generate_king_shield(color, king_sq);

    let mut total_pawn_shield = 0;

    let pawn_count = (pawns & first_row_shield).count_ones() as i32;

    total_pawn_shield += pawn_count;

    score += (pawn_count * 10) + ((3 - pawn_count) * -10);

    let second_row_shield = generate_king_shield_two_forward(color, king_sq);

    let second_pawn_count = (pawns & second_row_shield).count_ones() as i32;

    total_pawn_shield += second_pawn_count;

    score += second_pawn_count * 5;

    if total_pawn_shield >= 3 {
        score += 20;
    } else if total_pawn_shield == 0 {
        score -= 30;
    } else {
        score -= 10;
    }
    score
}

fn open_file_bonus(board: &Board, color: Color, king_sq: Square, info: &EvalInfo) -> i32 {
    if info.phase() < 10 {
        return 0; // ignore in endgames
    }

    let mut score = 0;

    let king_file = file_of(king_sq);

    let enemy_sliders = board.pieces(color.opposite(), PieceType::Rook)
        | board.pieces(color.opposite(), PieceType::Queen);

    let friendly_pawns = board.pieces(color, PieceType::Pawn);

    for (offset, penalty) in [(-1, 7), (0, 15), (1, 7)] {
        let file = king_file as i8 + offset;

        if !(0..=7).contains(&file) {
            continue;
        }

        let file_bb = file_mask(file as u8);

        if file_bb & friendly_pawns == 0 {
            score -= penalty;
        }
        if file_bb & enemy_sliders != 0 {
            score -= 10;
        }
    }

    score
}

fn open_diagonal_bonus(board: &Board, color: Color, king_sq: Square, info: &EvalInfo) -> i32 {
    if info.phase() < 10 {
        return 0;
    }

    let mut score = 0;

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
                    score -= 10;
                    break;
                }
            }

            file += df;
            rank += dr;
        }
        if open_diagonal {
            score -= 10;
        }
    }

    score
}

fn escape_score_bonus(board: &Board, color: Color, info: &EvalInfo) -> i32 {
    if info.phase() < 10 {
        return 0;
    }

    let all_attacks = info.all_attacks(color.opposite());

    let friends = board.occupancy_of(color);

    let king_ring = info.king_ring(color); // this includes the king itself

    let escape_squares = (king_ring & all_attacks & !friends).count_ones() as i32;

    if escape_squares == 0 {
        return -30;
    } else if escape_squares <= 2 {
        return -10;
    } else {
        return 15;
    }
}

fn generate_king_shield(color: Color, sq: Square) -> Bitboard {
    let b = bit(sq);

    let mut shield = 0u64;

    match color {
        Color::White => {
            // left shifts for white
            shield |= b << 8;
            shield |= (b & NOT_FILE_H) << 9;
            shield |= (b & NOT_FILE_A) << 7;
        }
        Color::Black => {
            // right shifts for black
            shield |= b >> 8;
            shield |= (b & NOT_FILE_H) >> 7;
            shield |= (b & NOT_FILE_A) >> 9;
        }
    }

    shield
}

fn generate_king_shield_two_forward(color: Color, sq: Square) -> Bitboard {
    let b = bit(sq);

    let mut shield = 0u64;

    match color {
        Color::White => {
            // left shifts for white
            shield |= b << 16;
            shield |= (b & NOT_FILE_H) << 17;
            shield |= (b & NOT_FILE_A) << 15;
        }
        Color::Black => {
            // right shifts for black
            shield |= b >> 16;
            shield |= (b & NOT_FILE_H) >> 15;
            shield |= (b & NOT_FILE_A) >> 17;
        }
    }

    shield
}
