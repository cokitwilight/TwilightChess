use crate::{
    engine::history::{HISTORY_KEY_COUNT, HistoryKey, history_bonus, history_malus, update},
    types::PieceType,
};

// indexed as history key([color][piece][to]) and captured piece type
const CAPTURE_SIZE: usize = HISTORY_KEY_COUNT * 6;

#[derive(Clone, Debug)]
pub struct CaptureHistory {
    table: Box<[i32]>,
}

impl Default for CaptureHistory {
    fn default() -> Self {
        Self {
            table: vec![0; CAPTURE_SIZE].into_boxed_slice(),
        }
    }
}

impl CaptureHistory {
    pub fn add_bonus(&mut self, key: HistoryKey, captured: PieceType, depth: u16) {
        let index = index(key, captured);
        let bonus = history_bonus(depth);

        update(&mut self.table[index], bonus);
    }

    pub fn add_malus(&mut self, key: HistoryKey, captured: PieceType, depth: u16) {
        let index = index(key, captured);
        let malus = history_malus(depth);

        update(&mut self.table[index], malus);
    }

    pub fn get(&self, key: HistoryKey, captured: PieceType) -> i32 {
        let index = index(key, captured);
        self.table[index]
    }
}

#[inline]
fn index(key: HistoryKey, captured: PieceType) -> usize {
    key.idx() * 6 + captured.idx()
}
