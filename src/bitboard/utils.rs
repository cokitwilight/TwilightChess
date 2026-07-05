use crate::bitboard::{Bitboard, Square};

// -------------------------
// File masks
// -------------------------

pub const FILE_A: Bitboard = 0x0101_0101_0101_0101;
pub const FILE_B: Bitboard = FILE_A << 1;
pub const FILE_C: Bitboard = FILE_A << 2;
pub const FILE_D: Bitboard = FILE_A << 3;
pub const FILE_E: Bitboard = FILE_A << 4;
pub const FILE_F: Bitboard = FILE_A << 5;
pub const FILE_G: Bitboard = FILE_A << 6;
pub const FILE_H: Bitboard = FILE_A << 7;

pub const NOT_FILE_A: Bitboard = !FILE_A;
pub const NOT_FILE_H: Bitboard = !FILE_H;
pub const NOT_FILE_AB: Bitboard = !(FILE_A | FILE_B);
pub const NOT_FILE_GH: Bitboard = !(FILE_G | FILE_H);

// -------------------------
// Rank masks
// -------------------------

pub const RANK_1: Bitboard = 0x0000_0000_0000_00FF;
pub const RANK_2: Bitboard = 0x0000_0000_0000_FF00;
pub const RANK_3: Bitboard = 0x0000_0000_00FF_0000;
pub const RANK_4: Bitboard = 0x0000_0000_FF00_0000;
pub const RANK_5: Bitboard = 0x0000_00FF_0000_0000;
pub const RANK_6: Bitboard = 0x0000_FF00_0000_0000;
pub const RANK_7: Bitboard = 0x00FF_0000_0000_0000;
pub const RANK_8: Bitboard = 0xFF00_0000_0000_0000;

// -------------------------
// Useful square constants
// -------------------------

pub const A1: Square = 0;
pub const B1: Square = 1;
pub const C1: Square = 2;
pub const D1: Square = 3;
pub const E1: Square = 4;
pub const F1: Square = 5;
pub const G1: Square = 6;
pub const H1: Square = 7;

pub const A8: Square = 56;
pub const B8: Square = 57;
pub const C8: Square = 58;
pub const D8: Square = 59;
pub const E8: Square = 60;
pub const F8: Square = 61;
pub const G8: Square = 62;
pub const H8: Square = 63;

#[inline]
pub fn bit(sq: Square) -> Bitboard {
    1u64 << sq
}

#[inline]
pub fn pop_lsb(bb: &mut Bitboard) -> Option<Square> {
    if *bb == 0 {
        return None;
    }

    let sq = bb.trailing_zeros() as Square;
    *bb &= *bb - 1;
    Some(sq)
}

#[inline]
pub fn file_of(sq: Square) -> u8 {
    sq % 8
}

pub fn file_mask(file: u8) -> Bitboard {
    match file {
        0 => FILE_A,
        1 => FILE_B,
        2 => FILE_C,
        3 => FILE_D,
        4 => FILE_E,
        5 => FILE_F,
        6 => FILE_G,
        7 => FILE_H,
        _ => {
            panic!("Larger file number > 8 in file_mask!");
        }
    }
}

#[inline]
pub fn rank_of(sq: Square) -> u8 {
    sq / 8
}

#[inline]
pub fn square(file: u8, rank: u8) -> Square {
    debug_assert!(file < 8);
    debug_assert!(rank < 8);
    rank * 8 + file
}

pub fn square_to_algebraic(sq: Square) -> String {
    let file = file_of(sq);
    let rank = rank_of(sq);

    let file_char = (b'a' + file) as char;
    let rank_char = (b'1' + rank) as char;

    format!("{}{}", file_char, rank_char)
}

pub fn square_from_algebraic(s: &str) -> Result<Square, String> {
    if s.len() != 2 {
        return Err(format!("Invalid square: {}", s));
    }

    let bytes = s.as_bytes();

    let file = match bytes[0] {
        b'a'..=b'h' => bytes[0] - b'a',
        _ => return Err(format!("Invalid file in square: {}", s)),
    };

    let rank = match bytes[1] {
        b'1'..=b'8' => bytes[1] - b'1',
        _ => return Err(format!("Invalid rank in square: {}", s)),
    };

    Ok(square(file, rank))
}

#[inline]
pub fn mirror_square_vertical(sq: Square) -> Square {
    // a1 <-> a8, b1 <-> b8, etc.
    sq ^ 56
}

pub fn print_bitboard(bb: Bitboard) {
    println!("  +-----------------+");

    for rank in (0..8).rev() {
        print!("{} |", rank + 1);

        for file in 0..8 {
            let sq = rank * 8 + file;
            let mask = 1u64 << sq;

            if bb & mask != 0 {
                print!(" 1");
            } else {
                print!(" .");
            }
        }

        println!(" |");
    }

    println!("  +-----------------+");
    println!("    a b c d e f g h");
}
