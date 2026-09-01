use crate::bitboard::{
    Bitboard, NOT_FILE_A, NOT_FILE_H, Square, all_bishop_attacks, all_queen_attacks,
    all_rook_attacks, attack_tables, bishop_attacks, pop_lsb, rook_attacks,
};
use crate::board::Board;
use crate::types::{Color, PieceType};

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
pub fn pawn_attacks_from_square(color: Color, sq: Square) -> Bitboard {
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
pub fn pawn_attacks(color: Color, pawns: Bitboard) -> Bitboard {
    match color {
        Color::White => white_pawn_attacks(pawns),
        Color::Black => black_pawn_attacks(pawns),
    }
}

pub fn all_attacks(board: &Board, by_color: Color) -> Bitboard {
    let occupancy = board.all_occupancy();
    let mut attacks = pawn_attacks(by_color, board.pieces(by_color, PieceType::Pawn));

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

/// Returns all pieces of `by_color` attacking `target` with the supplied occupancy.
pub fn attackers_to(
    pieces: &[[Bitboard; 6]; 2],
    occupied: Bitboard,
    target: Square,
    by_color: Color,
) -> Bitboard {
    let pawns = pieces[by_color.idx()][PieceType::Pawn.idx()];
    let knights = pieces[by_color.idx()][PieceType::Knight.idx()];
    let bishops = pieces[by_color.idx()][PieceType::Bishop.idx()];
    let rooks = pieces[by_color.idx()][PieceType::Rook.idx()];
    let queens = pieces[by_color.idx()][PieceType::Queen.idx()];
    let king = pieces[by_color.idx()][PieceType::King.idx()];

    let pawn_attackers = pawn_attacks_from_square(by_color.opposite(), target) & pawns;
    let knight_attackers = knight_attacks(target) & knights;
    let bishop_attackers = bishop_attacks(target, occupied) & (bishops | queens);
    let rook_attackers = rook_attacks(target, occupied) & (rooks | queens);
    let king_attackers = king_attacks(target) & king;

    pawn_attackers | knight_attackers | bishop_attackers | rook_attackers | king_attackers
}

pub fn square_attacked(
    pieces: &[[Bitboard; 6]; 2],
    occupied: Bitboard,
    target: Square,
    by_color: Color,
) -> bool {
    attackers_to(pieces, occupied, target, by_color) != 0
}

pub fn king_in_check(pieces: &[[Bitboard; 6]; 2], occupied: Bitboard, color: Color) -> bool {
    let king = pieces[color.idx()][PieceType::King.idx()];
    debug_assert!(king != 0, "No king found for {:?}", color);
    debug_assert_eq!(
        king.count_ones(),
        1,
        "Expected exactly one king for {:?}",
        color
    );

    let king_sq = king.trailing_zeros() as Square;
    square_attacked(pieces, occupied, king_sq, color.opposite())
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

        assert_eq!(pawn_attacks_from_square(Color::White, e4), expected);
    }

    #[test]
    fn black_pawn_attacks_from_e4() {
        let e4 = square(4, 3);

        let expected = bit(square(3, 2)) | bit(square(5, 2));

        assert_eq!(pawn_attacks_from_square(Color::Black, e4), expected);
    }

    #[test]
    fn white_pawn_on_a_file_does_not_wrap() {
        let a2 = square(0, 1);

        let expected = bit(square(1, 2)); // b3 only

        assert_eq!(pawn_attacks_from_square(Color::White, a2), expected);
    }

    #[test]
    fn black_pawn_on_h_file_does_not_wrap() {
        let h7 = square(7, 6);

        let expected = bit(square(6, 5)); // g6 only

        assert_eq!(pawn_attacks_from_square(Color::Black, h7), expected);
    }
}
