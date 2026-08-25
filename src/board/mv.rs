use crate::bitboard::Square;
use crate::types::PieceType;
use std::cmp::Reverse;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum MoveType {
    Normal = 0,
    Capture = 1,
    EnPassant = 2,
    Castle = 3,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Move {
    from: Square,
    to: Square,
    kind: MoveType,
    promotion: Option<PieceType>,
}

impl Move {
    pub const NULL: Move = Move {
        from: 0,
        to: 0,
        kind: MoveType::Normal,
        promotion: None,
    };

    pub fn new(from: Square, to: Square, kind: MoveType, promotion: Option<PieceType>) -> Self {
        Self {
            from,
            to,
            kind,
            promotion,
        }
    }

    pub fn is_capture(self) -> bool {
        matches!(self.kind, MoveType::Capture | MoveType::EnPassant)
    }

    pub fn is_promotion(self) -> bool {
        self.promotion.is_some()
    }

    pub fn is_castle(self) -> bool {
        matches!(self.kind, MoveType::Castle)
    }
    pub fn from(self) -> Square {
        self.from
    }
    pub fn to(self) -> Square {
        self.to
    }
    pub fn kind(self) -> MoveType {
        self.kind
    }
    pub fn promotion(self) -> Option<PieceType> {
        self.promotion
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct MoveList {
    moves: [Move; 256],
    len: usize,
}

impl MoveList {
    pub fn new() -> Self {
        Self {
            moves: [Move::NULL; 256],
            len: 0,
        }
    }

    pub fn push(&mut self, mv: Move) {
        assert!(
            self.len < self.moves.len(),
            "MoveList overflow: more than 256 moves generated"
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
        if self.len() == 0 || index >= self.len() {
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
