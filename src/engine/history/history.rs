use crate::bitboard::Square;
use crate::engine::history::{ContinuationHistory, MainHistory};
use crate::engine::{Engine, SearchContext};
use crate::types::{Color, PieceType};

const HISTORY_MAX: i32 = 16_384;

#[derive(Clone, Copy, Debug)]
pub struct HistoryKey(u16);

impl HistoryKey {
    #[inline]
    pub fn new(color: Color, piece: PieceType, to: Square) -> Self {
        let piece_index = color.idx() * 6 + piece.idx();
        Self((piece_index * 64 + to as usize) as u16)
    }

    #[inline]
    pub fn idx(self) -> usize {
        self.0 as usize
    }
}

#[derive(Clone, Debug, Default)]
pub struct HistoryTables {
    pub main: MainHistory,
    pub continuation: ContinuationHistory,
}

impl HistoryTables {
    pub fn new() -> Self {
        Self::default()
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

                if count >= 2 {
                    return true;
                }
            }
        }

        false
    }
}

#[inline]
pub fn update(value: &mut i32, bonus: i32) {
    let bonus = bonus.clamp(-HISTORY_MAX, HISTORY_MAX);
    let current = *value;

    *value = current + bonus - current * bonus.abs() / HISTORY_MAX;
}

#[inline]
pub fn history_bonus(depth: u16) -> i32 {
    32 * (depth * depth) as i32
}

#[inline]
pub fn history_malus(depth: u16) -> i32 {
    16 * (depth * depth) as i32
}
