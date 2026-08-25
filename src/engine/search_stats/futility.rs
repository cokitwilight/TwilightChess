use super::{
    DepthHistogram,
    formatting::{print_depth_histogram, submetric_count, submetric_pct, subsection},
};

#[derive(Clone, Copy, Debug, Default)]
pub struct FutilityStats {
    pub attempts: u64,
    pub pruned_moves: u64,
    pub attempts_by_depth: DepthHistogram,
    pub pruned_moves_by_depth: DepthHistogram,
}

impl FutilityStats {
    pub(super) fn has_data(&self) -> bool {
        self.attempts > 0
            || self.pruned_moves > 0
            || self.attempts_by_depth.total() > 0
            || self.pruned_moves_by_depth.total() > 0
    }

    pub(super) fn print(&self) {
        if !self.has_data() {
            return;
        }

        subsection("Futility pruning");
        submetric_count("Attempts", self.attempts);
        submetric_count("Pruned moves", self.pruned_moves);
        if self.attempts > 0 {
            submetric_pct("Prune rate", self.pruned_moves, self.attempts);
        }
        if self.attempts_by_depth.total() > 0 || self.pruned_moves_by_depth.total() > 0 {
            subsection("Attempts and prunes by depth");
            print_depth_histogram(
                &self.attempts_by_depth,
                &self.pruned_moves_by_depth,
                "Pruned",
            );
        }
    }
}

impl_counter_stats_ops!(
    FutilityStats,
    attempts,
    pruned_moves,
    attempts_by_depth,
    pruned_moves_by_depth,
);
