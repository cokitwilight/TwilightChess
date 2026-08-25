#[derive(Clone, Copy, Debug, Default)]
pub struct QuiescenceStats {
    pub normal_nodes: u64,
    pub in_check_nodes: u64,
    pub capture_candidates: u64,
    pub check_evasion_candidates: u64,
    pub max_ply_returns: u64,
    pub check_ply_limit_returns: u64,
}

impl QuiescenceStats {
    pub(super) fn print(&self) {
        if self.normal_nodes == 0 && self.in_check_nodes == 0 {
            return;
        }

        section("Quiescence Search");
        metric_count("Normal nodes", self.normal_nodes);
        metric_count("In-check nodes", self.in_check_nodes);
        metric_count("Capture candidates", self.capture_candidates);
        metric_count("Check-evasion candidates", self.check_evasion_candidates);
        metric_count("Max-ply returns", self.max_ply_returns);
        metric_count("Check-ply-limit returns", self.check_ply_limit_returns);
    }
}

impl_counter_stats_ops!(
    QuiescenceStats,
    normal_nodes,
    in_check_nodes,
    capture_candidates,
    check_evasion_candidates,
    max_ply_returns,
    check_ply_limit_returns,
);
use super::formatting::{metric_count, section};
