use super::formatting::{submetric_count, submetric_pct, subsection};

#[derive(Clone, Copy, Debug, Default)]
#[cfg_attr(feature = "debug-uci", derive(serde::Serialize))]
pub struct MateDistanceStats {
    pub checks: u64,
    pub alpha_raised: u64,
    pub beta_lowered: u64,
    pub cutoffs: u64,
}

impl MateDistanceStats {
    pub(super) fn print(&self) {
        if self.checks == 0 {
            return;
        }
        subsection("Mate distance pruning");
        submetric_count("Windows checked", self.checks);
        submetric_count("Alpha raised", self.alpha_raised);
        submetric_count("Beta lowered", self.beta_lowered);
        submetric_count("Cutoffs", self.cutoffs);
        submetric_pct("Cutoffs / checks", self.cutoffs, self.checks);
    }
}

impl_counter_stats_ops!(
    MateDistanceStats,
    checks,
    alpha_raised,
    beta_lowered,
    cutoffs
);
