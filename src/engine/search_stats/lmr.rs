use super::{
    DepthHistogram, REDUCTION_BUCKETS, ReductionHistogram,
    formatting::{
        count, metric_count, metric_pct, pct, print_depth_histogram, section, submetric_count,
        subsection,
    },
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
        let depth_attempts = self.attempts_by_depth.total();
        let depth_researches = self.researches_by_depth.total();
        if self.eligible_moves == 0
            && self.attempts == 0
            && self.researches == 0
            && self.reduction_histogram.total() == 0
            && depth_attempts == 0
            && depth_researches == 0
        {
            return;
        }

        section("Late Move Reductions");
        metric_count("Eligible moves", self.eligible_moves.max(self.attempts));
        metric_count("Reduced moves", self.reduced_moves);
        metric_count("Zero-ply reductions", self.zero_reduction_moves);
        metric_count("Total reduction plies", self.reduction_plies);
        metric_count("Re-searches", self.researches);
        metric_count(
            "Re-search alpha improvements",
            self.research_alpha_improvements,
        );
        metric_count("Re-search cutoffs", self.research_cutoffs);
        metric_pct(
            "Reduced-move rate",
            self.reduced_moves,
            self.eligible_moves.max(self.attempts),
        );
        if self.reduced_moves > 0 {
            metric_pct("Re-search rate", self.researches, self.reduced_moves);
        }

        if self.history_improvements > 0 || self.history_reductions > 0 {
            subsection("History adjustment effects");
            submetric_count("Smaller reductions", self.history_improvements);
            submetric_count("Larger reductions", self.history_reductions);
        }

        let total_reductions = self.reduction_histogram.total();
        if total_reductions > 0 {
            subsection("Reduction-size distribution");
            println!("    {:>10} {:>12} {:>10}", "Plies", "Moves", "Share");
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
                    "    {:>10} {:>12} {:>10}",
                    label,
                    count(moves),
                    pct(moves, total_reductions),
                );
            }
        }

        if depth_attempts > 0 || depth_researches > 0 {
            subsection("Re-searches by depth");
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
