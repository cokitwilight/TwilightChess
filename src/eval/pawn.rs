use crate::{
    bitboard::{
        Bitboard, FILE_MASKS, RANK_2, RANK_3, RANK_6, RANK_7, RANK_MASKS, Square, bit, file_of,
        pawn_attacks, pop_lsb, rank_of,
    },
    board::Board,
    types::{Color, PieceType},
};

pub fn pawn_eval(board: &Board, phase: i32) -> i32 {
    pawn_eval_raw(board, Color::White, phase) - pawn_eval_raw(board, Color::Black, phase)
}

pub fn pawn_eval_raw(board: &Board, color: Color, phase: i32) -> i32 {
    let mut score = 0;

    let pawns = board.pieces(color, PieceType::Pawn);

    score += stacked_pawns_bonus(board, color, pawns, phase);
    score += passed_pawn_bonus(board, color, pawns, phase);
    score += pawn_storm_bonus(board, color, pawns, phase);
    score += pawn_chain(board, color, pawns, phase);

    score
}

fn stacked_pawns_bonus(board: &Board, color: Color, pawns: Bitboard, phase: i32) -> i32 {
    if pawns == 0 {
        return 0;
    }
    let mut score = 0;

    for mask in FILE_MASKS {
        let pawns = (pawns & mask).count_ones() as i32;
        if pawns == 2 {
            score -= 10;
        } else if pawns == 3 {
            score -= 30;
        } else if pawns > 3 {
            score -= 15 * pawns;
        }
    }

    score
}

fn passed_pawn_bonus(board: &Board, color: Color, pawns: Bitboard, phase: i32) -> i32 {
    if pawns == 0 {
        return 0;
    }

    let mut score = 0;

    let enemy_pawns = board.pieces(color.opposite(), PieceType::Pawn);

    let mut pawns = pawns;

    while let Some(sq) = pop_lsb(&mut pawns) {
        if enemy_pawns & passed_pawn_mask(sq, color) != 0 {
            continue;
        }

        let rank = rank_of(sq);

        let relative_rank = match color {
            Color::White => rank,
            Color::Black => 7 - rank,
        };

        if relative_rank <= 3 {
            // passed pawn but not advanced
            score += 10;
            continue;
        }

        if relative_rank == 4 {
            score += 10;
        } else if relative_rank == 5 {
            score += 30;
        } else {
            score += 60;
        }
    }

    if phase < 12 {
        score *= 2;
    }

    score
}

fn pawn_storm_bonus(board: &Board, color: Color, pawns: Bitboard, phase: i32) -> i32 {
    if pawns == 0 {
        return 0;
    }
    if phase > 20 {
        return 0;
    }

    let mut score = 0;

    let starting_mask = match color {
        Color::White => RANK_2 | RANK_3,
        Color::Black => RANK_7 | RANK_6,
    };

    let Some(king_sq) = pop_lsb(&mut board.pieces(color, PieceType::King)) else {
        panic!("No king in pawn_storm_bonus!");
    };

    let king_file = file_of(king_sq);

    let mut king_file_mask = FILE_MASKS[king_file as usize];

    if king_file != 0 {
        king_file_mask |= FILE_MASKS[king_file as usize - 1];
    }
    if king_file != 7 {
        king_file_mask |= FILE_MASKS[king_file as usize + 1];
    }

    let starting_pawns = (pawns & starting_mask & !king_file_mask).count_ones();

    score -= starting_pawns as i32 * -8;

    score
}

fn pawn_chain(board: &Board, color: Color, pawns: Bitboard, phase: i32) -> i32 {
    if pawns == 0 {
        return 0;
    }

    let pawn_attacks = pawn_attacks(pawns, color);
    let defended_pawns = pawn_attacks & pawns;

    return defended_pawns.count_ones() as i32 * 4;
}

fn passed_pawn_mask(sq: Square, color: Color) -> Bitboard {
    let file = file_of(sq);
    let mut mask = bit(sq);

    if file != 0 {
        // pawn not on the outer left file
        mask |= bit(sq - 1);
    }

    if file != 7 {
        // pawn not on the outer right file
        mask |= bit(sq + 1);
    }

    match color {
        Color::White => {
            mask = mask << 8;
            mask |= mask << 8;
            mask |= mask << 16;
            mask |= mask << 32;
        }
        Color::Black => {
            mask = mask >> 8;
            mask |= mask >> 8;
            mask |= mask >> 16;
            mask |= mask >> 32;
        }
    }
    mask
}
