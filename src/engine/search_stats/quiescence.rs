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

        println!("Quiescence Search");
        println!("  {:<22} {:>14}", "Normal nodes:", count(self.normal_nodes));
        println!(
            "  {:<22} {:>14}",
            "In-check nodes:",
            count(self.in_check_nodes)
        );
        println!(
            "  {:<22} {:>14}",
            "Capture candidates:",
            count(self.capture_candidates)
        );
        println!(
            "  {:<22} {:>14}",
            "Evasion candidates:",
            count(self.check_evasion_candidates)
        );
        println!(
            "  {:<22} {:>14}",
            "Max-ply returns:",
            count(self.max_ply_returns)
        );
        println!(
            "  {:<22} {:>14}",
            "Check-ply limits:",
            count(self.check_ply_limit_returns)
        );
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
use super::count;
