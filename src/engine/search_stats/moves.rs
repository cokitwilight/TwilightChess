use crate::engine::search_stats::NodeStats;
use crate::engine::search_stats::formatting::{
    count, metric, metric_count, metric_pct, section, subsection,
};

#[derive(Clone, Copy, Debug, Default)]
#[cfg_attr(feature = "debug-uci", derive(serde::Serialize))]
pub struct MoveStats {
    pub main_searched: u64,
    pub quiescence_searched: u64,
    pub illegal_main: u64,
    pub illegal_quiescence: u64,
    pub picked: u64,
    pub missing_see: u64,
    pub missing_captured: u64,
    pub missing_history_key: u64,
    pub non_special_picked: u64,
    pub non_special_missing_see: u64,
    pub non_special_missing_captured: u64,
    pub non_special_missing_history_key: u64,
}

impl MoveStats {
    pub(super) fn print(&self, nodes: &NodeStats) {
        let total_moves = self.main_searched + self.quiescence_searched;
        let illegal_total = self.illegal_main + self.illegal_quiescence;

        if total_moves == 0 && illegal_total == 0 && self.picked == 0 {
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
        if self.picked > 0 {
            subsection("Move picker metadata");
            println!(
                "    {:<23} {:>12} {:>20}",
                "", "All moves", "Excl. PV/TT/killers"
            );
            for (label, all, non_special) in [
                ("Moves returned", self.picked, self.non_special_picked),
                (
                    "Missing SEE",
                    self.missing_see,
                    self.non_special_missing_see,
                ),
                (
                    "Missing captured piece",
                    self.missing_captured,
                    self.non_special_missing_captured,
                ),
                (
                    "Missing history key",
                    self.missing_history_key,
                    self.non_special_missing_history_key,
                ),
            ] {
                println!(
                    "    {label:<23} {:>12} {:>20}",
                    count(all),
                    count(non_special)
                );
            }
        }
    }
}

impl_counter_stats_ops!(
    MoveStats,
    main_searched,
    quiescence_searched,
    illegal_main,
    illegal_quiescence,
    picked,
    missing_see,
    missing_captured,
    missing_history_key,
    non_special_picked,
    non_special_missing_see,
    non_special_missing_captured,
    non_special_missing_history_key,
);
