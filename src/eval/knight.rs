use crate::bitboard::{Bitboard, bit, pop_lsb, rank_of};
use crate::board::Board;
use crate::eval::EvalInfo;
use crate::eval::lookup::KNIGHT_OUTPOSTS;
use crate::eval::scale_by_phase;
use crate::types::{Color, PieceType};

pub fn knight_eval(board: &Board, info: &EvalInfo) -> (i32, i32) {
    let (white_mg, white_eg) = knight_eval_raw(board, Color::White, info);
    let (black_mg, black_eg) = knight_eval_raw(board, Color::Black, info);
    (white_mg - black_mg, white_eg - black_eg)
}

pub fn knight_eval_raw(board: &Board, color: Color, info: &EvalInfo) -> (i32, i32) {
    let mut mg = 0;
    let mut eg = mg;

    let knights = board.pieces(color, PieceType::Knight);

    let (term_mg, term_eg) = knight_outpost_bonus(board, color, knights, info);
    mg += term_mg;
    eg += term_eg;

    (mg, eg)
}

// range per knight: -40 <-> 60
pub fn knight_outpost_bonus(
    board: &Board,
    color: Color,
    knights: Bitboard,
    info: &EvalInfo,
) -> (i32, i32) {
    let mut mg = 0;
    let mut eg = mg;

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
                mg += 30;
                eg += 30;
            } else {
                mg += 10;
                eg += 10;
            }

            if attacked {
                mg -= 50;
                eg -= 50;
            } else {
                mg += 30;
                eg += 30;
            }
            // TODO: Later add more detail like how valuable the knight outpost is
        }
    }

    mg = scale_by_phase(mg, info.phase(), 4, 8);
    eg = scale_by_phase(eg, info.phase(), 4, 8);

    (mg, eg)
}
