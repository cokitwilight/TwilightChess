use crate::bitboard::Square;
use crate::types::PieceType;
use std::cmp::Reverse;
use std::num::NonZeroU16;

pub const MAX_MOVES: usize = 256;

// Bits 0..=5 store the source, bits 6..=11 the destination, and bits 12..=15 the flags.
const SQUARE_MASK: u16 = 0x003F;
const TO_SHIFT: u32 = 6;

// Every supported flag encoding is nonzero, reserving zero for Option<Move>::None.
const NORMAL: u16 = 0x1000;
const EN_PASSANT: u16 = 0x2000;
const CASTLE: u16 = 0x3000;
const PROMOTION_KNIGHT: u16 = 0x4000;
const PROMOTION_BISHOP: u16 = 0x5000;
const PROMOTION_ROOK: u16 = 0x6000;
const PROMOTION_QUEEN: u16 = 0x7000;

const IS_PROMOTION: u16 = 0x4000;
const CAPTURE: u16 = 0x8000;
const CODE_MASK: u16 = 0x7000;
const FLAGS_MASK: u16 = CODE_MASK | CAPTURE;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum MoveType {
    Normal = 0,
    Capture = 1,
    EnPassant = 2,
    Castle = 3,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(transparent)]
pub struct Move {
    mv: NonZeroU16,
}

impl Move {
    pub const NULL: Move = Move {
        mv: NonZeroU16::new(NORMAL).expect("Normal move flags must be nonzero"),
    };

    pub fn new(from: Square, to: Square, kind: MoveType, promotion: Option<PieceType>) -> Self {
        let from = from as u16;
        let to = (to as u16) << TO_SHIFT;
        let kind = match kind {
            MoveType::Normal => NORMAL,
            MoveType::Capture => CAPTURE,
            MoveType::EnPassant => EN_PASSANT,
            MoveType::Castle => CASTLE,
        };
        let promotion = match promotion {
            Some(PieceType::Knight) => PROMOTION_KNIGHT,
            Some(PieceType::Bishop) => PROMOTION_BISHOP,
            Some(PieceType::Rook) => PROMOTION_ROOK,
            Some(PieceType::Queen) => PROMOTION_QUEEN,
            _ => 0,
        };

        // Promotion codes replace the normal code; only the capture bit is shared.
        let flags = if promotion != 0 {
            promotion | (kind & CAPTURE)
        } else {
            kind
        };

        Self {
            mv: NonZeroU16::new(from | to | flags).expect("Move flags must be nonzero"),
        }
    }

    pub fn is_capture(self) -> bool {
        return self.mv.get() & CAPTURE == CAPTURE || self.mv.get() & CODE_MASK == EN_PASSANT;
    }

    pub fn is_promotion(self) -> bool {
        self.mv.get() & IS_PROMOTION != 0
    }

    pub fn is_castle(self) -> bool {
        self.mv.get() & FLAGS_MASK == CASTLE
    }

    pub fn from(self) -> Square {
        (self.mv.get() & SQUARE_MASK) as Square
    }
    pub fn to(self) -> Square {
        ((self.mv.get() >> TO_SHIFT) & SQUARE_MASK) as Square
    }
    pub fn kind(self) -> MoveType {
        match self.mv.get() & FLAGS_MASK {
            NORMAL => MoveType::Normal,
            CAPTURE => MoveType::Capture,
            EN_PASSANT => MoveType::EnPassant,
            CASTLE => MoveType::Castle,
            _ => {
                if self.mv.get() & CAPTURE == CAPTURE {
                    MoveType::Capture // capture promotion
                } else {
                    MoveType::Normal // this is a normal promotion
                }
            }
        }
    }
    pub fn promotion(self) -> Option<PieceType> {
        let promo = Option::from(match self.mv.get() & CODE_MASK {
            PROMOTION_KNIGHT => PieceType::Knight,
            PROMOTION_BISHOP => PieceType::Bishop,
            PROMOTION_ROOK => PieceType::Rook,
            PROMOTION_QUEEN => PieceType::Queen,
            _ => return None,
        });

        return promo;
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct MoveList {
    moves: [Move; MAX_MOVES],
    len: usize,
}

impl MoveList {
    pub fn new() -> Self {
        Self {
            moves: [Move::NULL; MAX_MOVES],
            len: 0,
        }
    }

    pub fn push(&mut self, mv: Move) {
        assert!(
            self.len < self.moves.len(),
            "MoveList overflow: more than {} moves generated",
            MAX_MOVES
        );

        self.moves[self.len] = mv;
        self.len += 1;
    }

    pub fn pop(&mut self) -> Option<Move> {
        if self.len == 0 {
            return None;
        }

        let mv = self.moves[self.len - 1];
        self.len -= 1;
        Some(mv)
    }

    pub fn swap_remove(&mut self, index: usize) -> Move {
        if self.is_empty() || index >= self.len() {
            panic!("Invalid index in swap_remove in MoveList");
        }
        let removed = self.moves[index];
        self.moves[index] = self.moves[self.len() - 1];
        self.len -= 1;

        removed
    }

    pub fn get(&self, index: usize) -> Move {
        self.moves[index]
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn clear(&mut self) {
        self.len = 0;
    }

    pub fn as_slice(&self) -> &[Move] {
        &self.moves[..self.len]
    }

    pub fn iter(&self) -> std::slice::Iter<'_, Move> {
        self.as_slice().iter()
    }

    pub fn sort_by_score<F>(&mut self, mut score: F)
    where
        F: FnMut(Move) -> i32,
    {
        self.moves[..self.len].sort_by_key(|mv| Reverse(score(*mv)));
    }
}

impl Default for MoveList {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packed_moves_round_trip_all_squares_and_supported_flags() {
        let mut cases = vec![
            (MoveType::Normal, None),
            (MoveType::Capture, None),
            (MoveType::EnPassant, None),
            (MoveType::Castle, None),
        ];
        for piece in [
            PieceType::Knight,
            PieceType::Bishop,
            PieceType::Rook,
            PieceType::Queen,
        ] {
            cases.push((MoveType::Normal, Some(piece)));
            cases.push((MoveType::Capture, Some(piece)));
        }

        let mut seen = [false; 1 << 16];
        for from in 0..64 {
            for to in 0..64 {
                for &(kind, promotion) in &cases {
                    let mv = Move::new(from, to, kind, promotion);
                    assert_eq!(mv.mv.get() & 0x0FFF, (u16::from(to) << 6) | u16::from(from));
                    assert_eq!(mv.from(), from);
                    assert_eq!(mv.to(), to);
                    assert_eq!(mv.kind(), kind);
                    assert_eq!(mv.promotion(), promotion);
                    assert_eq!(
                        mv.is_capture(),
                        matches!(kind, MoveType::Capture | MoveType::EnPassant)
                    );
                    assert_eq!(mv.is_promotion(), promotion.is_some());
                    assert_eq!(mv.is_castle(), kind == MoveType::Castle, "{mv:?}");
                    assert_ne!(mv.mv.get(), 0);
                    assert!(!seen[mv.mv.get() as usize], "Duplicate encoding: {mv:?}");
                    seen[mv.mv.get() as usize] = true;
                }
            }
        }
        assert!(!seen[0], "Zero must remain reserved for Option<Move>::None");
    }

    #[test]
    fn packed_move_and_option_are_two_bytes_and_preserve_null() {
        assert_eq!(std::mem::size_of::<Move>(), 2);
        assert_eq!(std::mem::size_of::<Option<Move>>(), 2);
        assert_ne!(Move::NULL.mv.get(), 0);
        assert_eq!(Move::NULL, Move::new(0, 0, MoveType::Normal, None));
        assert_eq!(Move::NULL.from(), 0);
        assert_eq!(Move::NULL.to(), 0);
        assert_eq!(Move::NULL.kind(), MoveType::Normal);
        assert_eq!(Move::NULL.promotion(), None);
        assert!(!Move::NULL.is_capture());
        assert!(!Move::NULL.is_promotion());
        assert!(!Move::NULL.is_castle());
    }
}
