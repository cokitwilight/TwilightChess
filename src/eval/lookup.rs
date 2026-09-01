use crate::bitboard::{Bitboard, NOT_FILE_A, NOT_FILE_H, Square, bit, file_of};
use crate::types::Color;

pub const PAWN_WEAKNESS_TABLE: [i32; 9] = [0, 0, 0, 5, 20, 35, 50, 75, 110];

pub const KNIGHT_MOVE: [i32; 9] = [-30, -10, -5, 0, 5, 20, 25, 35, 35];
pub const BISHOP_MOVE: [i32; 14] = [-30, -30, -20, -5, -5, 0, 10, 20, 30, 30, 30, 35, 35, 35];
pub const ROOK_MOVE: [i32; 15] = [-25, -15, -10, -5, 0, 0, 0, 10, 20, 25, 30, 30, 30, 30, 30];
pub const QUEEN_MOVE: [i32; 28] = [
    -130, -80, -40, -30, -25, -15, -5, 0, 0, 0, 0, 0, 0, 10, 15, 20, 25, 30, 35, 40, 45, 50, 50,
    50, 50, 50, 50, 50,
];

pub const KING_DANGER_TABLE: [i32; 101] = [
    0, 0, 0, 0, 0, 1, 1, 1, 2, 2, 3, 3, 4, 5, 6, 7, 8, 9, 10, 12, 14, 16, 18, 20, 22, 24, 27, 30,
    33, 36, 39, 42, 46, 50, 54, 58, 62, 67, 72, 77, 82, 87, 93, 99, 105, 111, 118, 125, 132, 139,
    146, 154, 162, 170, 178, 187, 196, 205, 214, 224, 234, 244, 254, 265, 276, 287, 298, 310, 322,
    334, 346, 359, 372, 385, 398, 412, 426, 440, 454, 469, 484, 499, 514, 530, 546, 562, 578, 595,
    612, 629, 646, 664, 682, 700, 718, 737, 756, 775, 794, 814, 834,
];

pub const ALL_PSTS: [[i32; 64]; 12] = [
    [
        // PAWN PST
        0, 0, 0, 0, 0, 0, 0, 0, 5, 10, 10, -20, -20, 10, 10, 5, 5, -5, -10, 0, 0, -10, -5, 5, 0, 10,
        20, 25, 25, 20, 10, 0, 5, 5, 10, 25, 25, 10, 5, 5, 10, 10, 20, 30, 30, 20, 10, 10, 30, 30,
        30, 30, 30, 30, 30, 30, 0, 0, 0, 0, 0, 0, 0, 0,
    ],
    [
        // KNIGHT PST
        -50, -40, -30, -30, -30, -30, -40, -50, -40, -20, 0, 5, 5, 0, -20, -40, -30, 5, 10, 15, 15,
        10, 5, -30, -30, 0, 15, 20, 20, 15, 0, -30, -30, 5, 15, 20, 20, 15, 5, -30, -30, 0, 10, 15,
        15, 10, 0, -30, -40, -20, 0, 0, 0, 0, -20, -40, -50, -40, -30, -30, -30, -30, -40, -50,
    ],
    [
        // BISHOP PST
        -20, -10, -10, -10, -10, -10, -10, -20, -10, 0, 0, 0, 0, 0, 0, -10, -10, 0, 5, 10, 10, 5, 0,
        -10, -10, 5, 5, 10, 10, 5, 5, -10, -10, 0, 10, 10, 10, 10, 0, -10, -10, 10, 10, 10, 10, 10,
        10, -10, -10, 5, 0, 0, 0, 0, 5, -10, -20, -10, -10, -10, -10, -10, -10, -20,
    ],
    [
        // ROOK PST
        0, 0, 0, 0, 0, 0, 0, 0, 5, 10, 10, 10, 10, 10, 10, 5, -5, 0, 0, 0, 0, 0, 0, -5, -5, 0, 0, 0,
        0, 0, 0, -5, -5, 0, 0, 0, 0, 0, 0, -5, -5, 0, 0, 0, 0, 0, 0, -5, -5, 0, 0, 0, 0, 0, 0, -5,
        0, 0, 0, 5, 5, 0, 0, 0,
    ],
    [
        // QUEEN PST
        -20, -10, -10, -5, -5, -10, -10, -20, -10, 0, 5, 0, 0, 0, 0, -10, -10, 5, 5, 5, 5, 5, 0,
        -10, 0, 0, 5, 5, 5, 5, 0, -5, -5, 0, 5, 5, 5, 5, 0, -5, -10, 0, 5, 5, 5, 5, 0, -10, -10, 0,
        0, 0, 0, 0, 0, -10, -20, -10, -10, -5, -5, -10, -10, -20,
    ],
    [
        // KING PST
        20, 45, 10, 0, 0, 10, 45, 20, 20, 20, 0, 0, 0, 0, 20, 20, -10, -20, -20, -20, -20, -20, -20,
        -10, -20, -30, -30, -40, -40, -30, -30, -20, -30, -40, -40, -50, -50, -40, -40, -30, -30,
        -40, -40, -50, -50, -40, -40, -30, -30, -40, -40, -50, -50, -40, -40, -30, -30, -40, -40,
        -50, -50, -40, -40, -30,
    ],
    [
        // PAWN ENDGAME PST
        0, 0, 0, 0, 0, 0, 0, 0, 5, 5, 5, 5, 5, 5, 5, 5, 15, 15, 15, 15, 15, 15, 15, 15, 30, 30, 30,
        30, 30, 30, 30, 30, 50, 50, 50, 50, 50, 50, 50, 50, 70, 70, 70, 70, 70, 70, 70, 70, 100,
        100, 100, 100, 100, 100, 100, 100, 0, 0, 0, 0, 0, 0, 0, 0,
    ],
    [
        // KNIGHT ENDGAME PST
        -60, -40, -30, -20, -20, -30, -40, -60, -40, -20, 0, 10, 10, 0, -20, -40, -30, 0, 20, 30,
        30, 20, 0, -30, -20, 10, 30, 40, 40, 30, 10, -20, -20, 10, 30, 40, 40, 30, 10, -20, -30, 0,
        20, 30, 30, 20, 0, -30, -40, -20, 0, 10, 10, 0, -20, -40, -60, -40, -30, -20, -20, -30,
        -40, -60,
    ],
    [
        // BISHOP ENDGAME PST
        -10, 0, 0, 0, 0, 0, 0, -10, 0, 10, 10, 10, 10, 10, 10, 0, 0, 10, 20, 20, 20, 20, 10, 0, 0,
        10, 20, 30, 30, 20, 10, 0, 0, 10, 20, 30, 30, 20, 10, 0, 0, 10, 20, 20, 20, 20, 10, 0, 0,
        10, 10, 10, 10, 10, 10, 0, -10, 0, 0, 0, 0, 0, 0, -10,
    ],
    [
        // ROOK ENDGAME PST
        0, 0, 0, 0, 0, 0, 0, 0, 5, 10, 10, 10, 10, 10, 10, 5, -5, 0, 0, 0, 0, 0, 0, -5, -5, 0, 0, 0,
        0, 0, 0, -5, -5, 0, 0, 0, 0, 0, 0, -5, -5, 0, 0, 0, 0, 0, 0, -5, -5, 0, 0, 0, 0, 0, 0, -5,
        0, 0, 0, 5, 5, 0, 0, 0,
    ],
    [
        // QUEEN ENDGAME PST
        -20, -10, -10, -5, -5, -10, -10, -20, -10, 0, 5, 0, 0, 0, 0, -10, -10, 5, 5, 5, 5, 5, 0,
        -10, 0, 0, 5, 5, 5, 5, 0, -5, -5, 0, 5, 5, 5, 5, 0, -5, -10, 0, 5, 5, 5, 5, 0, -10, -10, 0,
        0, 0, 0, 0, 0, -10, -20, -10, -10, -5, -5, -10, -10, -20,
    ],
    [
        // KING ENDGAME PST
        -50, -30, -10, 0, 0, -10, -30, -50, -30, -10, 20, 30, 30, 20, -10, -30, -10, 20, 30, 40, 40,
        30, 20, -10, 0, 30, 40, 50, 50, 40, 30, 0, 0, 30, 40, 50, 50, 40, 30, 0, -10, 20, 30, 40,
        40, 30, 20, -10, -30, -10, 20, 30, 30, 20, -10, -30, -50, -30, -10, 0, 0, -10, -30, -50,
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
