use crate::engine::search_stats::formatting::{
    count, metric_count, metric_pct, pct, print_depth_histogram, section, submetric_count,
    subsection,
};
use crate::engine::search_stats::{DepthHistogram, REDUCTION_BUCKETS, ReductionHistogram};

#[derive(Clone, Copy, Debug, Default)]
pub struct LmrStats {
    pub attempts: u64,
    pub eligible_moves: u64,
    pub reduced_moves: u64,
    pub zero_reduction_moves: u64,
    pub reduction_plies: u64,
    pub history_adjustments: u64,
    pub history_effects: u64,
    pub researches: u64,
    pub research_alpha_improvements: u64,
    pub research_cutoffs: u64,
    pub history_improvements: u64,
    pub history_reductions: u64,
    pub non_improving_adjustments: u64,
    pub non_improving_effects: u64,
    pub pv_adjustments: u64,
    pub pv_effects: u64,
    pub capture_adjustments: u64,
    pub capture_effects: u64,
    pub check_adjustments: u64,
    pub check_effects: u64,
    pub dynamic_depth_increases: u64,
    pub dynamic_depth_decreases: u64,
    pub dynamic_depth_unchanged: u64,
    pub dynamic_searches: u64,
    pub dynamic_searches_at_modified_depth: u64,
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
        metric_count("Re-search triggers", self.researches);
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
            metric_pct(
                "Re-search-trigger rate",
                self.researches,
                self.reduced_moves,
            );
        }

        let modifier_adjustments = self.history_adjustments
            + self.non_improving_adjustments
            + self.pv_adjustments
            + self.capture_adjustments
            + self.check_adjustments;
        if modifier_adjustments > 0 {
            subsection("Reduction modifier effects");
            println!(
                "    {:<20} {:>12} {:>12} {:>10}",
                "Modifier", "Applied", "Effective", "Rate"
            );
            self.print_modifier("History", self.history_adjustments, self.history_effects);
            self.print_modifier(
                "Non-improving",
                self.non_improving_adjustments,
                self.non_improving_effects,
            );
            self.print_modifier("PV node", self.pv_adjustments, self.pv_effects);
            self.print_modifier("Capture", self.capture_adjustments, self.capture_effects);
            self.print_modifier("Gives check", self.check_adjustments, self.check_effects);

            if self.history_improvements > 0 || self.history_reductions > 0 {
                submetric_count("History made smaller", self.history_improvements);
                submetric_count("History made larger", self.history_reductions);
            }
        }

        let dynamic_depth_choices = self.dynamic_depth_increases
            + self.dynamic_depth_decreases
            + self.dynamic_depth_unchanged;
        if dynamic_depth_choices > 0 || self.dynamic_searches > 0 {
            subsection("Dynamic re-search depth");
            submetric_count("Deeper choices", self.dynamic_depth_increases);
            submetric_count("Shallower choices", self.dynamic_depth_decreases);
            submetric_count("Unchanged choices", self.dynamic_depth_unchanged);
            submetric_count("Searches run", self.dynamic_searches);
            submetric_count(
                "At modified child depth",
                self.dynamic_searches_at_modified_depth,
            );
            if self.dynamic_searches > 0 {
                println!(
                    "    {:<28} {:>16}",
                    "Modified-depth search rate",
                    pct(
                        self.dynamic_searches_at_modified_depth,
                        self.dynamic_searches,
                    ),
                );
            }
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
            subsection("Re-search triggers by depth");
            print_depth_histogram(
                &self.attempts_by_depth,
                &self.researches_by_depth,
                "Triggers",
            );
        }
    }

    fn print_modifier(&self, label: &str, applied: u64, effective: u64) {
        if applied == 0 && effective == 0 {
            return;
        }

        println!(
            "    {:<20} {:>12} {:>12} {:>10}",
            label,
            count(applied),
            count(effective),
            pct(effective, applied),
        );
    }
}

impl_counter_stats_ops!(
    LmrStats,
    attempts,
    eligible_moves,
    reduced_moves,
    zero_reduction_moves,
    reduction_plies,
    history_adjustments,
    history_effects,
    researches,
    research_alpha_improvements,
    research_cutoffs,
    history_improvements,
    history_reductions,
    non_improving_adjustments,
    non_improving_effects,
    pv_adjustments,
    pv_effects,
    capture_adjustments,
    capture_effects,
    check_adjustments,
    check_effects,
    dynamic_depth_increases,
    dynamic_depth_decreases,
    dynamic_depth_unchanged,
    dynamic_searches,
    dynamic_searches_at_modified_depth,
    reduction_histogram,
    attempts_by_depth,
    researches_by_depth,
);
