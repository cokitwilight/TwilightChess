use crate::bitboard::{Square, mirror_square_vertical, pop_lsb};
use crate::board::Board;
use crate::eval::phase::MAX_PHASE;
use crate::types::{Color, PIECE_TYPES, PieceType};

const ALL_PSTS: [[i32; 64]; 12] = [
    [
        // PAWN PST
        0, 0, 0, 0, 0, 0, 0, 0, 5, 10, 10, -20, -20, 10, 10, 5, 5, -5, -10, 0, 0, -10, -5, 5, 0, 10,
        20, 40, 40, 20, 10, 0, 5, 5, 10, 25, 25, 10, 5, 5, 10, 10, 20, 30, 30, 20, 10, 10, 50, 50,
        50, 50, 50, 50, 50, 50, 0, 0, 0, 0, 0, 0, 0, 0,
    ],
    [
        // KNIGHT PST
        -50, -40, -30, -30, -30, -30, -40, -50, -40, -20, 0, 5, 5, 0, -20, -40, -30, 5, 10, 15, 15,
        10, 5, -30, -30, 0, 15, 20, 20, 15, 0, -30, -30, 5, 15, 20, 20, 15, 5, -30, -30, 0, 10, 15,
        15, 10, 0, -30, -40, -20, 0, 0, 0, 0, -20, -40, -50, -40, -30, -30, -30, -30, -40, -50,
    ],
    [
        // BISHOP PST
        -20, -10, -10, -10, -10, -10, -10, -20, -10, 0, 0, 0, 0, 0, 0, -10, -10, 0, 5, 10, 10, 5, 0,
        -10, -10, 5, 5, 10, 10, 5, 5, -10, -10, 0, 10, 10, 10, 10, 0, -10, -10, 10, 10, 10, 10, 10,
        10, -10, -10, 5, 0, 0, 0, 0, 5, -10, -20, -10, -10, -10, -10, -10, -10, -20,
    ],
    [
        // ROOK PST
        0, 0, 0, 0, 0, 0, 0, 0, 5, 10, 10, 10, 10, 10, 10, 5, -5, 0, 0, 0, 0, 0, 0, -5, -5, 0, 0, 0,
        0, 0, 0, -5, -5, 0, 0, 0, 0, 0, 0, -5, -5, 0, 0, 0, 0, 0, 0, -5, -5, 0, 0, 0, 0, 0, 0, -5,
        0, 0, 0, 5, 5, 0, 0, 0,
    ],
    [
        // QUEEN PST
        -20, -10, -10, -5, -5, -10, -10, -20, -10, 0, 5, 0, 0, 0, 0, -10, -10, 5, 5, 5, 5, 5, 0,
        -10, 0, 0, 5, 5, 5, 5, 0, -5, -5, 0, 5, 5, 5, 5, 0, -5, -10, 0, 5, 5, 5, 5, 0, -10, -10, 0,
        0, 0, 0, 0, 0, -10, -20, -10, -10, -5, -5, -10, -10, -20,
    ],
    [
        // KING PST
        20, 30, 10, 0, 0, 10, 30, 20, 20, 20, 0, 0, 0, 0, 20, 20, -10, -20, -20, -20, -20, -20, -20,
        -10, -20, -30, -30, -40, -40, -30, -30, -20, -30, -40, -40, -50, -50, -40, -40, -30, -30,
        -40, -40, -50, -50, -40, -40, -30, -30, -40, -40, -50, -50, -40, -40, -30, -30, -40, -40,
        -50, -50, -40, -40, -30,
    ],
    [
        // PAWN ENDGAME PST
        0, 0, 0, 0, 0, 0, 0, 0, 5, 5, 5, 5, 5, 5, 5, 5, 15, 15, 15, 15, 15, 15, 15, 15, 30, 30, 30,
        30, 30, 30, 30, 30, 50, 50, 50, 50, 50, 50, 50, 50, 70, 70, 70, 70, 70, 70, 70, 70, 100,
        100, 100, 100, 100, 100, 100, 100, 0, 0, 0, 0, 0, 0, 0, 0,
    ],
    [
        // KNIGHT ENDGAME PST
        -60, -40, -30, -20, -20, -30, -40, -60, -40, -20, 0, 10, 10, 0, -20, -40, -30, 0, 20, 30,
        30, 20, 0, -30, -20, 10, 30, 40, 40, 30, 10, -20, -20, 10, 30, 40, 40, 30, 10, -20, -30, 0,
        20, 30, 30, 20, 0, -30, -40, -20, 0, 10, 10, 0, -20, -40, -60, -40, -30, -20, -20, -30,
        -40, -60,
    ],
    [
        // BISHOP ENDGAME PST
        -10, 0, 0, 0, 0, 0, 0, -10, 0, 10, 10, 10, 10, 10, 10, 0, 0, 10, 20, 20, 20, 20, 10, 0, 0,
        10, 20, 30, 30, 20, 10, 0, 0, 10, 20, 30, 30, 20, 10, 0, 0, 10, 20, 20, 20, 20, 10, 0, 0,
        10, 10, 10, 10, 10, 10, 0, -10, 0, 0, 0, 0, 0, 0, -10,
    ],
    [
        // ROOK ENDGAME PST
        0, 0, 0, 0, 0, 0, 0, 0, 5, 10, 10, 10, 10, 10, 10, 5, -5, 0, 0, 0, 0, 0, 0, -5, -5, 0, 0, 0,
        0, 0, 0, -5, -5, 0, 0, 0, 0, 0, 0, -5, -5, 0, 0, 0, 0, 0, 0, -5, -5, 0, 0, 0, 0, 0, 0, -5,
        0, 0, 0, 5, 5, 0, 0, 0,
    ],
    [
        // QUEEN ENDGAME PST
        -20, -10, -10, -5, -5, -10, -10, -20, -10, 0, 5, 0, 0, 0, 0, -10, -10, 5, 5, 5, 5, 5, 0,
        -10, 0, 0, 5, 5, 5, 5, 0, -5, -5, 0, 5, 5, 5, 5, 0, -5, -10, 0, 5, 5, 5, 5, 0, -10, -10, 0,
        0, 0, 0, 0, 0, -10, -20, -10, -10, -5, -5, -10, -10, -20,
    ],
    [
        // KING ENDGAME PST
        -50, -30, -10, 0, 0, -10, -30, -50, -30, -10, 20, 30, 30, 20, -10, -30, -10, 20, 30, 40, 40,
        30, 20, -10, 0, 30, 40, 50, 50, 40, 30, 0, 0, 30, 40, 50, 50, 40, 30, 0, -10, 20, 30, 40,
        40, 30, 20, -10, -30, -10, 20, 30, 30, 20, -10, -30, -50, -30, -10, 0, 0, -10, -30, -50,
    ],
];

pub fn pst_bonus(board: &Board, phase: i32) -> i32 {
    // white is positive, black is negative
    let mut bonus = 0;

    let eg_phase = MAX_PHASE - phase;

    // white
    for p in PIECE_TYPES {
        let mut pieces = board.pieces(Color::White, p);

        while let Some(sq) = pop_lsb(&mut pieces) {
            let mg_pst_bonus = ALL_PSTS[p.idx()][sq as usize];
            let eg_pst_bonus = ALL_PSTS[p.idx() + 6][sq as usize];
            bonus += (mg_pst_bonus * phase + eg_pst_bonus * eg_phase) / MAX_PHASE;
        }
    }

    // black
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
    // white is positive, black is negative
    let mut bonus = 0;

    // white
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
    // white is positive, black is negative
    let mut bonus = 0;

    // white
    for p in PIECE_TYPES {
        let mut pieces = board.pieces(Color::White, p);

        while let Some(sq) = pop_lsb(&mut pieces) {
            bonus += ALL_PSTS[p.idx() + 6][sq as usize];
        }
    }

    // black
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
