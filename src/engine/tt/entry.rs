use crate::board::Move;
use crate::engine::MATE_THRESHOLD;
use std::num::NonZeroU8;

const IS_PV: u8 = 0x80; // PV is the left most bit
const IS_LOWER_BOUND: u8 = 0x20;
const IS_UPPER_BOUND: u8 = 0x40; // Upper bound is the second left most bit
const IS_EXACT: u8 = 0x60; // Exact is the third left most bit
const BOUNDS: u8 = 0x60; // Mask to extract bound type from flag

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TTFlag {
    Exact,
    LowerBound,
    UpperBound,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum TTNodeType {
    Main = 1,
    Quiescence = 0,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct TTEntry {
    pub key: u16,
    pub eval: i16,
    pub static_eval: i16,
    pub best_move: Option<Move>,
    pub depth: u8,
    pub flag: TTPackedFlag,
}
// #[derive(Clone, Copy, Debug)]
// pub struct TTSlot {
//     pub key: u16,
//     pub entry: TTEntry,
//     pub generation: u8,
// }
//

impl TTEntry {
    pub fn new(
        key: u16,
        is_pv: bool,
        bounds: TTFlag,
        generation: u8,
        depth: u8,
        eval: i16,
        static_eval: i16,
        best_move: Option<Move>,
    ) -> Self {
        Self {
            key,
            flag: TTPackedFlag::new(is_pv, bounds, generation),
            depth,
            eval,
            static_eval,
            best_move,
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub struct TTPackedFlag {
    flag: NonZeroU8,
}

impl TTPackedFlag {
    pub fn new(is_pv: bool, flag: TTFlag, mut generation: u8) -> Self {
        generation &= 0x1F; // Ensure generation is within 5 bits
        let mut packed_flag = 0u8;

        if is_pv {
            packed_flag |= IS_PV;
        }
        packed_flag |= match flag {
            TTFlag::Exact => IS_EXACT,
            TTFlag::LowerBound => IS_LOWER_BOUND,
            TTFlag::UpperBound => IS_UPPER_BOUND,
        };

        packed_flag |= generation;

        Self {
            flag: NonZeroU8::new(packed_flag).unwrap(),
        }
    }

    pub fn is_pv(&self) -> bool {
        self.flag.get() & IS_PV != 0
    }

    pub fn bound_type(&self) -> TTFlag {
        match self.flag.get() & BOUNDS {
            IS_EXACT => TTFlag::Exact,
            IS_LOWER_BOUND => TTFlag::LowerBound,
            IS_UPPER_BOUND => TTFlag::UpperBound,
            _ => panic!("Invalid bound type in TTPackedFlag"), // only possible with 00
        }
    }

    pub fn generation(&self) -> u8 {
        self.flag.get() & 0x1F
    }

    pub fn update_generation(&mut self, generation: u8) {
        let generation = generation & 0x1F; // Ensure generation is within 5 bits

        let mut packed_flag = self.flag.get() & !0x1F; // Clear the old generation bits
        packed_flag |= generation; // Set the new generation bits
        self.flag = NonZeroU8::new(packed_flag).unwrap();
    }
}

#[inline(always)]
pub fn score_to_tt(score: i32, ply: usize) -> i16 {
    let ply = ply as i32;

    if score >= MATE_THRESHOLD {
        // Winning mate: convert root-relative distance
        // into position-relative distance.
        (score + ply) as i16
    } else if score <= -MATE_THRESHOLD {
        // Losing mate.
        (score - ply) as i16
    } else {
        score as i16
    }
}

#[inline(always)]
pub fn score_from_tt(score: i32, ply: usize) -> i32 {
    let ply = ply as i32;

    if score >= MATE_THRESHOLD {
        score - ply
    } else if score <= -MATE_THRESHOLD {
        score + ply
    } else {
        score
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::MoveType;
    use crate::engine::{CHECKMATE_SCORE, MAX_PLY};
    use crate::types::PieceType;
    use std::mem::size_of;

    #[test]
    fn packed_entry_and_its_option_are_ten_bytes() {
        assert_eq!(size_of::<TTPackedFlag>(), 1);
        assert_eq!(size_of::<Option<TTPackedFlag>>(), 1);
        assert_eq!(size_of::<TTEntry>(), 10);
        assert_eq!(size_of::<Option<TTEntry>>(), 10);
    }

    #[test]
    fn packed_flags_round_trip_every_pv_bound_and_generation() {
        // Check the actual encoding as well as the accessors so overlapping
        // masks cannot accidentally round-trip through the same mistake.
        for is_pv in [false, true] {
            for (bound, bound_bits) in [
                (TTFlag::LowerBound, 0x20),
                (TTFlag::UpperBound, 0x40),
                (TTFlag::Exact, 0x60),
            ] {
                for generation in 0..=u8::MAX {
                    let flag = TTPackedFlag::new(is_pv, bound, generation);
                    let expected =
                        (if is_pv { 0x80 } else { 0 }) | bound_bits | (generation & 0x1f);

                    assert_eq!(flag.flag.get(), expected);
                    assert_eq!(flag.is_pv(), is_pv);
                    assert_eq!(flag.bound_type(), bound);
                    assert_eq!(flag.generation(), generation % 32);
                }
            }
        }
    }

    #[test]
    fn updating_generation_preserves_pv_and_bound_and_clears_old_bits() {
        for is_pv in [false, true] {
            for bound in [TTFlag::Exact, TTFlag::LowerBound, TTFlag::UpperBound] {
                for old_generation in 0..32 {
                    for generation in 0..=u8::MAX {
                        let mut flag = TTPackedFlag::new(is_pv, bound, old_generation);
                        let old_metadata = flag.flag.get() & 0xe0;
                        flag.update_generation(generation);

                        assert_eq!(flag.flag.get(), old_metadata | (generation & 0x1f));
                        assert_eq!(flag.is_pv(), is_pv);
                        assert_eq!(flag.bound_type(), bound);
                        assert_eq!(flag.generation(), generation % 32);
                    }
                }
            }
        }
    }

    #[test]
    fn constructor_preserves_all_fields_and_optional_moves() {
        let promotion = Move::new(54, 63, MoveType::Capture, Some(PieceType::Knight));
        for best_move in [None, Some(Move::NULL), Some(promotion)] {
            let entry = TTEntry::new(
                u16::MAX,
                true,
                TTFlag::UpperBound,
                u8::MAX,
                u8::MAX,
                i16::MAX,
                i16::MIN,
                best_move,
            );

            assert_eq!(entry.key, u16::MAX);
            assert!(entry.flag.is_pv());
            assert_eq!(entry.flag.bound_type(), TTFlag::UpperBound);
            assert_eq!(entry.flag.generation(), 31);
            assert_eq!(entry.depth, u8::MAX);
            assert_eq!(entry.eval, i16::MAX);
            assert_eq!(entry.static_eval, i16::MIN);
            assert_eq!(entry.best_move, best_move);
        }
    }

    #[test]
    fn ordinary_scores_are_independent_of_ply() {
        for score in [-MATE_THRESHOLD + 1, -123, 0, 123, MATE_THRESHOLD - 1] {
            for ply in [0, 1, MAX_PLY - 1] {
                assert_eq!(i32::from(score_to_tt(score, ply)), score);
                assert_eq!(score_from_tt(score, ply), score);
            }
        }
    }

    #[test]
    fn mate_scores_round_trip_and_adjust_to_the_probe_ply() {
        for score in [
            -CHECKMATE_SCORE,
            -MATE_THRESHOLD,
            MATE_THRESHOLD,
            CHECKMATE_SCORE,
        ] {
            for store_ply in [0, 1, MAX_PLY - 1] {
                let stored = i32::from(score_to_tt(score, store_ply));
                let direction = score.signum();
                assert_eq!(stored, score + direction * store_ply as i32);
                assert_eq!(score_from_tt(stored, store_ply), score);

                for probe_ply in [0, 1, MAX_PLY - 1] {
                    assert_eq!(
                        score_from_tt(stored, probe_ply),
                        score + direction * (store_ply as i32 - probe_ply as i32),
                    );
                }
            }
        }
    }
}
