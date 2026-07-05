use crate::{
    bitboard::{Bitboard, Square, bit, file_of, pop_lsb, rank_of},
    board::Board,
    types::{Color, PieceType},
};

pub fn knight_eval(board: &Board, phase: i32) -> i32 {
    0 // placeholder
}

pub fn knight_eval_raw(board: &Board, color: Color, phase: i32) -> i32 {
    let mut score = 0;

    let knights = board.pieces(color, PieceType::Knight);

    score += knight_outpost_bonus(board, color, knights, phase);

    score
}

pub fn knight_outpost_bonus(board: &Board, color: Color, knights: Bitboard, phase: i32) -> i32 {
    let mut score = 0;

    let friendly_pawns = board.pieces(color, PieceType::Pawn);
    let enemy_pawns = board.pieces(color.opposite(), PieceType::Pawn);

    let mut knights = knights;

    while let Some(knight_sq) = pop_lsb(&mut knights) {
        let rank = rank_of(knight_sq);

        match color {
            // don't give a bonus if the knight is still in own half
            Color::White => {
                if rank < 4 {
                    continue;
                }
            }
            Color::Black => {
                if rank > 3 {
                    continue;
                }
            }
        }

        let outpost_mask = calculate_outpost_mask(knight_sq, color);
        let pawn_mask = calculate_pawn_mask(knight_sq, color);

        if enemy_pawns & outpost_mask == 0 {
            // no adjacent pawns can attack the knight
            if friendly_pawns & pawn_mask != 0 {
                // friendly pawn defending knight
                score += 40;
            } else {
                score += 10;
            }
        }
    }

    score
}

fn calculate_outpost_mask(sq: Square, color: Color) -> Bitboard {
    // creates a mask of all squares ahead of the knight and in the adjacent files(1-2)
    let file = file_of(sq);
    let mut mask = 0u64;

    if file != 0 {
        // knight not on the outer left file
        mask |= bit(sq - 1);
    }

    if file != 7 {
        // knight not on the outer right file
        mask |= bit(sq + 1);
    }

    match color {
        Color::White => {
            mask = mask << 8;
            mask |= mask << 8;
            mask |= mask << 16;
        }
        Color::Black => {
            mask = mask >> 8;
            mask |= mask >> 8;
            mask |= mask >> 16;
        }
    }

    mask
}

fn calculate_pawn_mask(sq: Square, color: Color) -> Bitboard {
    let file = file_of(sq);
    let mut mask = 0u64;

    if file != 0 {
        // knight not on the outer left file
        mask |= bit(sq - 1);
    }

    if file != 7 {
        // knight not on the outer right file
        mask |= bit(sq + 1);
    }

    match color {
        Color::White => {
            mask = mask >> 8;
        }
        Color::Black => {
            mask = mask << 8;
        }
    }

    mask
}
