use super::{
    MoveStats, NodeStats,
    formatting::{metric_count, metric_pct, section, submetric_count, submetric_pct, subsection},
};

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
        if self.beta == 0 && self.quiescence_beta == 0 && self.stand_pat == 0 {
            return;
        }

        section("Cutoffs");
        subsection("Main search");
        submetric_count("Beta cutoffs", self.beta);
        if moves.main_searched > 0 {
            submetric_pct(
                "Cutoff rate / searched moves",
                self.beta,
                moves.main_searched,
            );
        }

        subsection("Quiescence");
        submetric_count("Beta cutoffs", self.quiescence_beta);
        submetric_count("First-move beta cutoffs", self.first_move_quiescence_beta);
        submetric_count("Stand-pat cutoffs", self.stand_pat);
        if nodes.quiescence > 0 {
            submetric_pct(
                "Beta rate / searched moves",
                self.quiescence_beta,
                moves.quiescence_searched,
            );
            submetric_pct("Stand-pat rate / qnodes", self.stand_pat, nodes.quiescence);
        }
        if self.quiescence_beta > 0 {
            submetric_pct(
                "First-move beta rate",
                self.first_move_quiescence_beta,
                self.quiescence_beta,
            );
        }

        let total_beta = self.beta + self.quiescence_beta;
        metric_count("Combined beta cutoffs", total_beta);
        metric_pct(
            "Combined beta rate",
            total_beta,
            moves.main_searched + moves.quiescence_searched,
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
