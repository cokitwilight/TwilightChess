use crate::bitboard::{
    Bitboard, Square, bishop_attacks, bit, king_attacks, knight_attacks, pawn_attacks_from_square,
    rook_attacks,
};
use crate::board::{Board, Move, MoveType};
use crate::types::{Color, PieceType};

impl Board {
    pub fn move_gives_check(&self, mv: &Move) -> bool {
        let from = mv.from;
        let to = mv.to;

        let from_bb = bit(mv.from);
        let to_bb = bit(mv.to);

        let side_to_move = self.side_to_move();

        let piece = self.piece_at(from).expect("No piece in move_gives_check!");

        let mut all_pieces = self.all_pieces();
        let mut occupancy = self.all_occupancy();

        if mv.kind == MoveType::EnPassant || mv.kind == MoveType::Castle {
            let mut temp = self.clone();

            temp.make_move(*mv);

            return temp.in_check(temp.side_to_move());
        }

        match mv.kind {
            MoveType::Normal => {
                // move the piece from mv.from -> mv.to
                all_pieces[side_to_move.idx()][piece.kind.idx()] &= !from_bb;
                occupancy &= !from_bb;

                if let Some(pc) = mv.promotion {
                    all_pieces[side_to_move.idx()][pc.idx()] |= to_bb;
                } else {
                    all_pieces[side_to_move.idx()][piece.kind.idx()] |= to_bb;
                }

                occupancy |= to_bb;
                return check_from_occupancy(&all_pieces, occupancy, side_to_move.opposite());
            }
            MoveType::Capture => {
                let captured_piece = self
                    .piece_at(to)
                    .expect("No captured piece in move_gives_check!");

                all_pieces[side_to_move.idx()][piece.kind.idx()] &= !from_bb; // remove the moving piece
                all_pieces[side_to_move.opposite().idx()][captured_piece.kind.idx()] &= !to_bb; // remove the captured piece

                if let Some(pc) = mv.promotion {
                    all_pieces[side_to_move.idx()][pc.idx()] |= to_bb;
                } else {
                    all_pieces[side_to_move.idx()][piece.kind.idx()] |= to_bb;
                }

                occupancy &= !from_bb; // adjust occupancy. Note the to square remains the same

                return check_from_occupancy(&all_pieces, occupancy, side_to_move.opposite());
            }
            // since these are rare and edge cases just clone the board
            MoveType::EnPassant => {
                let captured_sq = match side_to_move {
                    Color::White => to
                        .checked_sub(8)
                        .expect("Invalid white en-passant destination"),
                    Color::Black => to
                        .checked_add(8)
                        .expect("Invalid black en-passant destination"),
                };
                let captured_bb = bit(captured_sq);

                // Remove the moving pawn from its original square.
                all_pieces[side_to_move.idx()][PieceType::Pawn.idx()] &= !from_bb;

                // Remove the captured enemy pawn from its actual square.
                all_pieces[side_to_move.opposite().idx()][PieceType::Pawn.idx()] &= !captured_bb;

                // Place the moving pawn on the en-passant destination.
                all_pieces[side_to_move.idx()][PieceType::Pawn.idx()] |= to_bb;

                // Update occupancy for all three affected squares.
                occupancy &= !from_bb;
                occupancy &= !captured_bb;
                occupancy |= to_bb;

                return check_from_occupancy(&all_pieces, occupancy, side_to_move.opposite());
            }
            MoveType::Castle => {
                let (rook_from, rook_to) = match (side_to_move, to) {
                    (Color::White, 6) => (7, 5),    // e1g1: h1 -> f1
                    (Color::White, 2) => (0, 3),    // e1c1: a1 -> d1
                    (Color::Black, 62) => (63, 61), // e8g8: h8 -> f8
                    (Color::Black, 58) => (56, 59), // e8c8: a8 -> d8

                    _ => panic!(
                        "Invalid castling move in move_gives_check: {:?} -> {:?}",
                        mv.from, mv.to
                    ),
                };

                let rook_from_bb = bit(rook_from);
                let rook_to_bb = bit(rook_to);

                // Move the king.
                all_pieces[side_to_move.idx()][PieceType::King.idx()] &= !from_bb;
                all_pieces[side_to_move.idx()][PieceType::King.idx()] |= to_bb;

                // Move the rook.
                all_pieces[side_to_move.idx()][PieceType::Rook.idx()] &= !rook_from_bb;
                all_pieces[side_to_move.idx()][PieceType::Rook.idx()] |= rook_to_bb;

                // Remove both pieces from their original squares.
                occupancy &= !from_bb;
                occupancy &= !rook_from_bb;

                // Add both pieces to their destination squares.
                occupancy |= to_bb;
                occupancy |= rook_to_bb;

                return check_from_occupancy(&all_pieces, occupancy, side_to_move.opposite());
            }
        }
    }
}

fn check_from_occupancy(pieces: &[[Bitboard; 6]; 2], occupied: Bitboard, color: Color) -> bool {
    let king = pieces[color.idx()][PieceType::King.idx()];
    debug_assert!(king != 0, "No king found for {:?}", color);

    debug_assert!(
        king.count_ones() == 1,
        "Expected exactly one king for {:?}, found {}",
        color,
        king.count_ones()
    );

    let king_sq = king.trailing_zeros() as Square;
    square_attacked(pieces, occupied, king_sq, color.opposite())
}

fn square_attacked(pieces: &[[Bitboard; 6]; 2], occupied: Bitboard, sq: Square, by: Color) -> bool {
    let pawns = pieces[by.idx()][PieceType::Pawn.idx()];
    let knights = pieces[by.idx()][PieceType::Knight.idx()];
    let bishops = pieces[by.idx()][PieceType::Bishop.idx()];
    let rooks = pieces[by.idx()][PieceType::Rook.idx()];
    let queens = pieces[by.idx()][PieceType::Queen.idx()];
    let king = pieces[by.idx()][PieceType::King.idx()];

    let pawn_attackers = match by {
        Color::White => pawn_attacks_from_square(sq, Color::Black) & pawns,
        Color::Black => pawn_attacks_from_square(sq, Color::White) & pawns,
    };

    if pawn_attackers != 0 {
        return true;
    }

    // -------------------------
    // Knights
    // -------------------------
    if knight_attacks(sq) & knights != 0 {
        return true;
    }

    // -------------------------
    // Kings
    // -------------------------
    if king_attacks(sq) & king != 0 {
        return true;
    }

    // -------------------------
    // Bishops / Queens
    // -------------------------
    let diagonal_attackers = bishops | queens;

    if bishop_attacks(sq, occupied) & diagonal_attackers != 0 {
        return true;
    }
    // -------------------------
    // Rooks / Queens
    // -------------------------
    let straight_attackers = rooks | queens;

    if rook_attacks(sq, occupied) & straight_attackers != 0 {
        return true;
    }

    false
}

#[cfg(test)]
mod move_gives_check_tests {
    use super::*;
    use crate::board::Move;
    use crate::types::{Color, Piece, PieceType};

    fn assert_move_gives_check(fen: &str, mv: Move, expected: bool) {
        let mut board = Board::from_fen(fen).expect("Invalid test FEN");

        let legal_moves = board.all_legal_moves();

        assert!(
            legal_moves.iter().any(|legal_mv| *legal_mv == mv),
            "Test move is not legal"
        );

        let original_hash = board.hash();

        // Result from the hypothetical occupancy implementation.
        let predicted = board.move_gives_check(&mv);

        // Ground-truth result from actually making the move.
        let undo = board.make_move(mv);

        // make_move switches side_to_move, so this checks whether
        // the opponent is now in check.
        let actual = board.in_check(board.side_to_move());

        board.undo_move(undo);

        assert_eq!(
            board.hash(),
            original_hash,
            "Board was not restored after test move"
        );

        assert_eq!(
            actual, expected,
            "The test position itself did not produce the expected result"
        );

        assert_eq!(
            predicted, actual,
            "move_gives_check disagreed with make_move + in_check"
        );
    }

    #[test]
    fn normal_move_direct_check() {
        // Bishop c4-b5+ attacks the black king on e8.
        let mv = Move {
            from: 26, // c4
            to: 33,   // b5
            kind: MoveType::Normal,
            promotion: None,
        };

        assert_move_gives_check("4k3/8/8/8/2B5/8/8/4K3 w - - 0 1", mv, true);
    }

    #[test]
    fn normal_move_does_not_give_check() {
        // Bishop c4-d3 does not attack the black king on e8.
        let mv = Move {
            from: 26, // c4
            to: 19,   // d3
            kind: MoveType::Normal,
            promotion: None,
        };

        assert_move_gives_check("4k3/8/8/8/2B5/8/8/4K3 w - - 0 1", mv, false);
    }

    #[test]
    fn normal_move_discovered_check() {
        // Moving the bishop from e2 opens the e-file rook check.
        let mv = Move {
            from: 12, // e2
            to: 21,   // f3
            kind: MoveType::Normal,
            promotion: None,
        };

        assert_move_gives_check("4k3/8/8/8/8/8/4B3/K3R3 w - - 0 1", mv, true);
    }

    #[test]
    fn normal_move_double_check() {
        // Bishop e2-b5 checks directly while also uncovering the e1 rook.
        let mv = Move {
            from: 12, // e2
            to: 33,   // b5
            kind: MoveType::Normal,
            promotion: None,
        };

        assert_move_gives_check("4k3/8/8/8/8/8/4B3/K3R3 w - - 0 1", mv, true);
    }

    #[test]
    fn capture_gives_check() {
        // Rook a1 captures the knight on a7 and checks the king on a8.
        let mv = Move {
            from: 0, // a1
            to: 48,  // a7
            kind: MoveType::Capture,
            promotion: None,
        };

        assert_move_gives_check("k7/n7/8/8/8/8/8/R6K w - - 0 1", mv, true);
    }

    #[test]
    fn quiet_promotion_gives_check() {
        // g7-g8=Q+ checks along the eighth rank.
        let mv = Move {
            from: 54, // g7
            to: 62,   // g8
            kind: MoveType::Normal,
            promotion: Some(PieceType::Queen),
        };

        assert_move_gives_check("k7/6P1/8/8/8/8/8/7K w - - 0 1", mv, true);
    }

    #[test]
    fn capture_promotion_gives_check() {
        // g7xh8=Q+ captures the rook and checks along the eighth rank.
        let mv = Move {
            from: 54, // g7
            to: 63,   // h8
            kind: MoveType::Capture,
            promotion: Some(PieceType::Queen),
        };

        assert_move_gives_check("k6r/6P1/8/8/8/8/8/7K w - - 0 1", mv, true);
    }

    #[test]
    fn en_passant_direct_check() {
        // e5xd6 e.p.; the pawn on d6 attacks the king on e7.
        let mv = Move {
            from: 36, // e5
            to: 43,   // d6
            kind: MoveType::EnPassant,
            promotion: None,
        };

        assert_move_gives_check("8/4k3/8/3pP3/8/8/8/K7 w - d6 0 1", mv, true);
    }

    #[test]
    fn en_passant_discovered_check() {
        // e5xd6 e.p. removes the pawn from the e-file,
        // uncovering the rook on e1.
        let mv = Move {
            from: 36, // e5
            to: 43,   // d6
            kind: MoveType::EnPassant,
            promotion: None,
        };

        assert_move_gives_check("4k3/8/8/3pP3/8/8/8/K3R3 w - d6 0 1", mv, true);
    }

    #[test]
    fn en_passant_double_check() {
        // The pawn on d6 directly attacks e7, while moving from e5
        // also uncovers the rook on e1.
        let mv = Move {
            from: 36, // e5
            to: 43,   // d6
            kind: MoveType::EnPassant,
            promotion: None,
        };

        assert_move_gives_check("8/4k3/8/3pP3/8/8/8/K3R3 w - d6 0 1", mv, true);
    }

    #[test]
    fn kingside_castling_gives_rook_check() {
        // White castles kingside. The rook moves h1-f1 and checks
        // the black king on f8.
        let mv = Move {
            from: 4, // e1
            to: 6,   // g1
            kind: MoveType::Castle,
            promotion: None,
        };

        assert_move_gives_check("5k2/8/8/8/8/8/8/4K2R w K - 0 1", mv, true);
    }

    #[test]
    fn queenside_castling_gives_rook_check() {
        // White castles queenside. The rook moves a1-d1 and checks
        // the black king on d8.
        let mv = Move {
            from: 4, // e1
            to: 2,   // c1
            kind: MoveType::Castle,
            promotion: None,
        };

        assert_move_gives_check("3k4/8/8/8/8/8/8/R3K3 w Q - 0 1", mv, true);
    }
}
