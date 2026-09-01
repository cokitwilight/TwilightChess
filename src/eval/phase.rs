use crate::board::Board;
use crate::types::{Color, PieceType};

pub const MAX_PHASE: i32 = 24;

// WILL LATER BE USED FOR DEBUGGING
pub fn calculate_phase(board: &Board) -> i32 {
    let knights = board.pieces(Color::White, PieceType::Knight)
        | board.pieces(Color::Black, PieceType::Knight);
    let bishops = board.pieces(Color::White, PieceType::Bishop)
        | board.pieces(Color::Black, PieceType::Bishop);
    let rooks =
        board.pieces(Color::White, PieceType::Rook) | board.pieces(Color::Black, PieceType::Rook);
    let queens =
        board.pieces(Color::White, PieceType::Queen) | board.pieces(Color::Black, PieceType::Queen);

    let mut phase = (knights.count_ones()
        + bishops.count_ones()
        + (rooks.count_ones() * 2)
        + (queens.count_ones() * 4)) as i32;

    phase = phase.min(MAX_PHASE);

    phase
}

impl PieceType {
    pub fn phase_value(self) -> i32 {
        match self {
            PieceType::Pawn => 0,
            PieceType::Knight => 1,
            PieceType::Bishop => 1,
            PieceType::Rook => 2,
            PieceType::Queen => 4,
            PieceType::King => 0,
        }
    }
}
