use crate::bitboard::pins::generate_pin_masks;
use crate::bitboard::{Bitboard, RANK_3, RANK_4, RANK_5, RANK_6, bit, knight_attacks, pop_lsb};
use crate::board::Board;
use crate::eval::EvalInfo;
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

pub fn knight_outpost_bonus(
    board: &Board,
    color: Color,
    mut knights: Bitboard,
    info: &EvalInfo,
) -> (i32, i32) {
    let mut enemy_span = info.attacks(color.opposite(), PieceType::Pawn);
    let ranks = match color {
        Color::White => {
            enemy_span |= enemy_span >> 8;
            enemy_span |= enemy_span >> 16;
            enemy_span |= enemy_span >> 32;
            RANK_4 | RANK_5 | RANK_6
        }
        Color::Black => {
            enemy_span |= enemy_span << 8;
            enemy_span |= enemy_span << 16;
            enemy_span |= enemy_span << 32;
            RANK_3 | RANK_4 | RANK_5
        }
    };
    let outposts = ranks & !enemy_span;
    let supported = info.attacks(color, PieceType::Pawn);
    let destinations = outposts & !board.occupancy_of(color);
    let mut pin_masks = None;
    let (mut mg, mut eg) = (0, 0);
    while let Some(sq) = pop_lsb(&mut knights) {
        let occupied = bit(sq);
        let bonus = if occupied & outposts != 0 {
            if occupied & supported != 0 {
                (35, 12)
            } else {
                (21, 6)
            }
        } else {
            let reachable = knight_attacks(sq) & destinations;
            if reachable == 0 {
                continue;
            }
            let pins = pin_masks
                .get_or_insert_with(|| generate_pin_masks(board, color, info.king_square(color)));
            if reachable & pins[sq as usize] == 0 {
                continue;
            }
            if reachable & supported != 0 {
                (17, 6)
            } else {
                (11, 3)
            }
        };
        mg += bonus.0;
        eg += bonus.1;
    }
    (mg, eg)
}
