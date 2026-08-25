use super::formatting::{metric_count, section, submetric_count, submetric_pct, subsection};

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

        section("Quiescence Pruning");
        if self.delta_attempts > 0 {
            subsection("Delta pruning");
            submetric_count("Attempts", self.delta_attempts);
            submetric_count("Pruned moves", self.delta_prunes);
            submetric_pct("Prune rate", self.delta_prunes, self.delta_attempts);
        }
        if self.see_attempts > 0 {
            subsection("SEE pruning");
            submetric_count("Attempts", self.see_attempts);
            submetric_count("Pruned moves", self.see_prunes);
            submetric_pct("Prune rate", self.see_prunes, self.see_attempts);
        }
        metric_count("Total pruned moves", self.delta_prunes + self.see_prunes);
    }
}

impl_counter_stats_ops!(
    QuiescencePruningStats,
    delta_attempts,
    delta_prunes,
    see_attempts,
    see_prunes,
);
