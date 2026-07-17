use crate::bitboard::{Square, bit, file_of, print_all_bitboards, rank_of, square};
use crate::board::{
    BLACK_KINGSIDE, BLACK_QUEENSIDE, Board, Move, MoveType, UndoMove, WHITE_KINGSIDE,
    WHITE_QUEENSIDE,
};
use crate::eval::pst::{eg_pst_bonus_at, mg_pst_bonus_at};
use crate::types::{COLORS, Color, PieceType};

const WHITE_HOME_RANK: u8 = 0;
const BLACK_HOME_RANK: u8 = 7;

impl Board {
    pub fn make_move(&mut self, mv: Move) -> UndoMove {
        let us = self.side_to_move;
        let them = us.opposite();

        let piece = self
            .piece_at(mv.from)
            .unwrap_or_else(|| panic!("make_move: no piece on from-square {}", mv.from));

        let moving_color = piece.color;
        let moving_piece = piece.kind;

        debug_assert_eq!(moving_color, us, "Tried to move wrong color");

        if let Some(promo) = mv.promotion {
            debug_assert_eq!(moving_piece, PieceType::Pawn, "Only pawns should promote");
            debug_assert_ne!(promo, PieceType::King, "Cannot promote to king");
            debug_assert_ne!(promo, PieceType::Pawn, "Cannot promote to pawn");
        }

        let captured_piece = if mv.kind == MoveType::EnPassant {
            let cap_sq = square(file_of(mv.to), rank_of(mv.from));

            Some((them, PieceType::Pawn, cap_sq))
        } else {
            self.piece_at(mv.to).map(|piece| {
                debug_assert_ne!(piece.color, us, "Tried to capture own piece");
                if piece.kind == PieceType::King {
                    print_all_bitboards(&self);
                    panic!("Move illegally captures king. Move: from: {}, to: {}, kind: {:?}, promtion: {}", mv.from, mv.to, mv.kind, mv.is_promotion());
                }
                // debug_assert_ne!(piece.kind, PieceType::King, "Move illegally captures king. Move: from: {}, to: {}, kind: {:?}, promtion: {}", mv.from, mv.to, mv.kind, mv.is_promotion());

                (piece.color, piece.kind, mv.to)
            })
        };

        let undo = UndoMove {
            mv,
            moved_piece: moving_piece,
            captured_piece,

            old_castling_rights: self.castling_rights,
            old_en_passant: self.en_passant,
            old_halfmove_clock: self.halfmove_clock,
            old_fullmove_number: self.fullmove_number,
            old_hash: self.hash,
            old_side_to_move: self.side_to_move,
            old_material: self.material,
            old_phase: self.phase,
            old_mg_pst_bonus: self.mg_pst,
            old_eg_pst_bonus: self.eg_pst,
        };

        // En passant target only lasts for one ply.
        self.clear_en_passant_hashed();

        // Remove moving piece from its source square.
        self.remove_piece_hashed(us, moving_piece, mv.from);
        self.remove_piece_increment(us, moving_piece, mv.from);

        // Remove captured piece, including en passant victim.
        if let Some((cap_color, cap_piece, cap_sq)) = captured_piece {
            self.remove_piece_hashed(cap_color, cap_piece, cap_sq);
            self.remove_piece_increment(cap_color, cap_piece, cap_sq);
        }

        // Handle castling rook movement.
        if moving_piece == PieceType::King
            && (mv.kind == MoveType::Castle || file_distance(mv.from, mv.to) == 2)
        // file distance technically not needed if MoveType::Castle is always used for castling, but this is a safety check
        {
            self.move_castling_rook_hashed(us, mv.from, mv.to);
            self.move_castling_rook_increment(us, mv.from, mv.to);
        }

        // Place piece on destination. Promotion replaces the pawn.
        let placed_piece = mv.promotion.unwrap_or(moving_piece);
        self.add_piece_hashed(us, placed_piece, mv.to);
        self.add_piece_increment(us, placed_piece, mv.to);

        // TODO: Add/remove piece increment compute phase and material despite the fact that after the move the total phase and material will be the same.
        // Additionally promotion could mess with material calculations. Ideally add a moved_piece_increment function that takes the from and to squares and handles promotion, but for now this is simpler.

        let old_castling_rights = self.castling_rights;

        // Update castling rights.
        self.update_castling_rights_after_move(us, moving_piece, mv.from);

        if let Some((cap_color, cap_piece, cap_sq)) = captured_piece {
            self.update_castling_rights_after_capture(cap_color, cap_piece, cap_sq);
        }

        self.update_castling_hash_after_rights_change(old_castling_rights);

        // Set new en passant square after a double pawn push.
        if moving_piece == PieceType::Pawn
            && same_file(mv.from, mv.to)
            && square_distance(mv.from, mv.to) == 16
        {
            let ep = (mv.from + mv.to) / 2;
            self.set_en_passant_hashed(Some(ep));
        }

        // Halfmove clock resets after pawn moves or captures.
        if moving_piece == PieceType::Pawn || captured_piece.is_some() {
            self.halfmove_clock = 0;
        } else {
            self.halfmove_clock += 1;
        }

        // Fullmove number increments after Black moves.
        if us == Color::Black {
            self.fullmove_number += 1;
        }

        self.xor_side_to_move_hash();
        self.side_to_move = them;

        // self.rebuild_occupancy();

        debug_assert_eq!(
            self.hash,
            self.compute_hash_from_scratch(),
            "Incremental hash mismatch after make_move"
        );
        undo
    }

    pub fn rebuild_occupancy(&mut self) {
        self.occupancy = [0; 2];

        for color in COLORS {
            let c = color.idx();

            let mut occ = 0;

            for p in 0..6 {
                occ |= self.pieces[c][p];
            }

            self.occupancy[c] = occ;
        }

        self.all_occupancy = self.occupancy[0] | self.occupancy[1];
    }

    fn move_castling_rook_hashed(&mut self, color: Color, king_from: Square, king_to: Square) {
        let rank = rank_of(king_from);
        let kingside = file_of(king_to) > file_of(king_from);

        let rook_from = if kingside {
            square(7, rank)
        } else {
            square(0, rank)
        };

        let rook_to = if kingside {
            square(5, rank)
        } else {
            square(3, rank)
        };

        debug_assert!(
            self.pieces[color.idx()][PieceType::Rook.idx()] & bit(rook_from) != 0,
            "Castling rook missing"
        );

        self.move_piece_hashed(color, PieceType::Rook, rook_from, rook_to);
    }

    fn update_castling_rights_after_move(&mut self, color: Color, piece: PieceType, from: Square) {
        match piece {
            PieceType::King => {
                self.clear_castling_rights_for_color(color);
            }

            PieceType::Rook => {
                self.clear_castling_rights_for_rook_square(color, from);
            }

            _ => {}
        }
    }

    fn update_castling_rights_after_capture(
        &mut self,
        captured_color: Color,
        captured_piece: PieceType,
        captured_square: Square,
    ) {
        if captured_piece == PieceType::Rook {
            self.clear_castling_rights_for_rook_square(captured_color, captured_square);
        }
    }
    fn clear_castling_rights_for_color(&mut self, color: Color) {
        match color {
            Color::White => {
                self.castling_rights &= !(WHITE_KINGSIDE | WHITE_QUEENSIDE);
            }

            Color::Black => {
                self.castling_rights &= !(BLACK_KINGSIDE | BLACK_QUEENSIDE);
            }
        }
    }

    fn clear_castling_rights_for_rook_square(&mut self, color: Color, sq: Square) {
        if rank_of(sq) != home_rank(color) {
            return;
        }

        match (color, file_of(sq)) {
            (Color::White, 0) => {
                self.castling_rights &= !WHITE_QUEENSIDE;
            }

            (Color::White, 7) => {
                self.castling_rights &= !WHITE_KINGSIDE;
            }

            (Color::Black, 0) => {
                self.castling_rights &= !BLACK_QUEENSIDE;
            }

            (Color::Black, 7) => {
                self.castling_rights &= !BLACK_KINGSIDE;
            }

            _ => {}
        }
    }

    // *****************************
    // **** INCREMENTAL UPDATES ****
    // *****************************

    fn remove_piece_increment(&mut self, color: Color, piece: PieceType, sq: Square) {
        let piece_value = piece.value();
        let piece_phase = piece.phase_value();
        let mg_pst_bonus = mg_pst_bonus_at(color, piece, sq);
        let eg_pst_bonus = eg_pst_bonus_at(color, piece, sq);

        self.phase -= piece_phase; // phase is updated regardless of color

        if color == Color::White {
            self.material -= piece_value;
            self.mg_pst -= mg_pst_bonus;
            self.eg_pst -= eg_pst_bonus;
        } else {
            self.material += piece_value;
            self.mg_pst += mg_pst_bonus;
            self.eg_pst += eg_pst_bonus;
        }
    }

    fn add_piece_increment(&mut self, color: Color, piece: PieceType, sq: Square) {
        let piece_value = piece.value();
        let piece_phase = piece.phase_value();
        let mg_pst_bonus = mg_pst_bonus_at(color, piece, sq);
        let eg_pst_bonus = eg_pst_bonus_at(color, piece, sq);

        self.phase += piece_phase; // phase is updated regardless of color

        if color == Color::White {
            self.material += piece_value;
            self.mg_pst += mg_pst_bonus;
            self.eg_pst += eg_pst_bonus;
        } else {
            self.material -= piece_value;
            self.mg_pst -= mg_pst_bonus;
            self.eg_pst -= eg_pst_bonus;
        }
    }

    fn move_castling_rook_increment(&mut self, color: Color, king_from: Square, king_to: Square) {
        let rank = rank_of(king_from);
        let kingside = file_of(king_to) > file_of(king_from);

        let rook_from = if kingside {
            square(7, rank)
        } else {
            square(0, rank)
        };

        let rook_to = if kingside {
            square(5, rank)
        } else {
            square(3, rank)
        };

        // TODO: Tehcnically phase and material remain the same so this is slightly wasteful, but it is simpler to just call remove and add.

        self.remove_piece_increment(color, PieceType::Rook, rook_from);
        // self.remove_piece_increment(color, PieceType::King, king_from);
        self.add_piece_increment(color, PieceType::Rook, rook_to);
        // self.add_piece_increment(color, PieceType::King, king_to);
    }
}

#[inline]
fn same_file(a: Square, b: Square) -> bool {
    file_of(a) == file_of(b)
}

#[inline]
fn square_distance(a: Square, b: Square) -> u8 {
    a.abs_diff(b)
}

#[inline]
fn file_distance(a: Square, b: Square) -> u8 {
    file_of(a).abs_diff(file_of(b))
}

#[inline]
fn home_rank(color: Color) -> u8 {
    match color {
        Color::White => WHITE_HOME_RANK,
        Color::Black => BLACK_HOME_RANK,
    }
}
