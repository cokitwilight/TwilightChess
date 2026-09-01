use crate::engine::search_stats::formatting::{metric_count, metric_pct, section};

#[derive(Clone, Copy, Debug, Default)]
pub struct AspirationStats {
    pub searches: u64,
    pub successful_first_windows: u64,
    pub fail_high: u64,
    pub fail_low: u64,
    pub full_window_fallbacks: u64,
    pub research_nodes: u64,
}

impl AspirationStats {
    pub(super) fn print(&self) {
        let total_fails = self.fail_high + self.fail_low;
        if self.searches == 0 && total_fails == 0 && self.full_window_fallbacks == 0 {
            return;
        }

        section("Aspiration Windows");
        metric_count("Searches", self.searches);
        metric_count("First-window successes", self.successful_first_windows);
        metric_count("Fail-highs", self.fail_high);
        metric_count("Fail-lows", self.fail_low);
        metric_count("Total failures", total_fails);
        metric_count("Full-window fallbacks", self.full_window_fallbacks);
        metric_count("Re-search nodes", self.research_nodes);
        if self.searches > 0 {
            metric_pct(
                "First-window success rate",
                self.successful_first_windows,
                self.searches,
            );
        }
        if total_fails == 0 {
            return;
        }
        metric_pct("Fail-high share", self.fail_high, total_fails);
    }
}

impl_counter_stats_ops!(
    AspirationStats,
    searches,
    successful_first_windows,
    fail_high,
    fail_low,
    full_window_fallbacks,
    research_nodes,
);
