use super::{
    DepthHistogram,
    formatting::{print_depth_histogram, submetric_count, submetric_pct, subsection},
};

#[derive(Clone, Copy, Debug, Default)]
pub struct ReverseFutilityStats {
    pub attempts: u64,
    pub cutoffs: u64,
    pub attempts_by_depth: DepthHistogram,
    pub cutoffs_by_depth: DepthHistogram,
}

impl ReverseFutilityStats {
    pub(super) fn has_data(&self) -> bool {
        self.attempts > 0
            || self.cutoffs > 0
            || self.attempts_by_depth.total() > 0
            || self.cutoffs_by_depth.total() > 0
    }

    pub(super) fn print(&self) {
        if !self.has_data() {
            return;
        }

        subsection("Reverse futility pruning");
        submetric_count("Attempts", self.attempts);
        submetric_count("Cutoffs", self.cutoffs);
        if self.attempts > 0 {
            submetric_pct("Cutoff rate", self.cutoffs, self.attempts);
        }
        if self.attempts_by_depth.total() > 0 || self.cutoffs_by_depth.total() > 0 {
            subsection("Attempts and cutoffs by depth");
            print_depth_histogram(&self.attempts_by_depth, &self.cutoffs_by_depth, "Cutoffs");
        }
    }
}

impl_counter_stats_ops!(
    ReverseFutilityStats,
    attempts,
    cutoffs,
    attempts_by_depth,
    cutoffs_by_depth,
);
