use crate::bitboard::Square;
use crate::engine::history::{CaptureHistory, ContinuationHistory, MainHistory};
use crate::engine::{Engine, MAX_PLY, SearchContext, SearchStackEntry};
use crate::types::{Color, PieceType};

const HISTORY_MAX: i32 = 16_384;

pub const HISTORY_KEY_COUNT: usize = 768;

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
    pub capture: CaptureHistory,
}

impl HistoryTables {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_quiet_score(
        &self,
        stack: &[SearchStackEntry; MAX_PLY],
        ply: usize,
        curr_key: HistoryKey,
    ) -> i32 {
        let mut score = self.main.get(curr_key);

        if let Some(prev_key) = stack[ply].history_index {
            score += self.continuation.get(1, prev_key, curr_key);
        }

        if ply > 0 {
            if let Some(prev_key) = stack[ply - 1].history_index {
                score += self.continuation.get(2, prev_key, curr_key) / 2;
            }

            if ply > 2 {
                if let Some(prev_key) = stack[ply - 3].history_index {
                    score += self.continuation.get(4, prev_key, curr_key) / 2;
                }
            }
        }

        score
    }

    pub fn get_capture_score(&self, curr_key: HistoryKey, captured: PieceType) -> i32 {
        self.capture.get(curr_key, captured)
    }

    pub fn add_quiet_bonus(
        &mut self,
        curr_key: HistoryKey,
        context: &mut SearchContext,
        ply: usize,
        depth: u16,
    ) {
        context.stats.history_stats.bonus_updates += 1;
        self.main.add_bonus(curr_key, depth);

        if let Some(prev_key) = context.stack[ply].history_index {
            context.stats.history_stats.continuation_bonus_updates += 1;

            self.continuation.add_bonus(1, prev_key, curr_key, depth);
        }

        if ply > 0 {
            if let Some(prev_key) = context.stack[ply - 1].history_index {
                context.stats.history_stats.continuation_bonus_updates += 1; // LATER CHANGE TO PRE PLY CHANGES NOT JUST ONE GROUPED ONE

                self.continuation.add_bonus(2, prev_key, curr_key, depth);
            }

            if ply > 2 {
                if let Some(prev_key) = context.stack[ply - 3].history_index {
                    context.stats.history_stats.continuation_bonus_updates += 1;

                    self.continuation.add_bonus(4, prev_key, curr_key, depth);
                }
            }
        }
    }

    pub fn add_capture_bonus(&mut self, curr_key: HistoryKey, captured: PieceType, depth: u16) {
        self.capture.add_bonus(curr_key, captured, depth);
    }

    pub fn add_quiet_malus(
        &mut self,
        curr_key: HistoryKey,
        context: &mut SearchContext,
        ply: usize,
        depth: u16,
    ) {
        context.stats.history_stats.malus_updates += 1;
        self.main.add_malus(curr_key, depth);

        if let Some(prev_key) = context.stack[ply].history_index {
            context.stats.history_stats.continuation_malus_updates += 1;

            self.continuation.add_malus(1, prev_key, curr_key, depth);
        }

        if ply > 0 {
            if let Some(prev_key) = context.stack[ply - 1].history_index {
                context.stats.history_stats.continuation_malus_updates += 1; // LATER CHANGE TO PRE PLY CHANGES NOT JUST ONE GROUPED ONE

                self.continuation.add_malus(2, prev_key, curr_key, depth);
            }

            if ply > 2 {
                if let Some(prev_key) = context.stack[ply - 3].history_index {
                    context.stats.history_stats.continuation_malus_updates += 1;

                    self.continuation.add_malus(4, prev_key, curr_key, depth);
                }
            }
        }
    }

    pub fn add_capture_malus(&mut self, curr_key: HistoryKey, captured: PieceType, depth: u16) {
        self.capture.add_malus(curr_key, captured, depth);
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
