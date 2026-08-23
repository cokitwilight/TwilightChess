use super::{DepthHistogram, count, formatting::print_depth_histogram, pct};

#[derive(Clone, Copy, Debug, Default)]
pub struct FutilityStats {
    pub attempts: u64,
    pub pruned_moves: u64,
    pub attempts_by_depth: DepthHistogram,
    pub pruned_moves_by_depth: DepthHistogram,
}

impl FutilityStats {
    pub(super) fn has_data(&self) -> bool {
        self.attempts > 0 || self.pruned_moves > 0
    }

    pub(super) fn print(&self) {
        if !self.has_data() {
            return;
        }

        println!("  Futility Pruning");
        println!("    {:<20} {:>14}", "Attempts:", count(self.attempts));
        println!(
            "    {:<20} {:>14}",
            "Pruned moves:",
            count(self.pruned_moves)
        );
        if self.attempts > 0 {
            println!(
                "    {:<20} {:>14}",
                "Prune rate:",
                pct(self.pruned_moves, self.attempts)
            );
        }
        print_depth_histogram(
            &self.attempts_by_depth,
            &self.pruned_moves_by_depth,
            "Pruned",
        );
    }
}

impl_counter_stats_ops!(
    FutilityStats,
    attempts,
    pruned_moves,
    attempts_by_depth,
    pruned_moves_by_depth,
);
