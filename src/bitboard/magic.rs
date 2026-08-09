use rand::Rng;
use rand::RngExt;

use std::sync::OnceLock;

use crate::bitboard::rays::bishop_attacks_slow;
use crate::bitboard::rays::rook_attacks_slow;
use crate::bitboard::{
    Bitboard, FILE_MASKS, RANK_MASKS, Square, bit, file_of, pop_lsb, rank_of, square,
};

static BISHOP_MAGIC_TABLE: OnceLock<[MagicEntry; 64]> = OnceLock::new();
static BISHOP_ATTACK_TABLE: OnceLock<Box<[Bitboard]>> = OnceLock::new();

static ROOK_MAGIC_TABLE: OnceLock<[MagicEntry; 64]> = OnceLock::new();
static ROOK_ATTACK_TABLE: OnceLock<Box<[Bitboard]>> = OnceLock::new();

pub fn bishop_magics() -> &'static [MagicEntry; 64] {
    BISHOP_MAGIC_TABLE.get_or_init(build_bishop_magic)
}

pub fn bishop_attack_table() -> &'static Box<[Bitboard]> {
    BISHOP_ATTACK_TABLE.get_or_init(|| {
        let magics = bishop_magics();
        build_bishop_attack_table(magics)
    })
}

pub fn rook_magics() -> &'static [MagicEntry; 64] {
    ROOK_MAGIC_TABLE.get_or_init(build_rook_magic)
}

pub fn rook_attack_table() -> &'static Box<[Bitboard]> {
    ROOK_ATTACK_TABLE.get_or_init(|| {
        let magics = rook_magics();
        build_rook_attack_table(magics)
    })
}

const ROOK_ATTACK_SIZE: usize = 102400;

const BISHOP_ATTACK_SIZE: usize = 5248;

const BISHOP_MAGIC_NUMBERS: [u64; 64] = [
    0x0818024428062100,
    0x044212A404008000,
    0x0088020400248700,
    0x0804040089140004,
    0x0402021022020012,
    0x0000900C608A0900,
    0x0510840402C08100,
    0x0002004108215004,
    0x1108409034014242,
    0x8000900118408880,
    0x0110412101050100,
    0x80800C0405900000,
    0x2400091140000000,
    0x00001208240400A0,
    0x2000028414124004,
    0x040C420042021040,
    0x0008021260084880,
    0x0802020832184200,
    0x0910012804401220,
    0x21C9082801410000,
    0x4002004422010102,
    0x2211040201008200,
    0x2440604082101000,
    0x8A41000024012410,
    0x0002880220600400,
    0x0004108102620811,
    0xA008040008102021,
    0x00260090080040C0,
    0x0001001001004002,
    0x180C010000E0A000,
    0x0008008302208440,
    0x023110C006022482,
    0x8401202004100402,
    0x0081241000208142,
    0x0230222800040800,
    0x5000040400080210,
    0x0080410040040040,
    0x0010260020021005,
    0x80104202018080A0,
    0x0824010026020080,
    0x0001100210B0A008,
    0x02C042021000A041,
    0xC080140201010800,
    0x8208084200802800,
    0x0044080104003042,
    0x6102101020802108,
    0x0005100202063441,
    0x4001420082080108,
    0x2080484404210208,
    0x2000411808030010,
    0x0205002084100800,
    0x0000080042020820,
    0x002420C082820054,
    0x0008420224010020,
    0x192021410A008200,
    0x0230110825004240,
    0x0800210410010810,
    0x4810002404024800,
    0x4100020225080810,
    0x200A80001420880A,
    0x4A440000881B0400,
    0x0800404002041448,
    0x0010448918080082,
    0x0228209682020420,
];

const ROOK_MAGIC_NUMBERS: [u64; 64] = [
    0xC080022040081280,
    0x2840400010002000,
    0x1080081000200080,
    0x1100040902100020,
    0x0A00020044502008,
    0x26000200C4081001,
    0x1480020041000080,
    0x0200044021008C06,
    0x0008800881644008,
    0x8104404000201000,
    0x8002001200208040,
    0xC042000A00204014,
    0x0404800400804800,
    0x021200304A001488,
    0x420D004402001900,
    0x0081000053000D8A,
    0x4080014001402001,
    0x4090004010402000,
    0x1400110041002000,
    0x210821000D001000,
    0x0040808004000801,
    0x0210808002000400,
    0x0100808002000100,
    0xC00026002108C084,
    0x0080400280008028,
    0x0000500040002004,
    0x0400420200201080,
    0x0210018480080010,
    0x1084110100080004,
    0x2040020080800400,
    0x0001000100040200,
    0x0210008200006104,
    0x0000400028800880,
    0x0000201000400041,
    0x0201001041002000,
    0x0400800804801000,
    0x0451880071000500,
    0xD022000802000410,
    0x000801C804000210,
    0x0208008402000041,
    0x04A0400080208002,
    0x0040081000212000,
    0x0104208200460010,
    0x4055000890050020,
    0x1020080005010010,
    0x0000041040080120,
    0x0800081082040001,
    0x42000070810A0004,
    0x880380030068C100,
    0x4002804000201280,
    0x0020801000200080,
    0x8108100100082100,
    0x8001040048008280,
    0x4002001008040200,
    0x0008102201882400,
    0x0000110040840200,
    0x428110A082004302,
    0x0004208900401202,
    0x0001410420010811,
    0x0C81001000042009,
    0x4009000210480045,
    0x241300080400120D,
    0x4200102800820104,
    0x2000004088240502,
];

#[derive(Clone, Copy)]
pub struct MagicEntry {
    pub mask: Bitboard,
    pub magic: u64,
    pub shift: u8,
    pub offset: usize,
}

pub fn build_bishop_attack_table(magic: &[MagicEntry; 64]) -> Box<[Bitboard]> {
    let mut table = vec![0u64; BISHOP_ATTACK_SIZE].into_boxed_slice();

    for sq in 0u8..64 {
        let entry = magic[sq as usize];

        let relevant_bits = (64 - entry.shift) as usize;

        let occupancy_count = 1usize << relevant_bits;

        for i in 0..occupancy_count {
            let occupancy = set_occupancy(i, entry.mask);

            let attacks = bishop_attacks_slow(sq, occupancy);

            let magic_index = (occupancy.wrapping_mul(entry.magic) >> entry.shift) as usize;

            table[entry.offset + magic_index] = attacks;
        }
    }

    table
}

pub fn build_rook_attack_table(magic: &[MagicEntry; 64]) -> Box<[Bitboard]> {
    let mut table = vec![0u64; ROOK_ATTACK_SIZE].into_boxed_slice();

    for sq in 0u8..64 {
        let entry = magic[sq as usize];

        let relevant_bits = (64 - entry.shift) as usize;

        let occupancy_count = 1usize << relevant_bits;

        for i in 0..occupancy_count {
            let occupancy = set_occupancy(i, entry.mask);

            let attacks = rook_attacks_slow(sq, occupancy);

            let magic_index = (occupancy.wrapping_mul(entry.magic) >> entry.shift) as usize;

            table[entry.offset + magic_index] = attacks;
        }
    }

    table
}

pub fn build_bishop_magic() -> [MagicEntry; 64] {
    let mut bishop_magic: [MagicEntry; 64] = [MagicEntry {
        mask: 0,
        magic: 0,
        shift: 0,
        offset: 0,
    }; 64];

    let mut offset = 0usize;

    for i in 0..64 {
        let mask = relevant_bishop_mask(i);
        let bits = mask.count_ones();

        let entry = MagicEntry {
            mask,
            magic: BISHOP_MAGIC_NUMBERS[i as usize],
            shift: (64 - bits) as u8,
            offset,
        };
        bishop_magic[i as usize] = entry;

        offset += 1usize << bits;
    }

    bishop_magic
}

pub fn build_rook_magic() -> [MagicEntry; 64] {
    let mut rook_magic: [MagicEntry; 64] = [MagicEntry {
        mask: 0,
        magic: 0,
        shift: 0,
        offset: 0,
    }; 64];

    let mut offset = 0usize;

    for i in 0..64 {
        let mask = relevant_rook_mask(i);
        let bits = mask.count_ones();

        let entry = MagicEntry {
            mask,
            magic: ROOK_MAGIC_NUMBERS[i as usize],
            shift: (64 - bits) as u8,
            offset,
        };
        rook_magic[i as usize] = entry;

        offset += 1usize << bits;
    }

    rook_magic
}

pub fn compute_bishop_magic() {
    let mut rng = rand::rng();

    for i in 0u8..64 {
        let mask = relevant_bishop_mask(i);
        let relevant_bits = mask.count_ones() as usize;

        let (occupancies, attacks) = generate_bishop_data(i);

        let magic = find_magic(mask, relevant_bits, &occupancies, &attacks, &mut rng);

        println!("0x{:016X},", magic);
    }
}

pub fn compute_rook_magic() {
    let mut rng = rand::rng();

    for i in 0u8..64 {
        let mask = relevant_rook_mask(i);
        let relevant_bits = mask.count_ones() as usize;

        let (occupancies, attacks) = generate_rook_data(i);

        let magic = find_magic(mask, relevant_bits, &occupancies, &attacks, &mut rng);

        println!("0x{:016X},", magic);
    }
}

fn find_magic<R: Rng + ?Sized>(
    mask: Bitboard,
    relevant_bits: usize,
    occupancies: &[Bitboard],
    attacks: &[Bitboard],
    rng: &mut R,
) -> u64 {
    loop {
        let magic = random_magic_candidate(rng);

        // Cheap heuristic
        let high_bits = mask.wrapping_mul(magic) & 0xFF00_0000_0000_0000;

        if high_bits.count_ones() < 6 {
            continue;
        }

        if test_magic(magic, relevant_bits, occupancies, attacks) {
            return magic;
        }
    }
}

fn test_magic(magic: u64, relevant_bits: usize, occupancies: &[u64], attacks: &[u64]) -> bool {
    let table_size = 1usize << relevant_bits;
    let mut used = vec![None; table_size];

    for i in 0..occupancies.len() {
        let index = (occupancies[i].wrapping_mul(magic) >> (64 - relevant_bits)) as usize;

        match used[index] {
            None => {
                used[index] = Some(attacks[i]);
            }

            Some(existing) if existing == attacks[i] => {
                // Constructive collision. Fine.
            }

            Some(_) => {
                return false;
            }
        }
    }

    true
}

fn random_magic_candidate<R: Rng + ?Sized>(rng: &mut R) -> u64 {
    rng.random::<u64>() & rng.random::<u64>() & rng.random::<u64>()
}

fn generate_bishop_data(sq: Square) -> (Vec<Bitboard>, Vec<Bitboard>) {
    let mask = relevant_bishop_mask(sq);

    let relevant_bits = mask.count_ones();

    let occupancy_count = 1usize << relevant_bits;

    let mut occupancies = Vec::with_capacity(occupancy_count);

    let mut attacks = Vec::with_capacity(occupancy_count);

    for index in 0..occupancy_count {
        let occupancy = set_occupancy(index, mask);

        let attack = bishop_attacks_slow(sq, occupancy);

        occupancies.push(occupancy);
        attacks.push(attack);
    }

    (occupancies, attacks)
}

fn generate_rook_data(sq: Square) -> (Vec<Bitboard>, Vec<Bitboard>) {
    let mask = relevant_rook_mask(sq);

    let relevant_bits = mask.count_ones();

    let occupancy_count = 1usize << relevant_bits;

    let mut occupancies = Vec::with_capacity(occupancy_count);

    let mut attacks = Vec::with_capacity(occupancy_count);

    for index in 0..occupancy_count {
        let occupancy = set_occupancy(index, mask);

        let attack = rook_attacks_slow(sq, occupancy);

        occupancies.push(occupancy);
        attacks.push(attack);
    }

    (occupancies, attacks)
}

fn set_occupancy(index: usize, mut mask: Bitboard) -> Bitboard {
    let mut occupancy = 0;
    let mut bit_index = 0;

    while mask != 0 {
        let sq = pop_lsb(&mut mask).unwrap();

        if index & (1usize << bit_index) != 0 {
            occupancy |= bit(sq);
        }

        bit_index += 1;
    }

    occupancy
}

pub fn relevant_bishop_mask(sq: Square) -> Bitboard {
    let file = file_of(sq) as i8;
    let rank = rank_of(sq) as i8;

    let mut bishop_mask = 0u64;

    // Northeast
    let mut f = file + 1;
    let mut r = rank + 1;

    while f < 7 && r < 7 {
        bishop_mask |= bit(square(f as u8, r as u8));

        f += 1;
        r += 1;
    }

    // Northwest
    let mut f = file - 1;
    let mut r = rank + 1;

    while f > 0 && r < 7 {
        bishop_mask |= bit(square(f as u8, r as u8));

        f -= 1;
        r += 1;
    }

    // Southeast
    let mut f = file + 1;
    let mut r = rank - 1;

    while f < 7 && r > 0 {
        bishop_mask |= bit(square(f as u8, r as u8));

        f += 1;
        r -= 1;
    }

    // Southwest
    let mut f = file - 1;
    let mut r = rank - 1;

    while f > 0 && r > 0 {
        bishop_mask |= bit(square(f as u8, r as u8));

        f -= 1;
        r -= 1;
    }

    bishop_mask
}

pub fn relevant_rook_mask(sq: Square) -> Bitboard {
    // goes out in each direction and fills in the rook attacking squares as if there was no other pieces
    // removes the outer edge afterwards. Does not include the square.
    let file = file_of(sq);
    let rank = rank_of(sq);

    let mut rook_mask = (FILE_MASKS[file as usize] | RANK_MASKS[rank as usize]) & !bit(sq);

    let mut edge_mask = 0u64;

    if file != 0 {
        edge_mask |= FILE_MASKS[0]; // file A
    }

    if file != 7 {
        edge_mask |= FILE_MASKS[7]; // file H
    }

    if rank != 0 {
        edge_mask |= RANK_MASKS[0]; // rank 1
    }

    if rank != 7 {
        edge_mask |= RANK_MASKS[7]; // rank 8
    }

    rook_mask &= !edge_mask;

    rook_mask
}

#[test]
fn test_all_bishop_magics_are_valid() {
    let bishop_magics = bishop_magics();

    for sq in 0u8..64 {
        let entry = bishop_magics[sq as usize];

        let relevant_bits = entry.mask.count_ones() as usize;

        let (occupancies, attacks) = generate_bishop_data(sq);

        assert!(
            test_magic(entry.magic, relevant_bits, &occupancies, &attacks,),
            "Invalid bishop magic for square {}",
            sq
        );
    }
}

#[test]
fn test_bishop_magic_attacks_against_slow() {
    let bishop_magics = bishop_magics();
    let bishop_attacks = bishop_attack_table();

    println!("Start.");

    let mut count = 0usize;

    for sq in 0u8..64 {
        println!("{}. Current count: {}", sq, count);
        let entry = bishop_magics[sq as usize];

        let relevant_bits = entry.mask.count_ones() as usize;

        let occupancy_count = 1usize << relevant_bits;

        for occupancy_index in 0..occupancy_count {
            let occupancy = set_occupancy(occupancy_index, entry.mask);

            let magic_index = (occupancy.wrapping_mul(entry.magic) >> entry.shift) as usize;

            let expected = bishop_attacks_slow(sq, occupancy);

            let actual = bishop_attacks[entry.offset + magic_index];

            count += 1;

            assert_eq!(
                actual, expected,
                "Bishop magic mismatch: sq={}, occupancy={:#018X}, index={}",
                sq, occupancy, magic_index,
            );
        }
    }

    println!("End: Count: {}", count);
}

#[test]
fn test_all_rook_magics_are_valid() {
    let rook_magics = rook_magics();

    for sq in 0u8..64 {
        let entry = rook_magics[sq as usize];

        let relevant_bits = entry.mask.count_ones() as usize;

        let (occupancies, attacks) = generate_rook_data(sq);

        assert!(
            test_magic(entry.magic, relevant_bits, &occupancies, &attacks,),
            "Invalid rook magic for square {}",
            sq
        );
    }
}

#[test]
fn test_rook_magic_attacks_against_slow() {
    let rook_magics = rook_magics();
    let rook_attacks = rook_attack_table();

    println!("Start.");

    let mut count = 0usize;

    for sq in 0u8..64 {
        println!("{}. Current count: {}", sq, count);
        let entry = rook_magics[sq as usize];

        let relevant_bits = entry.mask.count_ones() as usize;

        let occupancy_count = 1usize << relevant_bits;

        for occupancy_index in 0..occupancy_count {
            let occupancy = set_occupancy(occupancy_index, entry.mask);

            let magic_index = (occupancy.wrapping_mul(entry.magic) >> entry.shift) as usize;

            let expected = rook_attacks_slow(sq, occupancy);

            let actual = rook_attacks[entry.offset + magic_index];

            count += 1;

            assert_eq!(
                actual, expected,
                "Rook magic mismatch: sq={}, occupancy={:#018X}, index={}",
                sq, occupancy, magic_index,
            );
        }
    }

    println!("End: Count: {}", count);
}
