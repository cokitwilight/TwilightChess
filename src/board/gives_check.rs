use crate::bitboard::lookup::squares_aligned;
use crate::bitboard::{
    Square, bishop_attacks, bit, file_of, knight_attacks, pawn_attacks_from_square, queen_attacks,
    rank_of, rook_attacks, square,
};
use crate::board::{Board, Move, MoveType};
use crate::types::PieceType;

impl Board {
    /// Predicts whether a legal move checks the opposing king.
    pub fn move_gives_check(&self, mv: &Move) -> bool {
        let us = self.side_to_move();
        let them = us.opposite();
        let king = self.pieces(them, PieceType::King).trailing_zeros() as Square;
        let from = mv.from();
        let to = mv.to();
        let target = bit(to);
        let mut occupied = (self.all_occupancy() & !bit(from)) | target;

        if mv.kind() == MoveType::Castle {
            // Only the castling rook can check. Its old corner cannot block
            // the ray to its destination, and the king now occupies `to`.
            let rook_to = (from + to) / 2;
            return rook_attacks(king, occupied) & bit(rook_to) != 0;
        }

        let en_passant = mv.kind() == MoveType::EnPassant;
        let piece = if en_passant {
            occupied &= !bit(square(file_of(to), rank_of(from)));
            PieceType::Pawn
        } else {
            mv.promotion()
                .unwrap_or_else(|| self.piecetype_at(from).expect("missing moving piece"))
        };

        let direct = match piece {
            PieceType::Pawn => pawn_attacks_from_square(them, king),
            PieceType::Knight => knight_attacks(king),
            PieceType::Bishop => bishop_attacks(king, occupied),
            PieceType::Rook => rook_attacks(king, occupied),
            PieceType::Queen => queen_attacks(king, occupied),
            PieceType::King => 0,
        };
        if direct & target != 0 {
            return true;
        }

        // Normally only the source can uncover a slider. En passant vacates two squares.
        if !en_passant && !squares_aligned(from, king) {
            return false;
        }
        let queens = self.pieces(us, PieceType::Queen);
        let straight = file_of(from) == file_of(king) || rank_of(from) == rank_of(king);
        if straight || en_passant {
            let sliders = (self.pieces(us, PieceType::Rook) | queens) & !bit(from);
            if rook_attacks(king, occupied) & sliders != 0 {
                return true;
            }
        }
        if !straight || en_passant {
            let sliders = (self.pieces(us, PieceType::Bishop) | queens) & !bit(from);
            return bishop_attacks(king, occupied) & sliders != 0;
        }
        false
    }
}

#[cfg(test)]
mod move_gives_check_tests {
    use super::*;
    use crate::board::Move;
    use crate::types::PieceType;

    fn assert_move_gives_check(fen: &str, mv: Move, expected: bool) {
        let mut board = Board::from_fen(fen).expect("Invalid test FEN");

        let legal_moves = board.all_legal_moves();

        assert!(
            legal_moves.iter().any(|legal_mv| *legal_mv == mv),
            "Test move is not legal"
        );

        let original_hash = board.board_hash();

        // Result from the hypothetical occupancy implementation.
        let predicted = board.move_gives_check(&mv);

        // Ground-truth result from actually making the move.
        let undo = board.make_move(mv);

        // make_move switches side_to_move, so this checks whether
        // the opponent is now in check.
        let actual = board.in_check(board.side_to_move());

        board.undo_move(undo);

        assert_eq!(
            board.board_hash(),
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
    fn pawn_stays_between_rook_and_king() {
        let mv = Move::new(28, 36, MoveType::Normal, None); // e4-e5
        assert_move_gives_check("8/4k3/8/8/4P3/4R3/8/K7 w - - 0 1", mv, false);
    }

    #[test]
    fn promoted_slider_checks_from_its_destination() {
        for piece in [PieceType::Rook, PieceType::Queen] {
            let mv = Move::new(50, 58, MoveType::Normal, Some(piece)); // c7-c8
            assert_move_gives_check("8/2P4k/8/8/8/8/8/K7 w - - 0 1", mv, false);
        }
        for piece in [PieceType::Bishop, PieceType::Queen] {
            let mv = Move::new(54, 63, MoveType::Capture, Some(piece)); // g7xh8
            assert_move_gives_check("7r/6P1/5k2/8/8/8/8/K7 w - - 0 1", mv, true);
        }
    }

    #[test]
    fn en_passant_opens_rank_through_both_pawns() {
        let white = Move::new(36, 43, MoveType::EnPassant, None); // e5xd6
        assert_move_gives_check("8/8/8/R2pP2k/8/8/8/K7 w - d6 0 1", white, true);
        let black = Move::new(28, 19, MoveType::EnPassant, None); // e4xd3
        assert_move_gives_check("k7/8/8/8/r2Pp2K/8/8/8 b - d3 0 1", black, true);
    }

    #[test]
    fn castling_away_from_enemy_kings_file_is_not_check() {
        let mv = Move::new(4, 6, MoveType::Castle, None);
        assert_move_gives_check("4k3/8/8/8/8/8/8/4K2R w K - 0 1", mv, false);
    }

    #[test]
    fn normal_move_direct_check() {
        // Bishop c4-b5+ attacks the black king on e8.
        let mv = Move::new(
            26, // c4
            33, // b5
            MoveType::Normal,
            None,
        );

        assert_move_gives_check("4k3/8/8/8/2B5/8/8/4K3 w - - 0 1", mv, true);
    }

    #[test]
    fn normal_move_does_not_give_check() {
        // Bishop c4-d3 does not attack the black king on e8.
        let mv = Move::new(
            26, // c4
            19, // d3
            MoveType::Normal,
            None,
        );

        assert_move_gives_check("4k3/8/8/8/2B5/8/8/4K3 w - - 0 1", mv, false);
    }

    #[test]
    fn normal_move_discovered_check() {
        // Moving the bishop from e2 opens the e-file rook check.
        let mv = Move::new(
            12, // e2
            21, // f3
            MoveType::Normal,
            None,
        );

        assert_move_gives_check("4k3/8/8/8/8/8/4B3/K3R3 w - - 0 1", mv, true);
    }

    #[test]
    fn normal_move_double_check() {
        // Bishop e2-b5 checks directly while also uncovering the e1 rook.
        let mv = Move::new(
            12, // e2
            33, // b5
            MoveType::Normal,
            None,
        );

        assert_move_gives_check("4k3/8/8/8/8/8/4B3/K3R3 w - - 0 1", mv, true);
    }

    #[test]
    fn capture_gives_check() {
        // Rook a1 captures the knight on a7 and checks the king on a8.
        let mv = Move::new(
            0,  // a1
            48, // a7
            MoveType::Capture,
            None,
        );

        assert_move_gives_check("k7/n7/8/8/8/8/8/R6K w - - 0 1", mv, true);
    }

    #[test]
    fn quiet_promotion_gives_check() {
        // g7-g8=Q+ checks along the eighth rank.
        let mv = Move::new(
            54, // g7
            62, // g8
            MoveType::Normal,
            Some(PieceType::Queen),
        );

        assert_move_gives_check("k7/6P1/8/8/8/8/8/7K w - - 0 1", mv, true);
    }

    #[test]
    fn capture_promotion_gives_check() {
        // g7xh8=Q+ captures the rook and checks along the eighth rank.
        let mv = Move::new(
            54, // g7
            63, // h8
            MoveType::Capture,
            Some(PieceType::Queen),
        );

        assert_move_gives_check("k6r/6P1/8/8/8/8/8/7K w - - 0 1", mv, true);
    }

    #[test]
    fn en_passant_direct_check() {
        // e5xd6 e.p.; the pawn on d6 attacks the king on e7.
        let mv = Move::new(
            36, // e5
            43, // d6
            MoveType::EnPassant,
            None,
        );

        assert_move_gives_check("8/4k3/8/3pP3/8/8/8/K7 w - d6 0 1", mv, true);
    }

    #[test]
    fn en_passant_discovered_check() {
        // e5xd6 e.p. removes the pawn from the e-file,
        // uncovering the rook on e1.
        let mv = Move::new(
            36, // e5
            43, // d6
            MoveType::EnPassant,
            None,
        );

        assert_move_gives_check("4k3/8/8/3pP3/8/8/8/K3R3 w - d6 0 1", mv, true);
    }

    #[test]
    fn en_passant_double_check() {
        // The pawn on d6 directly attacks e7, while moving from e5
        // also uncovers the rook on e1.
        let mv = Move::new(
            36, // e5
            43, // d6
            MoveType::EnPassant,
            None,
        );

        assert_move_gives_check("8/4k3/8/3pP3/8/8/8/K3R3 w - d6 0 1", mv, true);
    }

    #[test]
    fn kingside_castling_gives_rook_check() {
        // White castles kingside. The rook moves h1-f1 and checks
        // the black king on f8.
        let mv = Move::new(
            4, // e1
            6, // g1
            MoveType::Castle,
            None,
        );

        assert_move_gives_check("5k2/8/8/8/8/8/8/4K2R w K - 0 1", mv, true);
    }

    #[test]
    fn queenside_castling_gives_rook_check() {
        // White castles queenside. The rook moves a1-d1 and checks
        // the black king on d8.
        let mv = Move::new(
            4, // e1
            2, // c1
            MoveType::Castle,
            None,
        );

        assert_move_gives_check("3k4/8/8/8/8/8/8/R3K3 w Q - 0 1", mv, true);
    }
}
