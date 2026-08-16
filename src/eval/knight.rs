use crate::bitboard::{Bitboard, Square, bit, file_of, pop_lsb, rank_of};
use crate::board::Board;
use crate::eval::eval::EvalInfo;
use crate::eval::scale_by_phase;
use crate::types::{Color, PieceType};

static KNIGHT_OUTPOSTS: [[Bitboard; 64]; 2] = calculate_outposts();

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

        let outpost_mask = KNIGHT_OUTPOSTS[color.idx()][knight_sq as usize]; // all squares to the adjacent files and in front of the knight

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

    score = scale_by_phase(score, info.phase(), 4, 8);

    score
}

const fn calculate_outposts() -> [[Bitboard; 64]; 2] {
    // although this fills squares for knights that are on the incorrect side it would never be called on low ranks
    let mut table = [[0u64; 64]; 2];

    let mut sq = 0u8;
    while sq < 64 {
        table[0][sq as usize] = calculate_outpost_mask(Color::White, sq);
        sq += 1;
    }

    sq = 0u8;
    while sq < 64 {
        table[1][sq as usize] = calculate_outpost_mask(Color::Black, sq);
        sq += 1;
    }

    table
}

const fn calculate_outpost_mask(color: Color, sq: Square) -> Bitboard {
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
            mask <<= 8;
            mask |= mask << 8;
            mask |= mask << 16;
        }
        Color::Black => {
            mask >>= 8;
            mask |= mask >> 8;
            mask |= mask >> 16;
        }
    }

    mask
}
