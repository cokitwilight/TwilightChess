use crate::engine::search_stats::formatting::{submetric_count, submetric_pct, subsection};

#[derive(Clone, Copy, Debug, Default)]
pub struct LmpStats {
    pub attempts: u64,
    pub history_rejections: u64,
    pub pruned_moves: u64,
}

impl LmpStats {
    pub(super) fn has_data(&self) -> bool {
        self.attempts > 0 || self.history_rejections > 0 || self.pruned_moves > 0
    }

    pub(super) fn print(&self) {
        if !self.has_data() {
            return;
        }

        subsection("Late move pruning");
        submetric_count("Attempts", self.attempts);
        submetric_count("Rejected by history", self.history_rejections);
        submetric_count("Pruned moves", self.pruned_moves);
        if self.attempts > 0 {
            submetric_pct("Prune rate", self.pruned_moves, self.attempts);
        }
    }
}

impl_counter_stats_ops!(LmpStats, attempts, history_rejections, pruned_moves,);
