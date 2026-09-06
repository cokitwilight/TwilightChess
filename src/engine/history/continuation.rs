use crate::engine::history::{HISTORY_KEY_COUNT, HistoryKey, history_bonus, history_malus, update};

const CONTINUATION_SIZE: usize = HISTORY_KEY_COUNT * HISTORY_KEY_COUNT; // for continuation it is color(2) * prev_piece(6) * prev_to(64) * color(2) * current_piece(6) * current_to(64) = 589824
// Indexed as [color_prev][prev_piece][prev_to][color_cur ][curr_piece][curr_to]

#[derive(Clone, Debug)]
pub struct ContinuationHistory {
    table_one: Box<[i32]>,  // 1 ply ago(coutner move history)
    table_two: Box<[i32]>,  // 2 plies ago
    table_four: Box<[i32]>, // 4 plies(later) ...
}

impl Default for ContinuationHistory {
    fn default() -> Self {
        Self {
            table_one: vec![0; CONTINUATION_SIZE].into_boxed_slice(),
            table_two: vec![0; CONTINUATION_SIZE].into_boxed_slice(),
            table_four: vec![0; CONTINUATION_SIZE].into_boxed_slice(),
        }
    }
}

impl ContinuationHistory {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn add_bonus(
        &mut self,
        ply: usize,
        prev_key: HistoryKey,
        curr_key: HistoryKey,
        depth: u16,
    ) {
        let index = index(prev_key, curr_key);
        let bonus = history_bonus(depth);

        match ply {
            1 => update(&mut self.table_one[index], bonus),
            2 => update(&mut self.table_two[index], bonus),
            4 => update(&mut self.table_four[index], bonus),
            _ => unreachable!("Continuation called with invalid ply count {}", ply),
        }
    }

    pub fn add_malus(
        &mut self,
        ply: usize,
        prev_key: HistoryKey,
        curr_key: HistoryKey,
        depth: u16,
    ) {
        let index = index(prev_key, curr_key);
        let malus = history_malus(depth);
        match ply {
            1 => update(&mut self.table_one[index], malus),
            2 => update(&mut self.table_two[index], malus),
            4 => update(&mut self.table_four[index], malus),
            _ => unreachable!("Continuation called with invalid ply count {}", ply),
        }
    }

    pub fn get(&self, ply: usize, prev_key: HistoryKey, curr_key: HistoryKey) -> i32 {
        let index = index(prev_key, curr_key);
        match ply {
            1 => self.table_one[index],
            2 => self.table_two[index],
            4 => self.table_four[index],
            _ => unreachable!("Continuation called with invalid ply count {}", ply),
        }
    }
}

#[inline]
fn index(prev_key: HistoryKey, curr_key: HistoryKey) -> usize {
    prev_key.idx() * HISTORY_KEY_COUNT + curr_key.idx()
}
