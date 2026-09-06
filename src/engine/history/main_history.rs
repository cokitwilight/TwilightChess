use crate::engine::history::{HistoryKey, history_bonus, history_malus, update};

const HISTORY_SIZE: usize = 768; // for regular history it is Color(2) * Piece(6) * To(64) = 768

#[derive(Clone, Debug)]
pub struct MainHistory {
    table: Box<[i32]>, // indexed as table[color][piece][to]
}

impl Default for MainHistory {
    fn default() -> Self {
        Self {
            table: vec![0; HISTORY_SIZE].into_boxed_slice(),
        }
    }
}

impl MainHistory {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self, key: HistoryKey) -> i32 {
        self.table[key.idx()]
    }

    pub fn add_bonus(&mut self, key: HistoryKey, depth: u16) {
        let bonus = history_bonus(depth);
        update(&mut self.table[key.idx()], bonus);
    }

    pub fn add_malus(&mut self, key: HistoryKey, depth: u16) {
        let malus = history_malus(depth);
        update(&mut self.table[key.idx()], malus);
    }
}
