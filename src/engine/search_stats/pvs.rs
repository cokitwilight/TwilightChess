#[derive(Clone, Copy, Debug, Default)]
pub struct PvsStats {
    pub null_window_searches: u64,
    pub alpha_improvements: u64,
    pub full_window_researches: u64,
    pub research_cutoffs: u64,
}

impl PvsStats {
    pub(super) fn print(&self) {
        if self.null_window_searches == 0 {
            return;
        }

        section("Principal Variation Search");
        metric_count("Null-window searches", self.null_window_searches);
        metric_count("Alpha improvements", self.alpha_improvements);
        metric_count("Full-window re-searches", self.full_window_researches);
        metric_count("Re-search cutoffs", self.research_cutoffs);
        metric_pct(
            "Alpha-improvement rate",
            self.alpha_improvements,
            self.null_window_searches,
        );
        metric_pct(
            "Full-window re-search rate",
            self.full_window_researches,
            self.null_window_searches,
        );
    }
}

impl_counter_stats_ops!(
    PvsStats,
    null_window_searches,
    alpha_improvements,
    full_window_researches,
    research_cutoffs,
);
use super::formatting::{metric_count, metric_pct, section};
