use crate::bitboard::{Bitboard, NOT_FILE_A, NOT_FILE_H, Square, bit, file_of};
use crate::types::Color;

// pub const PAWN_WEAKNESS_TABLE: [i32; 9] = [0, 0, 0, 5, 20, 35, 50, 75, 110];

// Stockfish 10 MobilityBonus, scaled by 100 / 208 and rounded to the nearest centipawn.
// The index is the piece's number of available squares.
pub const KNIGHT_MOVE_MG: [i32; 9] = [-30, -25, -6, -2, 1, 6, 11, 13, 16];
pub const KNIGHT_MOVE_EG: [i32; 9] = [-39, -27, -14, -7, 4, 7, 11, 13, 16];

pub const BISHOP_MOVE_MG: [i32; 14] = [-23, -10, 8, 13, 18, 25, 26, 30, 30, 33, 39, 39, 44, 47];
pub const BISHOP_MOVE_EG: [i32; 14] = [-28, -11, -1, 6, 12, 20, 26, 27, 31, 35, 38, 41, 42, 47];

pub const ROOK_MOVE_MG: [i32; 15] = [-28, -13, -7, -5, -2, -1, 4, 8, 14, 14, 15, 18, 22, 23, 28];
pub const ROOK_MOVE_EG: [i32; 15] = [-37, -9, 13, 26, 33, 39, 54, 57, 63, 68, 75, 79, 80, 81, 82];

pub const QUEEN_MOVE_MG: [i32; 28] = [
    -19, -10, 1, 1, 7, 11, 13, 20, 21, 23, 27, 29, 29, 32, 32, 34, 34, 35, 38, 42, 42, 48, 49, 49,
    51, 52, 54, 56,
];
pub const QUEEN_MOVE_EG: [i32; 28] = [
    -17, -7, 4, 9, 16, 26, 29, 35, 38, 44, 45, 50, 54, 58, 59, 61, 64, 65, 67, 69, 71, 80, 82, 84,
    88, 92, 99, 102,
];

pub const KING_DANGER_TABLE: [i32; 101] = [
    0, 0, 0, 0, 0, 1, 1, 1, 2, 2, 3, 3, 4, 5, 6, 7, 8, 9, 10, 12, 14, 16, 18, 20, 22, 24, 27, 30,
    33, 36, 39, 42, 46, 50, 54, 58, 62, 67, 72, 77, 82, 87, 93, 99, 105, 111, 118, 125, 132, 139,
    146, 154, 162, 170, 178, 187, 196, 205, 214, 224, 234, 244, 254, 265, 276, 287, 298, 310, 322,
    334, 346, 359, 372, 385, 398, 412, 426, 440, 454, 469, 484, 499, 514, 530, 546, 562, 578, 595,
    612, 629, 646, 664, 682, 700, 718, 737, 756, 775, 794, 814, 834,
];

// Stockfish 10 positional bonuses in centipawns (100 / 208), indexed from White's side.
pub const ALL_PSTS: [[i32; 64]; 12] = [
    [
        // Pawn MG (SF10 positional bonuses in centipawns, oriented for White).
        0, 0, 0, 0, 0, 0, 0, 0, // rank 1
        -5, 3, 3, 8, 8, 3, 3, -5, // rank 2
        -8, -1, 11, 11, 11, 11, -1, -8, // rank 3
        -7, -3, 10, 12, 12, 10, -3, -7, // rank 4
        -2, -1, 0, 6, 6, 0, -1, -2, // rank 5
        -5, -6, -1, 2, 2, -1, -6, -5, // rank 6
        -1, 10, -5, -1, -1, -5, 10, -1, // rank 7
        0, 0, 0, 0, 0, 0, 0, 0, // rank 8
    ],
    [
        // Knight MG
        -81, -46, -38, -38, -38, -38, -46, -81, // rank 1
        -38, -19, -12, -4, -4, -12, -19, -38, // rank 2
        -31, -10, 2, 9, 9, 2, -10, -31, // rank 3
        -13, 2, 20, 23, 23, 20, 2, -13, // rank 4
        -14, 6, 20, 25, 25, 20, 6, -14, // rank 5
        -5, 13, 30, 26, 26, 30, 13, -5, // rank 6
        -32, -10, 3, 18, 18, 3, -10, -32, // rank 7
        -96, -38, -25, -15, -15, -25, -38, -96, // rank 8
    ],
    [
        // Bishop MG
        -24, -3, -5, -16, -16, -5, -3, -24, // rank 1
        -12, 4, 7, 0, 0, 7, 4, -12, // rank 2
        -4, 11, -1, 6, 6, -1, 11, -4, // rank 3
        2, 4, 9, 19, 19, 9, 4, 2, // rank 4
        -4, 13, 6, 14, 14, 6, 13, -4, // rank 5
        -8, 7, -3, 3, 3, -3, 7, -8, // rank 6
        -9, -6, 3, -5, -5, 3, -6, -9, // rank 7
        -23, -3, -8, -14, -14, -8, -3, -23, // rank 8
    ],
    [
        // Rook MG
        -12, -7, -4, 0, 0, -4, -7, -12, // rank 1
        -9, -2, 0, 0, 0, 0, -2, -9, // rank 2
        -9, -5, 0, 0, 0, 0, -5, -9, // rank 3
        -10, -3, -2, -2, -2, -2, -3, -10, // rank 4
        -10, -6, 0, 2, 2, 0, -6, -10, // rank 5
        -11, -5, 0, 3, 3, 0, -5, -11, // rank 6
        -5, 4, 4, 6, 6, 4, 4, -5, // rank 7
        -12, -9, -5, 1, 1, -5, -9, -12, // rank 8
    ],
    [
        // Queen MG
        1, -2, -2, 2, 2, -2, -2, 1, // rank 1
        -1, 2, 4, 6, 6, 4, 2, -1, // rank 2
        -1, 3, 6, 3, 3, 6, 3, -1, // rank 3
        2, 2, 4, 4, 4, 4, 2, 2, // rank 4
        0, 7, 6, 2, 2, 6, 7, 0, // rank 5
        -2, 5, 3, 4, 4, 3, 5, -2, // rank 6
        -2, 3, 5, 4, 4, 5, 3, -2, // rank 7
        -1, -1, 0, -1, -1, 0, -1, -1, // rank 8
    ],
    [
        // King MG
        131, 156, 131, 91, 91, 131, 156, 131, // rank 1
        133, 147, 116, 88, 88, 116, 147, 133, // rank 2
        95, 122, 81, 58, 58, 81, 122, 95, // rank 3
        81, 92, 65, 52, 52, 65, 92, 81, // rank 4
        70, 85, 54, 33, 33, 54, 85, 70, // rank 5
        59, 76, 41, 17, 17, 41, 76, 59, // rank 6
        42, 58, 31, 12, 12, 31, 58, 42, // rank 7
        31, 42, 24, 0, 0, 24, 42, 31, // rank 8
    ],
    [
        // Pawn EG (oriented for White).
        0, 0, 0, 0, 0, 0, 0, 0, // rank 1
        -1, 0, 3, 1, 1, 3, 0, -1, // rank 2
        -1, 1, 3, 0, 0, 3, 1, -1, // rank 3
        3, -2, -4, 1, 1, -4, -2, 3, // rank 4
        6, 5, 0, -4, -4, 0, 5, 6, // rank 5
        8, 3, 0, 8, 8, 0, 3, 8, // rank 6
        0, -6, 3, 12, 12, 3, -6, 0, // rank 7
        0, 0, 0, 0, 0, 0, 0, 0, // rank 8
    ],
    [
        // Knight EG
        -50, -36, -22, -9, -9, -22, -36, -50, // rank 1
        -34, -27, -7, 3, 3, -7, -27, -34, // rank 2
        -18, -16, -2, 13, 13, -2, -16, -18, // rank 3
        -17, 0, 6, 16, 16, 6, 0, -17, // rank 4
        -20, -10, 2, 17, 17, 2, -10, -20, // rank 5
        -25, -18, -8, 9, 9, -8, -18, -25, // rank 6
        -31, -22, -18, 8, 8, -18, -22, -31, // rank 7
        -47, -43, -25, -8, -8, -25, -43, -47, // rank 8
    ],
    [
        // Bishop EG
        -28, -15, -18, -9, -9, -18, -15, -28, // rank 1
        -16, -4, -7, 2, 2, -7, -4, -16, // rank 2
        -11, 0, -1, 8, 8, -1, 0, -11, // rank 3
        -13, -1, -2, 8, 8, -2, -1, -13, // rank 4
        -13, -2, -3, 7, 7, -3, -2, -13, // rank 5
        -12, -1, 0, 6, 6, 0, -1, -12, // rank 6
        -16, -5, -6, 3, 3, -6, -5, -16, // rank 7
        -26, -15, -17, -8, -8, -17, -15, -26, // rank 8
    ],
    [
        // Rook EG
        0, 1, 0, 1, 1, 0, 1, 0, // rank 1
        -3, -2, -2, 0, 0, -2, -2, -3, // rank 2
        3, -3, 1, 1, 1, 1, -3, 3, // rank 3
        0, 2, -1, 0, 0, -1, 2, 0, // rank 4
        -3, 2, -2, -3, -3, -2, 2, -3, // rank 5
        1, 1, 0, 1, 1, 0, 1, 1, // rank 6
        0, 3, 5, 0, 0, 5, 3, 0, // rank 7
        3, 2, 3, 1, 1, 3, 2, 3, // rank 8
    ],
    [
        // Queen EG
        -33, -27, -23, -13, -13, -23, -27, -33, // rank 1
        -26, -15, -11, -2, -2, -11, -15, -26, // rank 2
        -19, -9, -4, 1, 1, -4, -9, -19, // rank 3
        -11, -1, 6, 12, 12, 6, -1, -11, // rank 4
        -14, -3, 4, 10, 10, 4, -3, -14, // rank 5
        -18, -9, -6, 0, 0, -6, -9, -18, // rank 6
        -24, -13, -12, -4, -4, -12, -13, -24, // rank 7
        -36, -25, -21, -17, -17, -21, -25, -36, // rank 8
    ],
    [
        // King EG
        0, 20, 38, 45, 45, 38, 20, 0, // rank 1
        27, 47, 66, 63, 63, 66, 47, 27, // rank 2
        41, 66, 79, 83, 83, 79, 66, 41, // rank 3
        50, 73, 81, 81, 81, 81, 73, 50, // rank 4
        47, 80, 95, 93, 93, 95, 80, 47, // rank 5
        42, 79, 84, 91, 91, 84, 79, 42, // rank 6
        19, 48, 62, 68, 68, 62, 48, 19, // rank 7
        2, 29, 36, 36, 36, 36, 29, 2, // rank 8
    ],
];

pub const KNIGHT_OUTPOSTS: [[Bitboard; 64]; 2] = generate_knight_outposts();
pub const PASSED_PAWNS: [[Bitboard; 64]; 2] = generate_passed_pawns();
pub const BACKWARDS_PAWNS: [[Bitboard; 64]; 2] = generate_backward_pawns();
pub const KING_PAWN_SHIELD: [[Bitboard; 64]; 2] = generate_king_shield();
pub const KING_PAWN_TWO_SHIELD: [[Bitboard; 64]; 2] = generate_king_two_shield();

/// Every square after the starting square along a direction, including the edge.
///
/// Direction components are indexed by adding one, so `-1`, `0`, and `1` map
/// directly to indices `0`, `1`, and `2`.
pub const RAY_MASKS: [[[Bitboard; 3]; 3]; 64] = generate_ray_masks();

/// Returns every square after `sq` along `(x, y)`, where `x` changes the file
/// and `y` changes the rank. The starting square is excluded and the edge
/// square is included.
///
/// `x` and `y` must each be in `-1..=1`, and they cannot both be zero.
#[inline]
pub const fn ray_mask(sq: Square, x: i8, y: i8) -> Bitboard {
    assert!(sq < 64, "square must be in 0..64");
    assert!(x >= -1 && x <= 1, "x must be in -1..=1");
    assert!(y >= -1 && y <= 1, "y must be in -1..=1");
    assert!(x != 0 || y != 0, "direction cannot be (0, 0)");

    RAY_MASKS[sq as usize][(x + 1) as usize][(y + 1) as usize]
}

const fn generate_ray_masks() -> [[[Bitboard; 3]; 3]; 64] {
    let mut table = [[[0; 3]; 3]; 64];
    let mut sq = 0;

    while sq < 64 {
        let mut x = -1;
        while x <= 1 {
            let mut y = -1;
            while y <= 1 {
                table[sq as usize][(x + 1) as usize][(y + 1) as usize] =
                    calculate_ray_mask(sq, x, y);
                y += 1;
            }
            x += 1;
        }
        sq += 1;
    }

    table
}

const fn calculate_ray_mask(sq: Square, x: i8, y: i8) -> Bitboard {
    if x == 0 && y == 0 {
        return 0;
    }

    let mut file = file_of(sq) as i8;
    let mut rank = (sq / 8) as i8;
    let mut ray = 0;

    loop {
        let next_file = file + x;
        let next_rank = rank + y;

        if next_file < 0 || next_file > 7 || next_rank < 0 || next_rank > 7 {
            return ray;
        }

        file = next_file;
        rank = next_rank;
        ray |= bit((rank * 8 + file) as Square);
    }
}

const fn generate_knight_outposts() -> [[Bitboard; 64]; 2] {
    let mut table = [[0; 64]; 2];
    let mut sq = 0;

    while sq < 64 {
        table[Color::White.idx()][sq as usize] = knight_outpost_mask(Color::White, sq);
        table[Color::Black.idx()][sq as usize] = knight_outpost_mask(Color::Black, sq);
        sq += 1;
    }

    table
}

const fn knight_outpost_mask(color: Color, sq: Square) -> Bitboard {
    let file = file_of(sq);
    let mut mask = 0;

    if file != 0 {
        mask |= bit(sq - 1);
    }
    if file != 7 {
        mask |= bit(sq + 1);
    }

    match color {
        Color::White => {
            mask <<= 8;
            mask |= mask << 8;
            mask |= mask << 16;
        }
        Color::Black => {
            mask >>= 8;
            mask |= mask >> 8;
            mask |= mask >> 16;
        }
    }

    mask
}

const fn generate_passed_pawns() -> [[Bitboard; 64]; 2] {
    let mut table = [[0; 64]; 2];
    let mut sq = 0;

    while sq < 64 {
        table[Color::White.idx()][sq as usize] = passed_pawn_mask(Color::White, sq);
        table[Color::Black.idx()][sq as usize] = passed_pawn_mask(Color::Black, sq);
        sq += 1;
    }

    table
}

const fn passed_pawn_mask(color: Color, sq: Square) -> Bitboard {
    let file = file_of(sq);
    let mut mask = bit(sq);

    if file != 0 {
        mask |= bit(sq - 1);
    }
    if file != 7 {
        mask |= bit(sq + 1);
    }

    match color {
        Color::White => {
            mask <<= 8;
            mask |= mask << 8;
            mask |= mask << 16;
            mask |= mask << 32;
        }
        Color::Black => {
            mask >>= 8;
            mask |= mask >> 8;
            mask |= mask >> 16;
            mask |= mask >> 32;
        }
    }

    mask
}

const fn generate_backward_pawns() -> [[Bitboard; 64]; 2] {
    let mut table = [[0; 64]; 2];
    let mut sq = 0;

    while sq < 64 {
        table[Color::White.idx()][sq as usize] = backward_pawn_mask(Color::White, sq);
        table[Color::Black.idx()][sq as usize] = backward_pawn_mask(Color::Black, sq);
        sq += 1;
    }

    table
}

const fn backward_pawn_mask(color: Color, sq: Square) -> Bitboard {
    let file = file_of(sq);
    let mut mask = 0;

    if file != 0 {
        mask |= bit(sq - 1);
    }
    if file != 7 {
        mask |= bit(sq + 1);
    }

    match color {
        Color::White => {
            mask >>= 8;
            mask |= mask >> 8;
            mask |= mask >> 16;
            mask |= mask >> 32;
        }
        Color::Black => {
            mask <<= 8;
            mask |= mask << 8;
            mask |= mask << 16;
            mask |= mask << 32;
        }
    }

    mask
}

const fn generate_king_shield() -> [[Bitboard; 64]; 2] {
    let mut table = [[0; 64]; 2];
    let mut sq = 0;

    while sq < 64 {
        table[Color::White.idx()][sq as usize] = king_shield_mask(Color::White, sq);
        table[Color::Black.idx()][sq as usize] = king_shield_mask(Color::Black, sq);
        sq += 1;
    }

    table
}

const fn generate_king_two_shield() -> [[Bitboard; 64]; 2] {
    let mut table = [[0; 64]; 2];
    let mut sq = 0;

    while sq < 64 {
        table[Color::White.idx()][sq as usize] = king_two_shield_mask(Color::White, sq);
        table[Color::Black.idx()][sq as usize] = king_two_shield_mask(Color::Black, sq);
        sq += 1;
    }

    table
}

const fn king_shield_mask(color: Color, sq: Square) -> Bitboard {
    let b = bit(sq);

    match color {
        Color::White => (b << 8) | ((b & NOT_FILE_H) << 9) | ((b & NOT_FILE_A) << 7),
        Color::Black => (b >> 8) | ((b & NOT_FILE_H) >> 7) | ((b & NOT_FILE_A) >> 9),
    }
}

const fn king_two_shield_mask(color: Color, sq: Square) -> Bitboard {
    let b = bit(sq);

    match color {
        Color::White => (b << 16) | ((b & NOT_FILE_H) << 17) | ((b & NOT_FILE_A) << 15),
        Color::Black => (b >> 16) | ((b & NOT_FILE_H) >> 15) | ((b & NOT_FILE_A) >> 17),
    }
}

#[cfg(test)]
mod tests {
    use super::ray_mask;
    use crate::bitboard::bit;

    #[test]
    fn ray_mask_finds_all_eight_rays() {
        let d4 = 27;

        assert_eq!(ray_mask(d4, -1, -1), bit(18) | bit(9) | bit(0));
        assert_eq!(ray_mask(d4, 0, -1), bit(19) | bit(11) | bit(3));
        assert_eq!(ray_mask(d4, 1, -1), bit(20) | bit(13) | bit(6));
        assert_eq!(ray_mask(d4, -1, 0), bit(26) | bit(25) | bit(24));
        assert_eq!(ray_mask(d4, 1, 0), bit(28) | bit(29) | bit(30) | bit(31));
        assert_eq!(ray_mask(d4, -1, 1), bit(34) | bit(41) | bit(48));
        assert_eq!(ray_mask(d4, 0, 1), bit(35) | bit(43) | bit(51) | bit(59));
        assert_eq!(ray_mask(d4, 1, 1), bit(36) | bit(45) | bit(54) | bit(63));
    }

    #[test]
    fn ray_mask_is_empty_when_direction_immediately_leaves_board() {
        assert_eq!(ray_mask(0, -1, 0), 0);
        assert_eq!(ray_mask(63, 1, 1), 0);
    }

    #[test]
    #[should_panic(expected = "direction cannot be (0, 0)")]
    fn ray_mask_rejects_zero_direction() {
        ray_mask(0, 0, 0);
    }

    #[test]
    #[should_panic(expected = "x must be in -1..=1")]
    fn ray_mask_rejects_out_of_range_components() {
        ray_mask(0, 2, 1);
    }
}
