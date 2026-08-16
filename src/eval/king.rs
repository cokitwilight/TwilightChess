use crate::bitboard::{
    Bitboard, NOT_FILE_A, NOT_FILE_H, Square, bit, file_mask, file_of, pop_lsb, rank_of, square,
};
use crate::board::Board;
use crate::eval::eval::{EvalInfo, KING_DANGER_TABLE, MAX_DANGER};
use crate::eval::scale_by_phase;
use crate::types::{Color, PieceType};

static KING_PAWN_SHIELD: [[Bitboard; 64]; 2] = generate_king_shield();
static KING_PAWN_TWO_SHIELD: [[Bitboard; 64]; 2] = generate_king_two_shield();

pub fn king_eval(board: &Board, info: &EvalInfo) -> i32 {
    king_eval_danger_raw(board, Color::White, info)
        - king_eval_danger_raw(board, Color::Black, info)
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

pub fn king_eval_danger_raw(board: &Board, color: Color, info: &EvalInfo) -> i32 {
    let mut danger = 0;

    let Some(king_sq) = pop_lsb(&mut board.pieces(color, PieceType::King)) else {
        panic!("No king in board.pieces in king_eval_raw!");
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

pub fn king_eval_danger_index(board: &Board, color: Color, info: &EvalInfo) -> i8 {
    let mut danger = 0;

    let Some(king_sq) = pop_lsb(&mut board.pieces(color, PieceType::King)) else {
        panic!("No king in board.pieces in king_eval_raw!");
    };

    danger += info.king_attack_weight(color) as i8;
    danger += pawn_shield_danger_score(board, color, king_sq, info) as i8;
    danger += open_file_danger_bonus(board, color, king_sq, info) as i8;
    danger += open_diagonal_danger_bonus(board, color, king_sq, info) as i8;
    danger += escape_score_danger_bonus(board, color, info) as i8;

    danger
}

fn king_ring_safety(_board: &Board, color: Color, info: &EvalInfo) -> i32 {
    let mut score = 0;

    // TODO: Add a king danger table instead of score(probably add this to all functions and then create one function that scores king danger)

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

    // TODO: Add a different score for different pieces(aka a queen attacking alone is useless but queen + bishop + rook is much worse)
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

    score = scale_by_phase(score, info.phase(), 4, 12);

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

    let first_row_shield = KING_PAWN_SHIELD[color.idx()][king_sq as usize];

    let mut total_pawn_shield = 0;

    let pawn_count = (pawns & first_row_shield).count_ones() as i32;

    total_pawn_shield += pawn_count;

    score += (pawn_count * 10) + ((3 - pawn_count) * -10);

    let second_row_shield = KING_PAWN_TWO_SHIELD[color.idx()][king_sq as usize];

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

    score = scale_by_phase(score, info.phase(), 6, 10);

    score
}

fn pawn_shield_danger_score(board: &Board, color: Color, king_sq: Square, info: &EvalInfo) -> i32 {
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

fn open_file_danger_bonus(board: &Board, color: Color, king_sq: Square, info: &EvalInfo) -> i32 {
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

fn open_diagonal_danger_bonus(
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

fn escape_score_bonus(board: &Board, color: Color, info: &EvalInfo) -> i32 {
    if info.phase() < 10 {
        return 0;
    }

    let all_attacks = info.all_attacks(color.opposite());

    let friends = board.occupancy_of(color);

    let king_ring = info.king_ring(color); // this includes the king itself

    let escape_squares = (king_ring & all_attacks & !friends).count_ones() as i32;

    if escape_squares == 0 {
        -30
    } else if escape_squares <= 2 {
        -10
    } else {
        15
    }
}

fn escape_score_danger_bonus(board: &Board, color: Color, info: &EvalInfo) -> i32 {
    if info.phase() < 10 {
        return 0;
    }

    let all_attacks = info.all_attacks(color.opposite());

    let friends = board.occupancy_of(color);

    let king_ring = info.king_ring(color); // this includes the king itself

    let escape_squares = (king_ring & all_attacks & !friends).count_ones() as i32;

    if escape_squares == 0 {
        12
    } else if escape_squares <= 2 {
        3
    } else {
        -5
    }
}

fn defender_danger_bonus(board: &Board, color: Color, info: &EvalInfo) -> i32 {
    let defended_squares = info.king_ring(color) & info.attacked_by_two(color); // king is included in regular attacks

    let local_defenders = info.king_ring(color) & board.occupancy_of(color);

    let mut danger = 0;

    // since this represents king danger the bonus is negative
    danger -= defended_squares.count_ones() as i32;
    danger -= local_defenders.count_ones() as i32 * 2;

    danger
}

const fn generate_king_shield() -> [[Bitboard; 64]; 2] {
    let mut table = [[0u64; 64]; 2];

    // square
    let mut sq = 0u8;
    while sq < 64 {
        table[0][sq as usize] = generate_king_shield_mask(Color::White, sq);
        sq += 1;
    }

    sq = 0u8;
    while sq < 64 {
        table[1][sq as usize] = generate_king_shield_mask(Color::Black, sq);
        sq += 1;
    }

    table
}

const fn generate_king_two_shield() -> [[Bitboard; 64]; 2] {
    let mut table = [[0u64; 64]; 2];

    // square
    let mut sq = 0u8;
    while sq < 64 {
        table[0][sq as usize] = generate_king_shield_two_mask(Color::White, sq);
        sq += 1;
    }

    sq = 0u8;
    while sq < 64 {
        table[1][sq as usize] = generate_king_shield_two_mask(Color::Black, sq);
        sq += 1;
    }

    table
}

const fn generate_king_shield_mask(color: Color, sq: Square) -> Bitboard {
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

const fn generate_king_shield_two_mask(color: Color, sq: Square) -> Bitboard {
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
