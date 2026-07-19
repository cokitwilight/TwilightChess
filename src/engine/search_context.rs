use crate::engine::ordering::KillerTable;
use crate::engine::{SearchLimits, SearchStats};

#[derive(Clone, Debug)]
pub struct SearchContext {
    pub limits: SearchLimits,
    pub stats: SearchStats,

    pub killer_moves: KillerTable,
    pub repetition_history: Vec<u64>,

    pub start_time: std::time::Instant,
    pub stopped: bool,
}

impl SearchContext {
    pub fn new(limits: SearchLimits, repetition_history: Vec<u64>) -> Self {
        Self {
            limits,
            stats: SearchStats::default(),
            killer_moves: KillerTable::new(),
            repetition_history,
            start_time: std::time::Instant::now(),
            stopped: false,
        }
    }

    pub fn should_stop(&self) -> bool {
        if self.stopped {
            return true;
        }

        if let Some(max_nodes) = self.limits.max_nodes {
            if self.stats.nodes >= max_nodes {
                return true;
            }
        }

        if let Some(time_limit_ms) = self.limits.soft_time_limit_ms
            && self.start_time.elapsed().as_millis() >= time_limit_ms as u128
        {
            return true;
        }

        false
    }
}
