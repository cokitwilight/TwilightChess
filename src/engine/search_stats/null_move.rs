use super::{DepthHistogram, count, formatting::print_depth_histogram, pct};

#[derive(Clone, Copy, Debug, Default)]
pub struct NullMoveStats {
    pub attempts: u64,
    pub cutoffs: u64,
    pub attempts_by_depth: DepthHistogram,
    pub cutoffs_by_depth: DepthHistogram,
}

impl NullMoveStats {
    pub(super) fn has_data(&self) -> bool {
        self.attempts > 0 || self.cutoffs > 0
    }

    pub(super) fn print(&self) {
        if !self.has_data() {
            return;
        }

        println!("  Null-Move Pruning");
        println!("    {:<20} {:>14}", "Attempts:", count(self.attempts));
        println!("    {:<20} {:>14}", "Cutoffs:", count(self.cutoffs));
        if self.attempts > 0 {
            println!(
                "    {:<20} {:>14}",
                "Cutoff rate:",
                pct(self.cutoffs, self.attempts)
            );
        }
        print_depth_histogram(&self.attempts_by_depth, &self.cutoffs_by_depth, "Cutoffs");
    }
}

impl_counter_stats_ops!(
    NullMoveStats,
    attempts,
    cutoffs,
    attempts_by_depth,
    cutoffs_by_depth,
);
