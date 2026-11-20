use super::DepthHistogram;
use super::formatting::{print_depth_histogram, submetric_count, submetric_pct, subsection};

/// Capture SEE pruning in negamax, excluding quiescence and move ordering.
#[derive(Clone, Copy, Debug, Default)]
#[cfg_attr(feature = "debug-uci", derive(serde::Serialize))]
pub struct MainSeeStats {
    pub attempts: u64,
    pub pruned_moves: u64,
    pub attempts_by_depth: DepthHistogram,
    pub pruned_moves_by_depth: DepthHistogram,
}

impl MainSeeStats {
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
        subsection("Main SEE pruning");
        submetric_count("Attempts", self.attempts);
        submetric_count("Pruned moves", self.pruned_moves);
        submetric_pct("Prune rate", self.pruned_moves, self.attempts);
        print_depth_histogram(
            &self.attempts_by_depth,
            &self.pruned_moves_by_depth,
            "Pruned",
        );
    }
}

impl_counter_stats_ops!(
    MainSeeStats,
    attempts,
    pruned_moves,
    attempts_by_depth,
    pruned_moves_by_depth
);
