use super::{count, pct};

#[derive(Clone, Copy, Debug, Default)]
pub struct QuiescencePruningStats {
    pub delta_attempts: u64,
    pub delta_prunes: u64,
    pub see_attempts: u64,
    pub see_prunes: u64,
}

impl QuiescencePruningStats {
    pub(super) fn print(&self) {
        if self.delta_attempts == 0 && self.see_attempts == 0 {
            return;
        }

        println!("Quiescence Pruning");
        if self.delta_attempts > 0 {
            println!("  Delta Pruning");
            println!("    {:<20} {:>14}", "Attempts:", count(self.delta_attempts));
            println!(
                "    {:<20} {:>14}",
                "Pruned moves:",
                count(self.delta_prunes)
            );
            println!(
                "    {:<20} {:>14}",
                "Prune rate:",
                pct(self.delta_prunes, self.delta_attempts)
            );
        }
        if self.see_attempts > 0 {
            println!("  SEE Pruning");
            println!("    {:<20} {:>14}", "Attempts:", count(self.see_attempts));
            println!("    {:<20} {:>14}", "Pruned moves:", count(self.see_prunes));
            println!(
                "    {:<20} {:>14}",
                "Prune rate:",
                pct(self.see_prunes, self.see_attempts)
            );
        }
        println!(
            "  {:<22} {:>14}",
            "Total q prunes:",
            count(self.delta_prunes + self.see_prunes)
        );
    }
}

impl_counter_stats_ops!(
    QuiescencePruningStats,
    delta_attempts,
    delta_prunes,
    see_attempts,
    see_prunes,
);
