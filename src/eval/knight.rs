use crate::bitboard::{Bitboard, Square, bit, file_of, pop_lsb, rank_of};
use crate::board::Board;
use crate::eval::eval::EvalInfo;
use crate::types::{Color, PieceType};

pub fn knight_eval(board: &Board, info: &EvalInfo) -> i32 {
    knight_eval_raw(board, Color::White, info) - knight_eval_raw(board, Color::Black, info)
}

pub fn knight_eval_raw(board: &Board, color: Color, info: &EvalInfo) -> i32 {
    let mut score = 0;

    let knights = board.pieces(color, PieceType::Knight);

    score += knight_outpost_bonus(board, color, knights, info);

    score
}

pub fn knight_outpost_bonus(
    board: &Board,
    color: Color,
    knights: Bitboard,
    info: &EvalInfo,
) -> i32 {
    let mut score = 0;

    let enemy = color.opposite();

    let enemy_pawns = board.pieces(enemy, PieceType::Pawn);

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

        if enemy_pawns & outpost_mask == 0 {
            let knight_bb = bit(knight_sq);
            // no adjacent pawns can attack the knight
            let defended = knight_bb & info.attacks(enemy, PieceType::Pawn) != 0;

            let attacked = knight_bb
                & (info.attacks(enemy, PieceType::Knight) | info.attacks(enemy, PieceType::Bishop))
                != 0;

            if defended {
                // friendly pawn defending knight
                score += 50;
            } else {
                score += 20;
            }

            if attacked {
                score -= 20;
            } else {
                score += 50;
            }
            // TODO: Later add more detail like how valuable the knight outpost is
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
