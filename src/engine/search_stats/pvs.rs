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

        println!("Principal Variation Search");
        println!(
            "  {:<22} {:>14}",
            "Null-window searches:",
            count(self.null_window_searches)
        );
        println!(
            "  {:<22} {:>14}",
            "Alpha improvements:",
            count(self.alpha_improvements)
        );
        println!(
            "  {:<22} {:>14}",
            "Full-window searches:",
            count(self.full_window_researches)
        );
        println!(
            "  {:<22} {:>14}",
            "Re-search cutoffs:",
            count(self.research_cutoffs)
        );
        println!(
            "  {:<22} {:>14}",
            "Alpha-improve rate:",
            pct(self.alpha_improvements, self.null_window_searches)
        );
        println!(
            "  {:<22} {:>14}",
            "Full-window rate:",
            pct(self.full_window_researches, self.null_window_searches)
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
use super::{count, pct};
