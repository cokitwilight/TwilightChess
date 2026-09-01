use crate::bitboard::{Square, file_distance, file_of, rank_of, square};
use crate::board::{Board, Move, MoveType};
use crate::types::{Color, PieceType};

#[derive(Clone, Copy)]
pub struct UndoMove {
    pub mv: Move,

    pub moved_piece: PieceType,
    pub captured_piece: Option<(Color, PieceType, Square)>,

    pub old_castling_rights: u8,
    pub old_en_passant: Option<Square>,
    pub old_halfmove_clock: u16,
    pub old_fullmove_number: u16,
    pub old_hash: u64,
    pub old_side_to_move: Color,
    pub old_material: i32,
    pub old_phase: i32,
    pub old_mg_pst_bonus: i32,
    pub old_eg_pst_bonus: i32,
}

impl Board {
    pub fn undo_move(&mut self, undo: UndoMove) {
        let mv = undo.mv;
        let us = undo.old_side_to_move;

        self.side_to_move = undo.old_side_to_move;
        self.castling_rights = undo.old_castling_rights;
        self.en_passant = undo.old_en_passant;
        self.halfmove_clock = undo.old_halfmove_clock;
        self.fullmove_number = undo.old_fullmove_number;

        // Restore material and phase.
        self.material = undo.old_material;
        self.phase = undo.old_phase;
        self.mg_pst = undo.old_mg_pst_bonus;
        self.eg_pst = undo.old_eg_pst_bonus;

        let placed_piece = mv.promotion().unwrap_or(undo.moved_piece);

        // Remove moved/promoted piece from destination.
        self.remove_piece(us, placed_piece, mv.to());

        // Undo castling rook movement.
        if undo.moved_piece == PieceType::King
            && (mv.kind() == MoveType::Castle || file_distance(mv.from(), mv.to()) == 2)
        {
            self.undo_castling_rook(us, mv.from(), mv.to());
        }

        // Put original moving piece back.
        self.add_piece(us, undo.moved_piece, mv.from());

        // Restore captured piece.
        if let Some((cap_color, cap_piece, cap_sq)) = undo.captured_piece {
            self.add_piece(cap_color, cap_piece, cap_sq);
        }

        // self.rebuild_occupancy();  // HOTSPOT. TRY AND MAKE INCREMENTAL

        self.hash = undo.old_hash;
        debug_assert_eq!(
            self.hash,
            self.compute_hash_from_scratch(),
            "Hash mismatch after undo_move"
        );
    }

    fn undo_castling_rook(&mut self, color: Color, king_from: Square, king_to: Square) {
        let rank = rank_of(king_from);
        let kingside = file_of(king_to) > file_of(king_from);

        let rook_from = if kingside {
            square(5, rank)
        } else {
            square(3, rank)
        };

        let rook_to = if kingside {
            square(7, rank)
        } else {
            square(0, rank)
        };

        self.remove_piece(color, PieceType::Rook, rook_from);
        self.add_piece(color, PieceType::Rook, rook_to);
    }
}
