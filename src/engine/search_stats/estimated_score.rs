use super::formatting::{metric, metric_count, metric_pct, section};

#[derive(Clone, Copy, Debug, Default)]
#[cfg_attr(feature = "debug-uci", derive(serde::Serialize))]
pub struct EstimatedScoreStats {
    pub estimates: u64,
    pub tt_selections: u64,
    pub changed_scores: u64,
    pub absolute_delta_sum: u64,
}

impl EstimatedScoreStats {
    pub(super) fn print(&self) {
        if self.estimates == 0 {
            return;
        }
        section("Estimated Score");
        metric_count("Estimates calculated", self.estimates);
        metric_count("TT score selected", self.tt_selections);
        metric_pct(
            "TT selections / estimates",
            self.tt_selections,
            self.estimates,
        );
        metric_count("TT changed score", self.changed_scores);
        metric_pct("Changed / estimates", self.changed_scores, self.estimates);
        if self.changed_scores > 0 {
            let mean = self.absolute_delta_sum as f64 / self.changed_scores as f64;
            metric("Mean |TT - corrected| (changed)", format!("{mean:.2}"));
        }
    }
}

impl_counter_stats_ops!(
    EstimatedScoreStats,
    estimates,
    tt_selections,
    changed_scores,
    absolute_delta_sum
);
