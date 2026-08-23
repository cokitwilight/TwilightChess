use super::{MoveStats, NodeStats, count, pct};

#[derive(Clone, Copy, Debug, Default)]
pub struct CutoffStats {
    pub beta: u64,
    pub quiescence_beta: u64,
    pub stand_pat: u64,
    pub first_move_beta: u64,
    pub first_move_quiescence_beta: u64,
}

impl CutoffStats {
    pub(super) fn print(&self, nodes: &NodeStats, moves: &MoveStats) {
        if self.beta == 0 && self.stand_pat == 0 {
            return;
        }

        println!("Cutoffs");
        println!("  {:<22} {:>14}", "Beta cutoffs:", count(self.beta));
        if self.first_move_beta > 0 && self.beta > 0 {
            println!(
                "  {:<22} {:>14}",
                "First Move Beta cutoffs:",
                count(self.first_move_beta)
            );
            println!(
                "  {:<22} {:>14}",
                "First Move Beta / Beta:",
                pct(self.first_move_beta, self.beta)
            );
        }

        println!(
            "  {:<22} {:>14}",
            "Q Beta cutoffs:",
            count(self.quiescence_beta)
        );
        if self.first_move_quiescence_beta > 0 && self.quiescence_beta > 0 {
            println!(
                "  {:<22} {:>14}",
                "First Q Move Beta cutoffs:",
                count(self.first_move_quiescence_beta)
            );
            println!(
                "  {:<22} {:>14}",
                "First Move Q Beta / Q Beta:",
                pct(self.first_move_quiescence_beta, self.quiescence_beta)
            );
        }

        println!(
            "  {:<22} {:>14}",
            "Stand-pat cutoffs:",
            count(self.stand_pat)
        );
        if moves.main_searched > 0 {
            println!(
                "  {:<22} {:>14}",
                "Beta / main moves:",
                pct(self.beta, moves.main_searched)
            );
        }
        if nodes.quiescence > 0 {
            println!(
                "  {:<22} {:>14}",
                "Q-Beta / q moves:",
                pct(self.quiescence_beta, moves.quiescence_searched)
            );
            println!(
                "  {:<22} {:>14}",
                "Stand-pat / qnodes:",
                pct(self.stand_pat, nodes.quiescence)
            );
        }

        println!("Total");
        println!(
            "  {:<22} {:>14}",
            "Beta cutoffs:",
            count(self.beta + self.quiescence_beta)
        );
        println!(
            "  {:<22} {:>14}",
            "Beta / Move",
            pct(
                self.beta + self.quiescence_beta,
                moves.main_searched + moves.quiescence_searched,
            )
        );
    }
}

impl_counter_stats_ops!(
    CutoffStats,
    beta,
    quiescence_beta,
    stand_pat,
    first_move_beta,
    first_move_quiescence_beta,
);
