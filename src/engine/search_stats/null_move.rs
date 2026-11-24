use crate::engine::search_stats::formatting::{
    count, pct, print_depth_histogram, submetric_count, submetric_pct, subsection,
};
use crate::engine::search_stats::{DepthHistogram, REDUCTION_BUCKETS, ReductionHistogram};

#[derive(Clone, Copy, Debug, Default)]
#[cfg_attr(feature = "debug-uci", derive(serde::Serialize))]
pub struct NullMoveStats {
    pub attempts: u64,
    pub cutoffs: u64,
    pub attempts_by_depth: DepthHistogram,
    pub cutoffs_by_depth: DepthHistogram,
    /// Calculated extra reduction R in child_depth = max(depth - 1 - R, 0).
    /// Bucket 63 combines reductions of 63 plies and above.
    pub reduction_histogram: ReductionHistogram,
    pub cutoffs_by_reduction: ReductionHistogram,
}

impl NullMoveStats {
    pub(super) fn has_data(&self) -> bool {
        self.attempts > 0
            || self.cutoffs > 0
            || self.attempts_by_depth.total() > 0
            || self.cutoffs_by_depth.total() > 0
            || self.reduction_histogram.total() > 0
            || self.cutoffs_by_reduction.total() > 0
    }

    pub(super) fn print(&self) {
        if !self.has_data() {
            return;
        }

        subsection("Null-move pruning");
        submetric_count("Attempts", self.attempts);
        submetric_count("Cutoffs", self.cutoffs);
        if self.attempts > 0 {
            submetric_pct("Cutoff rate", self.cutoffs, self.attempts);
        }
        if self.attempts_by_depth.total() > 0 || self.cutoffs_by_depth.total() > 0 {
            subsection("Attempts and cutoffs by depth");
            print_depth_histogram(&self.attempts_by_depth, &self.cutoffs_by_depth, "Cutoffs");
        }
        if self.reduction_histogram.total() > 0 || self.cutoffs_by_reduction.total() > 0 {
            subsection("Attempts and cutoffs by reduction (extra plies)");
            println!(
                "    {:>9} {:>12} {:>12} {:>10}",
                "Reduction", "Attempts", "Cutoffs", "Rate"
            );
            for reduction in 0..REDUCTION_BUCKETS {
                let attempts = self.reduction_histogram.bins[reduction];
                let cutoffs = self.cutoffs_by_reduction.bins[reduction];
                if attempts == 0 && cutoffs == 0 {
                    continue;
                }
                let label = if reduction == REDUCTION_BUCKETS - 1 {
                    format!("{reduction}+")
                } else {
                    reduction.to_string()
                };
                println!(
                    "    {:>9} {:>12} {:>12} {:>10}",
                    label,
                    count(attempts),
                    count(cutoffs),
                    pct(cutoffs, attempts),
                );
            }
        }
    }
}

impl_counter_stats_ops!(
    NullMoveStats,
    attempts,
    cutoffs,
    attempts_by_depth,
    cutoffs_by_depth,
    reduction_histogram,
    cutoffs_by_reduction,
);
