use std::time::{Duration, Instant};

#[cfg(feature = "debug-uci")]
use crate::debug_uci::discovery::FindSolutionState;
use crate::engine::history::KillerTable;
use crate::engine::staged::staged::MoveBuffer;
use crate::engine::{MAX_PLY, SearchLimits, SearchStackEntry, SearchStats, SearchTermination};
#[cfg(feature = "debug-uci")]
use crate::engine::{SearchIteration, SearchResult};

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
    /// Deepest root-relative ply visited in the current iteration, including quiescence.
    pub seldepth: usize,
    pub stack: [SearchStackEntry; MAX_PLY],

    pub move_buffers: Box<[MoveBuffer]>,

    pub killer_moves: KillerTable,
    pub repetition_history: Vec<u64>,

    pub start_time: Instant,
    pub stopped: bool,
    pub stop_reason: Option<SearchTermination>,
    #[cfg(feature = "debug-uci")]
    pub(crate) cancellation: Option<std::sync::Arc<std::sync::atomic::AtomicBool>>,
    #[cfg(feature = "debug-uci")]
    pub(crate) iterations: Vec<SearchIteration>,
    #[cfg(feature = "debug-uci")]
    pub(crate) finder: Option<FindSolutionState>,
}

impl SearchContext {
    pub fn new(limits: SearchLimits, repetition_history: Vec<u64>) -> Self {
        Self {
            limits,
            stats: SearchStats::default(),
            seldepth: 0,
            stack: [SearchStackEntry::default(); MAX_PLY],
            move_buffers: vec![MoveBuffer::new(); PICKER_FRAME_COUNT].into_boxed_slice(),
            killer_moves: KillerTable::new(),
            repetition_history,
            start_time: Instant::now(),
            stopped: false,
            stop_reason: None,
            #[cfg(feature = "debug-uci")]
            cancellation: None,
            #[cfg(feature = "debug-uci")]
            iterations: Vec::new(),
            #[cfg(feature = "debug-uci")]
            finder: None,
        }
    }

    #[cfg(feature = "debug-uci")]
    /// Save a completed depth and return whether the optional finder confirmed a solution.
    pub(crate) fn record_iteration(&mut self, result: &SearchResult) -> bool {
        let iteration = SearchIteration::from_result(result);
        let solved = if let Some(finder) = &mut self.finder {
            finder.observe(&iteration)
        } else {
            false
        };
        self.iterations.push(iteration);
        solved
    }

    #[inline]
    pub fn elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }

    pub fn should_stop(&mut self) -> bool {
        #[cfg(feature = "debug-uci")]
        if self
            .cancellation
            .as_ref()
            .is_some_and(|flag| flag.load(std::sync::atomic::Ordering::Relaxed))
        {
            self.stopped = true;
            self.stop_reason = Some(SearchTermination::Cancelled);
        }
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
