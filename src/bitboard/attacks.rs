use crate::bitboard::rays::{all_bishop_attacks, all_queen_attacks, all_rook_attacks};
use crate::board::Board;
use crate::types::{Color, PieceType};

use crate::bitboard::{Bitboard, NOT_FILE_A, NOT_FILE_H, Square, attack_tables, pop_lsb};

#[inline]
pub fn knight_attacks(sq: Square) -> Bitboard {
    attack_tables().knight[sq as usize]
}

pub fn all_knight_attacks(knights: Bitboard) -> Bitboard {
    let mut attacks = 0;
    let mut knights_copy = knights;

    while let Some(sq) = pop_lsb(&mut knights_copy) {
        attacks |= knight_attacks(sq);
    }

    attacks
}

#[inline]
pub fn king_attacks(sq: Square) -> Bitboard {
    attack_tables().king[sq as usize]
}

#[inline]
pub fn pawn_attacks_from_square(sq: Square, color: Color) -> Bitboard {
    match color {
        Color::White => attack_tables().white_pawn[sq as usize],
        Color::Black => attack_tables().black_pawn[sq as usize],
    }
}

#[inline]
pub fn white_pawn_attacks(pawns: Bitboard) -> Bitboard {
    ((pawns & NOT_FILE_A) << 7) | ((pawns & NOT_FILE_H) << 9)
}

#[inline]
pub fn black_pawn_attacks(pawns: Bitboard) -> Bitboard {
    ((pawns & NOT_FILE_A) >> 9) | ((pawns & NOT_FILE_H) >> 7)
}

#[inline]
pub fn pawn_attacks(pawns: Bitboard, color: Color) -> Bitboard {
    match color {
        Color::White => white_pawn_attacks(pawns),
        Color::Black => black_pawn_attacks(pawns),
    }
}

pub fn all_attacks(board: &Board, by_color: Color) -> Bitboard {
    let occupancy = board.all_occupancy();
    let mut attacks = pawn_attacks(board.pieces(by_color, PieceType::Pawn), by_color);

    let Some(king_sq) = pop_lsb(&mut board.pieces(by_color, PieceType::King)) else {
        panic!("No king in all_attacks!");
    };

    attacks |= all_knight_attacks(board.pieces(by_color, PieceType::Knight));
    attacks |= king_attacks(king_sq);
    attacks |= all_bishop_attacks(board.pieces(by_color, PieceType::Bishop), occupancy);
    attacks |= all_rook_attacks(board.pieces(by_color, PieceType::Rook), occupancy);
    attacks |= all_queen_attacks(board.pieces(by_color, PieceType::Queen), occupancy);

    attacks
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bitboard::{bit, square};

    #[test]
    fn knight_attacks_from_e4() {
        // e4 = file 4, rank 3
        let e4 = square(4, 3);

        let expected = bit(square(3, 1))
            | bit(square(5, 1))
            | bit(square(2, 2))
            | bit(square(6, 2))
            | bit(square(2, 4))
            | bit(square(6, 4))
            | bit(square(3, 5))
            | bit(square(5, 5));

        assert_eq!(knight_attacks(e4), expected);
    }

    #[test]
    fn knight_attacks_from_a1() {
        let a1 = square(0, 0);

        let expected = bit(square(1, 2)) | bit(square(2, 1));

        assert_eq!(knight_attacks(a1), expected);
    }

    #[test]
    fn king_attacks_from_e4() {
        let e4 = square(4, 3);

        let expected = bit(square(3, 2))
            | bit(square(4, 2))
            | bit(square(5, 2))
            | bit(square(3, 3))
            | bit(square(5, 3))
            | bit(square(3, 4))
            | bit(square(4, 4))
            | bit(square(5, 4));

        assert_eq!(king_attacks(e4), expected);
    }

    #[test]
    fn king_attacks_from_a1() {
        let a1 = square(0, 0);

        let expected = bit(square(1, 0)) | bit(square(0, 1)) | bit(square(1, 1));

        assert_eq!(king_attacks(a1), expected);
    }

    #[test]
    fn white_pawn_attacks_from_e4() {
        let e4 = square(4, 3);

        let expected = bit(square(3, 4)) | bit(square(5, 4));

        assert_eq!(pawn_attacks_from_square(e4, Color::White), expected);
    }

    #[test]
    fn black_pawn_attacks_from_e4() {
        let e4 = square(4, 3);

        let expected = bit(square(3, 2)) | bit(square(5, 2));

        assert_eq!(pawn_attacks_from_square(e4, Color::Black), expected);
    }

    #[test]
    fn white_pawn_on_a_file_does_not_wrap() {
        let a2 = square(0, 1);

        let expected = bit(square(1, 2)); // b3 only

        assert_eq!(pawn_attacks_from_square(a2, Color::White), expected);
    }

    #[test]
    fn black_pawn_on_h_file_does_not_wrap() {
        let h7 = square(7, 6);

        let expected = bit(square(6, 5)); // g6 only

        assert_eq!(pawn_attacks_from_square(h7, Color::Black), expected);
    }
}
