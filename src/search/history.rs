use crate::bitboard::Square;
use crate::search::Engine;
use crate::search::engine::SearchContext;
use crate::types::Color;

#[derive(Clone, Copy, Debug)]
pub struct HistoryTable {
    table: [[[i32; 64]; 64]; 2],
}

impl Default for HistoryTable {
    fn default() -> Self {
        Self {
            table: [[[0; 64]; 64]; 2],
        }
    }
}

impl HistoryTable {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self, color: Color, from: Square, to: Square) -> i32 {
        self.table[color as usize][from as usize][to as usize]
    }

    pub fn add_bonus(&mut self, color: Color, from: Square, to: Square, depth: usize) {
        let bonus = (depth * depth) as i32;
        self.table[color as usize][from as usize][to as usize] += bonus;
    }

    pub fn age(&mut self) {
        for color in 0..2 {
            for from in 0..64 {
                for to in 0..64 {
                    self.table[color][from][to] /= 2;
                }
            }
        }
    }
}

impl Engine {
    pub fn repetition_in_search(
        context: &SearchContext,
        board_hash: u64,
        halfmove_clock: usize,
    ) -> bool {
        let mut count = 0;

        // Do not look back farther than the reversible move window.
        let max_to_check = halfmove_clock.min(context.repetition_history.len());

        for &hash in context
            .repetition_history
            .iter()
            .rev()
            .take(max_to_check + 1)
            .step_by(2)
        {
            if hash == board_hash {
                count += 1;

                if count >= 1 {
                    return true;
                }
            }
        }

        false
    }
}
