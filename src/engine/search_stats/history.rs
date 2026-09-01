use crate::engine::search_stats::formatting::{metric_count, section};

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
    pub(super) fn print(&self) {
        if self.bonus_updates == 0
            && self.malus_updates == 0
            && self.continuation_bonus_updates == 0
            && self.continuation_malus_updates == 0
        {
            return;
        }

        section("History Updates");
        metric_count("Bonuses", self.bonus_updates);
        metric_count("Maluses", self.malus_updates);
        metric_count("Continuation bonuses", self.continuation_bonus_updates);
        metric_count("Continuation maluses", self.continuation_malus_updates);
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
