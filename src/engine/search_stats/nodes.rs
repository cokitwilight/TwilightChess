use super::{count, nps};

#[derive(Clone, Copy, Debug, Default)]
pub struct NodeStats {
    pub main: u64,
    pub quiescence: u64,
    pub pv: u64,
    pub non_pv: u64,
    pub in_check: u64,
    pub root: u64,
}

impl NodeStats {
    #[inline]
    pub fn total(&self) -> u64 {
        self.main + self.quiescence
    }

    pub(super) fn print(&self, elapsed_secs: f64) {
        let total = self.total();

        println!("Nodes");
        println!("  {:<22} {:>14}", "Main:", count(self.main));
        println!("  {:<22} {:>14}", "Quiescence:", count(self.quiescence));
        println!("  {:<22} {:>14}", "Total:", count(total));
        println!("  {:<22} {:>14}", "NPS:", nps(total, elapsed_secs));
        println!("  {:<22} {:>14}", "PV:", count(self.pv));
        println!("  {:<22} {:>14}", "Non-PV:", count(self.non_pv));
        println!("  {:<22} {:>14}", "In check:", count(self.in_check));
        println!("  {:<22} {:>14}", "Root:", count(self.root));
    }
}

impl_counter_stats_ops!(NodeStats, main, quiescence, pv, non_pv, in_check, root);
