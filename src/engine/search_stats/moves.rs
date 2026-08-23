use super::{NodeStats, count, pct};

#[derive(Clone, Copy, Debug, Default)]
pub struct MoveStats {
    pub main_searched: u64,
    pub quiescence_searched: u64,
    pub illegal_main: u64,
    pub illegal_quiescence: u64,
}

impl MoveStats {
    pub(super) fn print(&self, nodes: &NodeStats) {
        let total_moves = self.main_searched + self.quiescence_searched;
        let illegal_total = self.illegal_main + self.illegal_quiescence;

        if total_moves == 0 && self.illegal_main == 0 {
            return;
        }

        println!("Moves");
        println!(
            "  {:<22} {:>14}",
            "Main searched:",
            count(self.main_searched)
        );
        println!(
            "  {:<22} {:>14}",
            "Q searched:",
            count(self.quiescence_searched)
        );
        println!("  {:<22} {:>14}", "Total searched:", count(total_moves));
        println!(
            "  {:<22} {:>14}",
            "Illegal pseudo:",
            count(self.illegal_main)
        );
        println!(
            "  {:<22} {:>14}",
            "Illegal Q pseudo:",
            count(self.illegal_quiescence)
        );

        let pseudo_total = total_moves + illegal_total;
        if pseudo_total > 0 {
            println!(
                "  {:<22} {:>14}",
                "Illegal rate:",
                pct(illegal_total, pseudo_total)
            );
        }
        if nodes.main > 0 {
            println!(
                "  {:<22} {:>14.2}",
                "Moves / node:",
                self.main_searched as f64 / nodes.main as f64
            );
        }
        if nodes.quiescence > 0 {
            println!(
                "  {:<22} {:>14.2}",
                "Q moves / qnode:",
                self.quiescence_searched as f64 / nodes.quiescence as f64
            );
        }
    }
}

impl_counter_stats_ops!(
    MoveStats,
    main_searched,
    quiescence_searched,
    illegal_main,
    illegal_quiescence,
);
