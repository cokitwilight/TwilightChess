use std::time::{Duration, Instant};

#[cfg(feature = "debug-uci")]
use crate::board::Move;
#[cfg(feature = "debug-uci")]
use crate::engine::MAX_PV;
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
    /// Deepest root-relative ply visited in the current iteration, including quiescence.
    pub seldepth: usize,
    pub stack: [SearchStackEntry; MAX_PLY as usize],

    pub move_buffers: Box<[MoveBuffer]>,

    pub killer_moves: KillerTable,
    pub repetition_history: Vec<u64>,

    pub start_time: Instant,
    pub stopped: bool,
    pub stop_reason: Option<SearchTermination>,
    #[cfg(feature = "debug-uci")]
    pub(crate) cancellation: Option<std::sync::Arc<std::sync::atomic::AtomicBool>>,
    // Indexed by picker frame, not ply: singular verification recurses at the same ply.
    #[cfg(feature = "debug-uci")]
    pv_table: Option<PvTable>,
}

#[cfg(feature = "debug-uci")]
#[derive(Clone, Debug)]
struct PvTable {
    lines: Vec<[Option<Move>; MAX_PV]>,
    lengths: Vec<usize>,
}

impl SearchContext {
    pub fn new(limits: SearchLimits, repetition_history: Vec<u64>) -> Self {
        Self {
            limits,
            stats: SearchStats::default(),
            seldepth: 0,
            stack: [SearchStackEntry::default(); MAX_PLY as usize],
            move_buffers: vec![MoveBuffer::new(); PICKER_FRAME_COUNT].into_boxed_slice(),
            killer_moves: KillerTable::new(),
            repetition_history,
            start_time: Instant::now(),
            stopped: false,
            stop_reason: None,
            #[cfg(feature = "debug-uci")]
            cancellation: None,
            #[cfg(feature = "debug-uci")]
            pv_table: None,
        }
    }

    #[cfg(feature = "debug-uci")]
    pub(crate) fn enable_pv(&mut self) {
        self.pv_table = Some(PvTable {
            lines: vec![[None; MAX_PV]; PICKER_FRAME_COUNT],
            lengths: vec![0; PICKER_FRAME_COUNT],
        });
    }

    #[cfg(feature = "debug-uci")]
    #[inline]
    pub(crate) fn clear_pv(&mut self, frame: PickerFrame) {
        if let Some(table) = &mut self.pv_table {
            table.lengths[frame.index()] = 0;
        }
    }

    #[cfg(feature = "debug-uci")]
    #[inline]
    pub(crate) fn update_pv(&mut self, frame: PickerFrame, mv: Move) {
        if let Some(table) = &mut self.pv_table {
            let index = frame.index();
            let child_len = table.lengths[index + 1].min(MAX_PV - 1);
            let (parents, children) = table.lines.split_at_mut(index + 1);
            parents[index][0] = Some(mv);
            parents[index][1..child_len + 1].copy_from_slice(&children[0][..child_len]);
            table.lengths[index] = child_len + 1;
        }
    }

    #[cfg(feature = "debug-uci")]
    pub(crate) fn pv_line(&self, frame: PickerFrame) -> Option<[Option<Move>; MAX_PV]> {
        self.pv_table.as_ref().map(|table| {
            let mut line = [None; MAX_PV];
            let len = table.lengths[frame.index()];
            line[..len].copy_from_slice(&table.lines[frame.index()][..len]);
            line
        })
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
