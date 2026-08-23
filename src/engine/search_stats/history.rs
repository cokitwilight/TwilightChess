use super::{count, pct};

#[derive(Clone, Copy, Debug, Default)]
pub struct HistoryStats {
    pub killer_cutoffs: u64,
    pub history_cutoffs: u64,
    pub bonus_updates: u64,
    pub malus_updates: u64,
    pub continuation_bonus_updates: u64,
    pub continuation_malus_updates: u64,
}

impl HistoryStats {
    pub(super) fn print(&self, beta_cutoffs: u64) {
        if self.killer_cutoffs == 0 && self.history_cutoffs == 0 {
            return;
        }

        println!("Move History");
        println!(
            "  {:<22} {:>14}",
            "Killer cutoffs:",
            count(self.killer_cutoffs)
        );
        println!(
            "  {:<22} {:>14}",
            "History cutoffs:",
            count(self.history_cutoffs)
        );
        println!(
            "  {:<22} {:>14}",
            "History bonuses:",
            count(self.bonus_updates)
        );
        println!(
            "  {:<22} {:>14}",
            "History maluses:",
            count(self.malus_updates)
        );
        println!(
            "  {:<22} {:>14}",
            "Continuation bonuses:",
            count(self.continuation_bonus_updates)
        );
        println!(
            "  {:<22} {:>14}",
            "Continuation maluses:",
            count(self.continuation_malus_updates)
        );

        if beta_cutoffs > 0 {
            println!(
                "  {:<22} {:>14}",
                "Killer / beta:",
                pct(self.killer_cutoffs, beta_cutoffs)
            );
            println!(
                "  {:<22} {:>14}",
                "History / beta:",
                pct(self.history_cutoffs, beta_cutoffs)
            );
        }
    }
}

impl_counter_stats_ops!(
    HistoryStats,
    killer_cutoffs,
    history_cutoffs,
    bonus_updates,
    malus_updates,
    continuation_bonus_updates,
    continuation_malus_updates,
);
