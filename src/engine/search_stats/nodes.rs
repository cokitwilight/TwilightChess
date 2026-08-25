use super::formatting::{metric_count, nps, section};

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

        section("Nodes");
        metric_count("Main search", self.main);
        metric_count("Quiescence", self.quiescence);
        metric_count("Total", total);
        super::formatting::metric("Nodes per second", nps(total, elapsed_secs));
        metric_count("PV", self.pv);
        metric_count("Non-PV", self.non_pv);
        metric_count("In check", self.in_check);
        metric_count("Root", self.root);
    }
}

impl_counter_stats_ops!(NodeStats, main, quiescence, pv, non_pv, in_check, root);
