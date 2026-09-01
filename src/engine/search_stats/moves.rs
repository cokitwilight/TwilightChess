use crate::engine::search_stats::NodeStats;
use crate::engine::search_stats::formatting::{metric, metric_count, metric_pct, section};

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

        section("Moves");
        metric_count("Main searched", self.main_searched);
        metric_count("Quiescence searched", self.quiescence_searched);
        metric_count("Total searched", total_moves);
        if illegal_total > 0 {
            metric_count("Illegal main pseudo-moves", self.illegal_main);
            metric_count("Illegal q pseudo-moves", self.illegal_quiescence);
        }

        let pseudo_total = total_moves + illegal_total;
        if pseudo_total > 0 {
            metric_pct("Illegal pseudo-move rate", illegal_total, pseudo_total);
        }
        if nodes.main > 0 {
            metric(
                "Moves per main node",
                format!("{:.2}", self.main_searched as f64 / nodes.main as f64),
            );
        }
        if nodes.quiescence > 0 {
            metric(
                "Moves per qnode",
                format!(
                    "{:.2}",
                    self.quiescence_searched as f64 / nodes.quiescence as f64
                ),
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
