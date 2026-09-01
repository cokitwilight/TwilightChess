use crate::bitboard::{Bitboard, bit, pop_lsb, rank_of};
use crate::board::Board;
use crate::eval::EvalInfo;
use crate::eval::lookup::KNIGHT_OUTPOSTS;
use crate::eval::scale_by_phase;
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

// range per knight: -40 <-> 60
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
            let defended = knight_bb & info.attacks(color, PieceType::Pawn) != 0;

            let attacked = knight_bb
                & (info.attacks(enemy, PieceType::Knight) | info.attacks(enemy, PieceType::Bishop))
                != 0;

            if defended {
                // friendly pawn defending knight
                score += 30;
            } else {
                score += 10;
            }

            if attacked {
                score -= 50;
            } else {
                score += 30;
            }
            // TODO: Later add more detail like how valuable the knight outpost is
        }
    }

    score = scale_by_phase(score, info.phase(), 4, 8);

    score
}
