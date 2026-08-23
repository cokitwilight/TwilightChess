use super::{count, pct};

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

        println!("Aspiration Windows");
        println!("  {:<22} {:>14}", "Searches:", count(self.searches));
        println!(
            "  {:<22} {:>14}",
            "First-window hits:",
            count(self.successful_first_windows)
        );
        println!("  {:<22} {:>14}", "Fail high:", count(self.fail_high));
        println!("  {:<22} {:>14}", "Fail low:", count(self.fail_low));
        println!("  {:<22} {:>14}", "Total fails:", count(total_fails));
        println!(
            "  {:<22} {:>14}",
            "Full-window fallbacks:",
            count(self.full_window_fallbacks)
        );
        println!(
            "  {:<22} {:>14}",
            "Re-search nodes:",
            count(self.research_nodes)
        );
        if self.searches > 0 {
            println!(
                "  {:<22} {:>14}",
                "First-window rate:",
                pct(self.successful_first_windows, self.searches)
            );
        }
        if total_fails == 0 {
            return;
        }
        println!(
            "  {:<22} {:>14}",
            "Fail-high share:",
            pct(self.fail_high, total_fails)
        );
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
