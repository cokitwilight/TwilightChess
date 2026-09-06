use std::time::{Duration, Instant};

use crate::engine::history::KillerTable;
use crate::engine::staged::staged::MoveBuffer;
use crate::engine::{MAX_PLY, SearchLimits, SearchStackEntry, SearchStats, SearchTermination};

const PICKER_FRAME_COUNT: usize = MAX_PLY + 1;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct PickerFrame(usize);

impl PickerFrame {
    pub(crate) const ROOT: Self = Self(0);

    pub(crate) const fn child(self) -> Self {
        Self(self.0 + 1)
    }

    pub(crate) const fn index(self) -> usize {
        self.0
    }
}

#[derive(Clone, Debug)]
pub struct SearchContext {
    pub limits: SearchLimits,
    pub stats: SearchStats,
    pub stack: [SearchStackEntry; MAX_PLY as usize],

    pub move_buffers: Box<[MoveBuffer]>,

    pub killer_moves: KillerTable,
    pub repetition_history: Vec<u64>,

    pub start_time: Instant,
    pub stopped: bool,
    pub stop_reason: Option<SearchTermination>,
}

impl SearchContext {
    pub fn new(limits: SearchLimits, repetition_history: Vec<u64>) -> Self {
        Self {
            limits,
            stats: SearchStats::default(),
            stack: [SearchStackEntry::default(); MAX_PLY as usize],
            move_buffers: vec![MoveBuffer::new(); PICKER_FRAME_COUNT].into_boxed_slice(),
            killer_moves: KillerTable::new(),
            repetition_history,
            start_time: Instant::now(),
            stopped: false,
            stop_reason: None,
        }
    }

    #[inline]
    pub fn elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }

    // pub(crate) fn reset_staged_move_buffer(&mut self, picker_frame: PickerFrame) {
    //     let picker_frame = picker_frame.index();
    //     assert!(
    //         picker_frame < self.move_buffers.len(),
    //         "picker frame {picker_frame} exceeded the preallocated search-frame limit"
    //     );
    //     self.move_buffers[picker_frame].clear();
    // }

    pub fn should_stop(&mut self) -> bool {
        if self.stopped {
            return true;
        }

        if let Some(max_nodes) = self.limits.max_nodes
            && self.stats.total_nodes() >= max_nodes
        {
            self.stopped = true;
            self.stop_reason = Some(SearchTermination::NodeLimit);
            return true;
        }

        if let Some(time_limit_ms) = self.limits.soft_time_limit_ms
            && self.elapsed() >= Duration::from_millis(time_limit_ms)
        {
            self.stopped = true;
            self.stop_reason = Some(SearchTermination::TimeLimit);
            return true;
        }

        false
    }
}
