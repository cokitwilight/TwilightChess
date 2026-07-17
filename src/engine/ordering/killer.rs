use crate::board::Move;

const MAX_PLY: usize = 128;

#[derive(Clone, Copy, Debug)]
pub struct KillerTable {
    moves: [[Option<Move>; 2]; MAX_PLY],
}

impl Default for KillerTable {
    fn default() -> Self {
        Self {
            moves: [[None; 2]; MAX_PLY],
        }
    }
}

impl KillerTable {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, ply: usize, mv: Move) {
        if ply >= MAX_PLY {
            return;
        }

        if self.moves[ply][0] == Some(mv) {
            return;
        }

        self.moves[ply][1] = self.moves[ply][0];
        self.moves[ply][0] = Some(mv);
    }

    pub fn contains(&self, ply: usize, mv: Move) -> bool {
        ply < MAX_PLY && (self.moves[ply][0] == Some(mv) || self.moves[ply][1] == Some(mv))
    }
}
