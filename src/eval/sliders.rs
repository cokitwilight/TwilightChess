use crate::{
    bitboard::{Bitboard, bit, file_of, pop_lsb, rank_of, square},
    board::Board,
    types::{Color, PieceType},
};

pub fn sliders_eval(board: &Board, phase: i32) -> i32 {
    sliders_eval_raw(board, Color::White, phase) - sliders_eval_raw(board, Color::Black, phase)
}

pub fn sliders_eval_raw(board: &Board, color: Color, phase: i32) -> i32 {
    let mut score = 0;

    let sliders = board.pieces(color, PieceType::Bishop) | board.pieces(color, PieceType::Queen);

    score
}

fn connected_diagonals_bonus(board: &Board, color: Color, sliders: Bitboard, phase: i32) -> i32 {
    let mut score = 0;

    let mut sliders = sliders;

    let occupied = board.all_occupancy();

    let directions = &[(1, 1), (1, -1), (-1, 1), (-1, -1)];

    while let Some(sq) = pop_lsb(&mut sliders) {
        for (df, dr) in directions {
            let mut file = file_of(sq) as i8 + df;
            let mut rank = rank_of(sq) as i8 + dr;

            while (0..8).contains(&file) && (0..8).contains(&rank) {
                let ray_sq = square(file as u8, rank as u8);
                let ray_mask = bit(ray_sq);

                if ray_mask & occupied != 0 {
                    if ray_mask & sliders != 0 {
                        score += 30;
                        sliders &= !ray_mask; // dont double count pieces
                    }
                    break;
                }

                file += df;
                rank += dr;
            }
        }
    }

    score
}
