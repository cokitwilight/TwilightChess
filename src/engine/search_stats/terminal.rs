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

        println!("Terminal Nodes");
        println!("  {:<22} {:>14}", "Checkmates:", count(self.checkmates));
        println!("  {:<22} {:>14}", "Stalemates:", count(self.stalemates));
        println!(
            "  {:<22} {:>14}",
            "Max-ply returns:",
            count(self.max_ply_returns)
        );
        println!(
            "  {:<22} {:>14}",
            "Stopped returns:",
            count(self.stopped_returns)
        );
    }
}

impl_counter_stats_ops!(
    TerminalStats,
    checkmates,
    stalemates,
    max_ply_returns,
    stopped_returns,
);
use super::count;
