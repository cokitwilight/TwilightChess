use crate::engine::search_stats::formatting::{metric, metric_count, metric_pct, section};

#[derive(Clone, Copy, Debug, Default)]
pub struct SingularExtensionStats {
    pub attempts: u64,
    pub extensions: u64,
    pub fail_highs: u64,
    pub no_alternatives: u64,
    pub verification_nodes: u64,
}

impl SingularExtensionStats {
    pub(super) fn print(&self, total_nodes: u64) {
        if self.attempts == 0
            && self.extensions == 0
            && self.fail_highs == 0
            && self.no_alternatives == 0
            && self.verification_nodes == 0
        {
            return;
        }

        section("Singular Extensions");
        metric_count("Attempts", self.attempts);
        metric_count("Extensions", self.extensions);
        metric_count("Alternative fail-highs", self.fail_highs);
        metric_count("No alternatives", self.no_alternatives);
        metric_count("Verification nodes", self.verification_nodes);

        if self.attempts > 0 {
            metric_pct("Extension rate", self.extensions, self.attempts);
            metric_pct("Alternative fail-high rate", self.fail_highs, self.attempts);
            metric_pct("No-alternative rate", self.no_alternatives, self.attempts);
            metric(
                "Verification nodes / attempt",
                format!(
                    "{:.2}",
                    self.verification_nodes as f64 / self.attempts as f64
                ),
            );
        }
        if total_nodes > 0 {
            metric_pct(
                "Verification-node share",
                self.verification_nodes,
                total_nodes,
            );
        }
    }
}

impl_counter_stats_ops!(
    SingularExtensionStats,
    attempts,
    extensions,
    fail_highs,
    no_alternatives,
    verification_nodes,
);
