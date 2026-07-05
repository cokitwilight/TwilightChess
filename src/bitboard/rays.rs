use crate::bitboard::{Bitboard, Square, bit, file_of, pop_lsb, rank_of, square};

/// Generate attacks in one direction until edge of board or first blocker.
///
/// `df` = file delta
/// `dr` = rank delta
///
/// Examples:
/// north:      df =  0, dr =  1
/// south:      df =  0, dr = -1
/// east:       df =  1, dr =  0
/// west:       df = -1, dr =  0
/// northeast:  df =  1, dr =  1
/// northwest:  df = -1, dr =  1
fn ray_attacks(sq: Square, occupied: Bitboard, df: i8, dr: i8) -> Bitboard {
    let mut attacks = 0u64;

    let mut file = file_of(sq) as i8 + df;
    let mut rank = rank_of(sq) as i8 + dr;

    while (0..8).contains(&file) && (0..8).contains(&rank) {
        let to = square(file as u8, rank as u8);
        let to_mask = bit(to);

        attacks |= to_mask;

        if occupied & to_mask != 0 {
            // hit a piece
            break;
        }

        file += df;
        rank += dr;
    }

    attacks
}

pub fn rook_attacks(sq: Square, occupied: Bitboard) -> Bitboard {
    ray_attacks(sq, occupied, 0, 1)   // north
        | ray_attacks(sq, occupied, 0, -1)  // south
        | ray_attacks(sq, occupied, 1, 0)   // east
        | ray_attacks(sq, occupied, -1, 0) // west
}

#[inline]
pub fn bishop_attacks(sq: Square, occupied: Bitboard) -> Bitboard {
    ray_attacks(sq, occupied, 1, 1)    // northeast
        | ray_attacks(sq, occupied, -1, 1)  // northwest
        | ray_attacks(sq, occupied, 1, -1)  // southeast
        | ray_attacks(sq, occupied, -1, -1) // southwest
}

#[inline]
pub fn queen_attacks(sq: Square, occupied: Bitboard) -> Bitboard {
    rook_attacks(sq, occupied) | bishop_attacks(sq, occupied)
}

pub fn all_bishop_attacks(bishops: Bitboard, occupied: Bitboard) -> Bitboard {
    let mut attacks = 0u64;
    let mut b = bishops;

    while let Some(sq) = pop_lsb(&mut b) {
        attacks |= bishop_attacks(sq, occupied);
    }

    attacks
}

pub fn all_rook_attacks(rooks: Bitboard, occupied: Bitboard) -> Bitboard {
    let mut attacks = 0u64;
    let mut r = rooks;

    while let Some(sq) = pop_lsb(&mut r) {
        attacks |= rook_attacks(sq, occupied);
    }

    attacks
}

pub fn all_queen_attacks(queens: Bitboard, occupied: Bitboard) -> Bitboard {
    let mut attacks = 0u64;
    let mut q = queens;

    while let Some(sq) = pop_lsb(&mut q) {
        attacks |= queen_attacks(sq, occupied);
    }

    attacks
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bitboard::{bit, square};

    #[test]
    fn rook_attacks_from_d4_no_blockers() {
        let d4 = square(3, 3);
        let occupied = 0u64;

        let mut expected = 0u64;

        // d-file
        expected |= bit(square(3, 0));
        expected |= bit(square(3, 1));
        expected |= bit(square(3, 2));
        expected |= bit(square(3, 4));
        expected |= bit(square(3, 5));
        expected |= bit(square(3, 6));
        expected |= bit(square(3, 7));

        // rank 4
        expected |= bit(square(0, 3));
        expected |= bit(square(1, 3));
        expected |= bit(square(2, 3));
        expected |= bit(square(4, 3));
        expected |= bit(square(5, 3));
        expected |= bit(square(6, 3));
        expected |= bit(square(7, 3));

        assert_eq!(rook_attacks(d4, occupied), expected);
    }

    #[test]
    fn rook_attacks_from_d4_with_blockers() {
        let d4 = square(3, 3);

        let occupied = bit(square(3, 5)) | // d6
            bit(square(3, 1)) | // d2
            bit(square(1, 3)) | // b4
            bit(square(5, 3)); // f4

        let expected = bit(square(3, 4)) | // d5
            bit(square(3, 5)) | // d6 blocker

            bit(square(3, 2)) | // d3
            bit(square(3, 1)) | // d2 blocker

            bit(square(2, 3)) | // c4
            bit(square(1, 3)) | // b4 blocker

            bit(square(4, 3)) | // e4
            bit(square(5, 3)); // f4 blocker

        assert_eq!(rook_attacks(d4, occupied), expected);
    }

    #[test]
    fn bishop_attacks_from_d4_no_blockers() {
        let d4 = square(3, 3);
        let occupied = 0u64;

        let expected =
            // northeast
            bit(square(4, 4)) |
            bit(square(5, 5)) |
            bit(square(6, 6)) |
            bit(square(7, 7)) |

            // northwest
            bit(square(2, 4)) |
            bit(square(1, 5)) |
            bit(square(0, 6)) |

            // southeast
            bit(square(4, 2)) |
            bit(square(5, 1)) |
            bit(square(6, 0)) |

            // southwest
            bit(square(2, 2)) |
            bit(square(1, 1)) |
            bit(square(0, 0));

        assert_eq!(bishop_attacks(d4, occupied), expected);
    }

    #[test]
    fn bishop_attacks_from_d4_with_blockers() {
        let d4 = square(3, 3);

        let occupied = bit(square(5, 5)) | // f6
            bit(square(1, 5)) | // b6
            bit(square(5, 1)) | // f2
            bit(square(1, 1)); // b2

        let expected =
            // northeast
            bit(square(4, 4)) | // e5
            bit(square(5, 5)) | // f6 blocker

            // northwest
            bit(square(2, 4)) | // c5
            bit(square(1, 5)) | // b6 blocker

            // southeast
            bit(square(4, 2)) | // e3
            bit(square(5, 1)) | // f2 blocker

            // southwest
            bit(square(2, 2)) | // c3
            bit(square(1, 1)); // b2 blocker

        assert_eq!(bishop_attacks(d4, occupied), expected);
    }

    #[test]
    fn bishop_attacks_from_a1_no_blockers() {
        let a1 = square(0, 0);
        let occupied = 0u64;

        let expected = bit(square(1, 1))
            | bit(square(2, 2))
            | bit(square(3, 3))
            | bit(square(4, 4))
            | bit(square(5, 5))
            | bit(square(6, 6))
            | bit(square(7, 7));

        assert_eq!(bishop_attacks(a1, occupied), expected);
    }

    #[test]
    fn queen_attacks_equals_rook_or_bishop() {
        let d4 = square(3, 3);

        let occupied = bit(square(3, 5)) | bit(square(1, 1)) | bit(square(6, 3));

        assert_eq!(
            queen_attacks(d4, occupied),
            rook_attacks(d4, occupied) | bishop_attacks(d4, occupied)
        );
    }
}
