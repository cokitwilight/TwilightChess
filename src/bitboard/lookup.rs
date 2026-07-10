use std::sync::OnceLock;

use crate::bitboard::{
    Bitboard, FILE_A, FILE_H, NOT_FILE_A, NOT_FILE_AB, NOT_FILE_GH, NOT_FILE_H, Square,
};

pub struct AttackTables {
    pub knight: [Bitboard; 64],
    pub king: [Bitboard; 64],
    pub white_pawn: [Bitboard; 64],
    pub black_pawn: [Bitboard; 64],
}
impl Default for AttackTables {
    fn default() -> Self {
        Self::new()
    }
}

impl AttackTables {
    pub fn new() -> Self {
        let mut knight = [0u64; 64];
        let mut king = [0u64; 64];
        let mut white_pawn = [0u64; 64];
        let mut black_pawn = [0u64; 64];

        for sq in 0..64 {
            let sq = sq as Square;

            knight[sq as usize] = generate_knight_attacks(sq);
            king[sq as usize] = generate_king_attacks(sq);
            white_pawn[sq as usize] = generate_white_pawn_attacks(sq);
            black_pawn[sq as usize] = generate_black_pawn_attacks(sq);
        }

        Self {
            knight,
            king,
            white_pawn,
            black_pawn,
        }
    }
}

static ATTACK_TABLES: OnceLock<AttackTables> = OnceLock::new();

pub fn attack_tables() -> &'static AttackTables {
    ATTACK_TABLES.get_or_init(AttackTables::new)
}

fn generate_knight_attacks(sq: Square) -> Bitboard {
    let b = bit(sq);

    let mut attacks = 0u64;

    attacks |= (b & NOT_FILE_H) << 17;
    attacks |= (b & NOT_FILE_A) << 15;
    attacks |= (b & NOT_FILE_GH) << 10;
    attacks |= (b & NOT_FILE_AB) << 6;

    attacks |= (b & NOT_FILE_A) >> 17;
    attacks |= (b & NOT_FILE_H) >> 15;
    attacks |= (b & NOT_FILE_AB) >> 10;
    attacks |= (b & NOT_FILE_GH) >> 6;

    attacks
}

fn generate_king_attacks(sq: Square) -> Bitboard {
    let b = bit(sq);

    let mut attacks = 0u64;

    // Vertical
    attacks |= b << 8;
    attacks |= b >> 8;

    // Horizontal
    attacks |= (b & NOT_FILE_H) << 1;
    attacks |= (b & NOT_FILE_A) >> 1;

    // Diagonals upward
    attacks |= (b & NOT_FILE_H) << 9;
    attacks |= (b & NOT_FILE_A) << 7;

    // Diagonals downward
    attacks |= (b & NOT_FILE_H) >> 7;
    attacks |= (b & NOT_FILE_A) >> 9;

    attacks
}

fn generate_white_pawn_attacks(sq: Square) -> Bitboard {
    let b = bit(sq);

    let mut attacks = 0u64;

    attacks |= (b & NOT_FILE_A) << 7;
    attacks |= (b & NOT_FILE_H) << 9;

    attacks
}

fn generate_black_pawn_attacks(sq: Square) -> Bitboard {
    let b = bit(sq);

    let mut attacks = 0u64;

    attacks |= (b & NOT_FILE_A) >> 9;
    attacks |= (b & NOT_FILE_H) >> 7;

    attacks
}

// *************************
// **** BETWEEN[sq][sq] ****
// *************************

pub const BETWEEN: [[Bitboard; 64]; 64] = build_between_table();

pub const fn bit(sq: Square) -> Bitboard {
    1u64 << sq
}

const fn file_of_const(sq: Square) -> i32 {
    (sq & 7) as i32
}

const fn rank_of_const(sq: Square) -> i32 {
    (sq >> 3) as i32
}

const fn abs_i32(x: i32) -> i32 {
    if x < 0 { -x } else { x }
}

const fn sign_i32(x: i32) -> i32 {
    if x < 0 {
        -1
    } else if x > 0 {
        1
    } else {
        0
    }
}

const fn square_of_const(file: i32, rank: i32) -> Square {
    (rank as Square) * 8 + file as Square
}

pub const fn squares_aligned(a: Square, b: Square) -> bool {
    let af = file_of_const(a);
    let ar = rank_of_const(a);
    let bf = file_of_const(b);
    let br = rank_of_const(b);

    let df = bf - af;
    let dr = br - ar;

    df == 0 || dr == 0 || abs_i32(df) == abs_i32(dr)
}

const fn between_bb(a: Square, b: Square) -> Bitboard {
    if a == b || !squares_aligned(a, b) {
        return 0;
    }

    let af = file_of_const(a);
    let ar = rank_of_const(a);
    let bf = file_of_const(b);
    let br = rank_of_const(b);

    let step_file = sign_i32(bf - af);
    let step_rank = sign_i32(br - ar);

    let mut file = af + step_file;
    let mut rank = ar + step_rank;

    let mut mask = 0u64;

    while file != bf || rank != br {
        let sq = square_of_const(file, rank);
        mask |= bit(sq);

        file += step_file;
        rank += step_rank;
    }

    mask
}

const fn build_between_table() -> [[Bitboard; 64]; 64] {
    let mut table = [[0u64; 64]; 64];

    let mut from = 0u8;
    while from < 64 {
        let mut to = 0u8;
        while to < 64 {
            table[from as usize][to as usize] = between_bb(from, to);
            to += 1;
        }
        from += 1;
    }

    table
}
