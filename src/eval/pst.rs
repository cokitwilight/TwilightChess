use crate::bitboard::{Square, mirror_square_vertical, pop_lsb};
use crate::board::Board;
use crate::eval::MAX_PHASE;
use crate::eval::lookup::ALL_PSTS;
use crate::types::{Color, PIECE_TYPES, PieceType};

pub fn pst_bonus(board: &Board, phase: i32) -> i32 {
    // White is positive, black is negative.
    let mut bonus = 0;
    let eg_phase = MAX_PHASE - phase;

    for p in PIECE_TYPES {
        let mut pieces = board.pieces(Color::White, p);

        while let Some(sq) = pop_lsb(&mut pieces) {
            let mg_pst_bonus = ALL_PSTS[p.idx()][sq as usize];
            let eg_pst_bonus = ALL_PSTS[p.idx() + 6][sq as usize];
            bonus += (mg_pst_bonus * phase + eg_pst_bonus * eg_phase) / MAX_PHASE;
        }
    }

    for p in PIECE_TYPES {
        let mut pieces = board.pieces(Color::Black, p);

        while let Some(sq) = pop_lsb(&mut pieces) {
            let mirror_sq = mirror_square_vertical(sq);
            let mg_pst_bonus = ALL_PSTS[p.idx()][mirror_sq as usize];
            let eg_pst_bonus = ALL_PSTS[p.idx() + 6][mirror_sq as usize];
            bonus -= (mg_pst_bonus * phase + eg_pst_bonus * eg_phase) / MAX_PHASE;
        }
    }

    bonus
}

pub fn mg_pst_bonus(board: &Board) -> i32 {
    // White is positive, black is negative.
    let mut bonus = 0;

    for p in PIECE_TYPES {
        let mut w_pieces = board.pieces(Color::White, p);
        let mut b_pieces = board.pieces(Color::Black, p);

        while let Some(sq) = pop_lsb(&mut w_pieces) {
            bonus += ALL_PSTS[p.idx()][sq as usize];
        }
        while let Some(sq) = pop_lsb(&mut b_pieces) {
            let mirror_sq = mirror_square_vertical(sq);
            bonus -= ALL_PSTS[p.idx()][mirror_sq as usize];
        }
    }

    bonus
}

pub fn eg_pst_bonus(board: &Board) -> i32 {
    // White is positive, black is negative.
    let mut bonus = 0;

    for p in PIECE_TYPES {
        let mut pieces = board.pieces(Color::White, p);

        while let Some(sq) = pop_lsb(&mut pieces) {
            bonus += ALL_PSTS[p.idx() + 6][sq as usize];
        }
    }

    for p in PIECE_TYPES {
        let mut pieces = board.pieces(Color::Black, p);

        while let Some(sq) = pop_lsb(&mut pieces) {
            let mirror_sq = mirror_square_vertical(sq);
            bonus -= ALL_PSTS[p.idx() + 6][mirror_sq as usize];
        }
    }

    bonus
}

pub fn mg_pst_bonus_at(color: Color, piece: PieceType, sq: Square) -> i32 {
    let final_sq = match color {
        Color::White => sq,
        Color::Black => mirror_square_vertical(sq),
    };

    ALL_PSTS[piece.idx()][final_sq as usize]
}

pub fn eg_pst_bonus_at(color: Color, piece: PieceType, sq: Square) -> i32 {
    let final_sq = match color {
        Color::White => sq,
        Color::Black => mirror_square_vertical(sq),
    };

    ALL_PSTS[piece.idx() + 6][final_sq as usize]
}
