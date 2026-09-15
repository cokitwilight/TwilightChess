use crate::bitboard::Square;
use crate::board::Board;

#[derive(Clone, PartialEq, Eq)]
pub struct UndoNullMove {
    pub en_passant: Option<Square>,
    pub halfmove_clock: u16,
    pub board_hash: u64,
    pub pawn_hash: u64,
}

impl Board {
    pub fn make_null_move(&mut self) -> UndoNullMove {
        let undo = UndoNullMove {
            en_passant: self.en_passant,
            halfmove_clock: self.halfmove_clock,
            board_hash: self.board_hash,
            pawn_hash: self.pawn_hash,
        };

        self.xor_side_to_move_hash();
        self.side_to_move = self.side_to_move.opposite();

        self.clear_en_passant_hashed(); // this checks if en square exists and sets en_passant to None

        self.halfmove_clock += 1;

        undo
    }
    pub fn undo_null_move(&mut self, undo: UndoNullMove) {
        self.side_to_move = self.side_to_move.opposite();

        self.en_passant = undo.en_passant;
        self.halfmove_clock = undo.halfmove_clock;
        self.board_hash = undo.board_hash;
        self.pawn_hash = undo.pawn_hash;
    }
}

pub fn null_move_reduction(depth: u16) -> u16 {
    if depth >= 6 { 3 } else { 2 }
}
