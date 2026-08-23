use super::{
    DepthHistogram, REDUCTION_BUCKETS, ReductionHistogram, count,
    formatting::print_depth_histogram, pct,
};

#[derive(Clone, Copy, Debug, Default)]
pub struct LmrStats {
    pub attempts: u64,
    pub eligible_moves: u64,
    pub reduced_moves: u64,
    pub zero_reduction_moves: u64,
    pub reduction_plies: u64,
    pub researches: u64,
    pub research_alpha_improvements: u64,
    pub research_cutoffs: u64,
    pub history_improvements: u64,
    pub history_reductions: u64,
    pub reduction_histogram: ReductionHistogram,
    pub attempts_by_depth: DepthHistogram,
    pub researches_by_depth: DepthHistogram,
}

impl LmrStats {
    pub(super) fn print(&self) {
        if self.attempts == 0 && self.researches == 0 {
            return;
        }

        println!("Search Reductions");
        println!("  Late Move Reductions");
        println!("    {:<20} {:>14}", "Attempts:", count(self.attempts));
        println!(
            "    {:<20} {:>14}",
            "Eligible moves:",
            count(self.eligible_moves)
        );
        println!(
            "    {:<20} {:>14}",
            "Reduced moves:",
            count(self.reduced_moves)
        );
        println!(
            "    {:<20} {:>14}",
            "Zero reductions:",
            count(self.zero_reduction_moves)
        );
        println!(
            "    {:<20} {:>14}",
            "Reduction plies:",
            count(self.reduction_plies)
        );
        println!("    {:<20} {:>14}", "Re-searches:", count(self.researches));
        println!(
            "    {:<20} {:>14}",
            "Research improves:",
            count(self.research_alpha_improvements)
        );
        println!(
            "    {:<20} {:>14}",
            "Research cutoffs:",
            count(self.research_cutoffs)
        );
        println!(
            "    {:<20} {:>14}",
            "History Improvements:",
            count(self.history_improvements)
        );
        println!(
            "    {:<20} {:>14}",
            "History Reductions:",
            count(self.history_reductions)
        );
        if self.attempts > 0 {
            println!(
                "    {:<20} {:>14}",
                "Re-search rate:",
                pct(self.researches, self.attempts)
            );
        }

        let total_reductions = self.reduction_histogram.total();
        if total_reductions > 0 {
            println!("  Reduction Sizes");
            println!("    {:>10} {:>14} {:>10}", "Reduction", "Moves", "Share");
            for reduction in 0..REDUCTION_BUCKETS {
                let moves = self.reduction_histogram.bins[reduction];
                if moves == 0 {
                    continue;
                }

                let label = if reduction == REDUCTION_BUCKETS - 1 {
                    format!("{reduction}+")
                } else {
                    reduction.to_string()
                };
                println!(
                    "    {:>10} {:>14} {:>10}",
                    label,
                    count(moves),
                    pct(moves, total_reductions),
                );
            }
        }

        if self.attempts_by_depth.total() > 0 {
            println!("  LMR by Depth");
            print_depth_histogram(
                &self.attempts_by_depth,
                &self.researches_by_depth,
                "Re-searches",
            );
        }
    }
}

impl_counter_stats_ops!(
    LmrStats,
    attempts,
    eligible_moves,
    reduced_moves,
    zero_reduction_moves,
    reduction_plies,
    researches,
    research_alpha_improvements,
    research_cutoffs,
    history_improvements,
    history_reductions,
    reduction_histogram,
    attempts_by_depth,
    researches_by_depth,
);
