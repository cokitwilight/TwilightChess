#[derive(Clone, Copy, Debug, Default)]
pub struct TerminalStats {
    pub checkmates: u64,
    pub stalemates: u64,
    pub max_ply_returns: u64,
    pub stopped_returns: u64,
}

impl TerminalStats {
    pub(super) fn print(&self) {
        if self.checkmates == 0
            && self.stalemates == 0
            && self.max_ply_returns == 0
            && self.stopped_returns == 0
        {
            return;
        }

        section("Terminal Nodes");
        metric_count("Checkmates", self.checkmates);
        metric_count("Stalemates", self.stalemates);
        metric_count("Max-ply returns", self.max_ply_returns);
        metric_count("Stopped returns", self.stopped_returns);
    }
}

impl_counter_stats_ops!(
    TerminalStats,
    checkmates,
    stalemates,
    max_ply_returns,
    stopped_returns,
);
use crate::engine::search_stats::formatting::{metric_count, section};
