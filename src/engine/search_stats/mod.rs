use std::ops::{AddAssign, Sub};

macro_rules! impl_counter_stats_ops {
    ($stats:ty, $($field:ident),+ $(,)?) => {
        impl std::ops::Sub for $stats {
            type Output = Self;

            fn sub(self, rhs: Self) -> Self {
                Self {
                    $($field: self.$field.saturating_sub(rhs.$field)),+
                }
            }
        }

        impl std::ops::AddAssign for $stats {
            fn add_assign(&mut self, rhs: Self) {
                $(self.$field += rhs.$field;)+
            }
        }
    };
}

mod aspiration;
mod correction_history;
mod cutoffs;
mod draws;
mod formatting;
mod futility;
mod histogram;
mod history;
mod iterative_deepening;
mod lmp;
mod lmr;
mod move_ordering;
mod moves;
mod nodes;
mod null_move;
mod pvs;
mod quiescence;
mod quiescence_pruning;
mod reverse_futility;
mod singular_extensions;
mod terminal;
mod transposition_table;

pub use aspiration::AspirationStats;
pub use correction_history::{
    CORRECTION_BUCKET_WIDTH_CP, CorrectionHistoryStats, CorrectionUpdateKind,
};
pub use cutoffs::CutoffStats;
pub use draws::DrawStats;
pub use futility::FutilityStats;
pub use histogram::{
    CORRECTION_MAGNITUDE_BUCKETS, CorrectionMagnitudeHistogram, DepthHistogram, Histogram,
    MAX_TRACKED_DEPTH, MOVE_INDEX_BUCKETS, MoveIndexHistogram, REDUCTION_BUCKETS,
    ReductionHistogram,
};
pub use history::HistoryStats;
pub use iterative_deepening::IterativeDeepeningStats;
pub use lmp::LmpStats;
pub use lmr::LmrStats;
pub use move_ordering::MoveOrderingStats;
pub use moves::MoveStats;
pub use nodes::NodeStats;
pub use null_move::NullMoveStats;
pub use pvs::PvsStats;
pub use quiescence::QuiescenceStats;
pub use quiescence_pruning::QuiescencePruningStats;
pub use reverse_futility::ReverseFutilityStats;
pub use singular_extensions::SingularExtensionStats;
pub use terminal::TerminalStats;
pub use transposition_table::TranspositionTableStats;

use formatting::{count, report_footer, report_header, section};

/// Statistics collected during a search, grouped by the feature that owns them.
///
/// New counters belong in the relevant feature module. New feature groups only
/// need a field here and entries in the two arithmetic implementations below.
#[derive(Clone, Copy, Debug, Default)]
pub struct SearchStats {
    pub node_stats: NodeStats,
    pub iterative_deepening_stats: IterativeDeepeningStats,
    pub move_stats: MoveStats,
    pub cutoff_stats: CutoffStats,
    pub aspiration_stats: AspirationStats,
    pub pvs_stats: PvsStats,
    pub move_ordering_stats: MoveOrderingStats,
    pub history_stats: HistoryStats,
    pub correction_stats: CorrectionHistoryStats,
    pub lmp_stats: LmpStats,
    pub lmr_stats: LmrStats,
    pub rfp_stats: ReverseFutilityStats,
    pub fut_stats: FutilityStats,
    pub null_move_stats: NullMoveStats,
    pub q_pruning_stats: QuiescencePruningStats,
    pub quiescence_stats: QuiescenceStats,
    pub draw_stats: DrawStats,
    pub terminal_stats: TerminalStats,
    pub singular_stats: SingularExtensionStats,
    pub tt_stats: TranspositionTableStats,
}

impl SearchStats {
    #[inline]
    pub fn total_nodes(&self) -> u64 {
        self.node_stats.total()
    }

    pub fn print_all(&self, depth: u16, elapsed_secs: f64) {
        self.print_report(&format!("Search Statistics — Depth {depth}"), elapsed_secs);
    }

    pub fn print_all_with_title(&self, title: &str, elapsed_secs: f64) {
        self.print_report(title, elapsed_secs);
    }

    fn print_report(&self, title: &str, elapsed_secs: f64) {
        report_header(title);

        self.print_nodes(elapsed_secs);
        self.print_iterative_deepening();
        self.print_moves();
        self.print_cutoffs();
        self.print_move_ordering();
        self.print_pvs();
        self.print_aspiration();
        self.print_history();
        self.print_correction_history();
        self.print_reductions();
        self.print_main_pruning();
        self.print_quiescence();
        self.print_q_pruning();
        self.print_singular_extensions();
        self.print_terminal();
        self.print_returns();
        self.print_tts();

        report_footer();
    }

    pub fn print_nodes(&self, elapsed_secs: f64) {
        self.node_stats.print(elapsed_secs);
    }

    pub fn print_iterative_deepening(&self) {
        self.iterative_deepening_stats.print();
    }

    pub fn print_moves(&self) {
        self.move_stats.print(&self.node_stats);
    }

    pub fn print_cutoffs(&self) {
        self.cutoff_stats.print(&self.node_stats, &self.move_stats);
    }

    pub fn print_aspiration(&self) {
        self.aspiration_stats.print();
    }

    pub fn print_pvs(&self) {
        self.pvs_stats.print();
    }

    pub fn print_move_ordering(&self) {
        self.move_ordering_stats.print();
    }

    pub fn print_history(&self) {
        self.history_stats.print();
    }

    pub fn print_correction_history(&self) {
        self.correction_stats.print();
    }

    pub fn print_reductions(&self) {
        self.lmr_stats.print();
    }

    pub fn print_main_pruning(&self) {
        if !self.rfp_stats.has_data()
            && !self.fut_stats.has_data()
            && !self.lmp_stats.has_data()
            && !self.null_move_stats.has_data()
        {
            return;
        }

        section("Main-Search Pruning");
        self.rfp_stats.print();
        self.lmp_stats.print();
        self.fut_stats.print();
        self.null_move_stats.print();
    }

    pub fn print_q_pruning(&self) {
        self.q_pruning_stats.print();
    }

    pub fn print_quiescence(&self) {
        self.quiescence_stats.print();
    }

    // Kept as a convenience wrapper for existing callers.
    pub fn print_pruning(&self) {
        self.print_reductions();
        self.print_main_pruning();
        self.print_q_pruning();
    }

    pub fn print_singular_extensions(&self) {
        self.singular_stats.print(self.total_nodes());
    }

    pub fn print_returns(&self) {
        self.draw_stats.print();
    }

    pub fn print_terminal(&self) {
        self.terminal_stats.print();
    }

    pub fn print_tts(&self) {
        self.tt_stats.print();
    }
}

impl Sub for SearchStats {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        Self {
            node_stats: self.node_stats - rhs.node_stats,
            iterative_deepening_stats: self.iterative_deepening_stats
                - rhs.iterative_deepening_stats,
            move_stats: self.move_stats - rhs.move_stats,
            cutoff_stats: self.cutoff_stats - rhs.cutoff_stats,
            aspiration_stats: self.aspiration_stats - rhs.aspiration_stats,
            pvs_stats: self.pvs_stats - rhs.pvs_stats,
            move_ordering_stats: self.move_ordering_stats - rhs.move_ordering_stats,
            history_stats: self.history_stats - rhs.history_stats,
            correction_stats: self.correction_stats - rhs.correction_stats,
            lmp_stats: self.lmp_stats - rhs.lmp_stats,
            lmr_stats: self.lmr_stats - rhs.lmr_stats,
            rfp_stats: self.rfp_stats - rhs.rfp_stats,
            fut_stats: self.fut_stats - rhs.fut_stats,
            null_move_stats: self.null_move_stats - rhs.null_move_stats,
            q_pruning_stats: self.q_pruning_stats - rhs.q_pruning_stats,
            quiescence_stats: self.quiescence_stats - rhs.quiescence_stats,
            draw_stats: self.draw_stats - rhs.draw_stats,
            terminal_stats: self.terminal_stats - rhs.terminal_stats,
            singular_stats: self.singular_stats - rhs.singular_stats,
            tt_stats: self.tt_stats - rhs.tt_stats,
        }
    }
}

impl AddAssign for SearchStats {
    fn add_assign(&mut self, rhs: Self) {
        self.node_stats += rhs.node_stats;
        self.iterative_deepening_stats += rhs.iterative_deepening_stats;
        self.move_stats += rhs.move_stats;
        self.cutoff_stats += rhs.cutoff_stats;
        self.aspiration_stats += rhs.aspiration_stats;
        self.pvs_stats += rhs.pvs_stats;
        self.move_ordering_stats += rhs.move_ordering_stats;
        self.history_stats += rhs.history_stats;
        self.correction_stats += rhs.correction_stats;
        self.lmp_stats += rhs.lmp_stats;
        self.lmr_stats += rhs.lmr_stats;
        self.rfp_stats += rhs.rfp_stats;
        self.fut_stats += rhs.fut_stats;
        self.null_move_stats += rhs.null_move_stats;
        self.q_pruning_stats += rhs.q_pruning_stats;
        self.quiescence_stats += rhs.quiescence_stats;
        self.draw_stats += rhs.draw_stats;
        self.terminal_stats += rhs.terminal_stats;
        self.singular_stats += rhs.singular_stats;
        self.tt_stats += rhs.tt_stats;
    }
}

pub fn median_f64(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }

    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let mid = sorted.len() / 2;

    if sorted.len().is_multiple_of(2) {
        (sorted[mid - 1] + sorted[mid]) / 2.0
    } else {
        sorted[mid]
    }
}

pub fn fmt_nps(value: f64) -> String {
    if !value.is_finite() || value <= 0.0 {
        "0".to_string()
    } else {
        count(value as u64)
    }
}

#[cfg(test)]
mod tests {
    use super::SearchStats;
    use crate::engine::configs::EngineConfig;
    use crate::engine::{Engine, SearchLimits};
    use crate::game::Game;

    #[test]
    fn grouped_stats_aggregate_and_subtract_per_feature() {
        let mut total = SearchStats::default();
        total.node_stats.main = 100;
        total.lmp_stats.attempts = 8;
        total.lmp_stats.history_rejections = 3;
        total.lmp_stats.pruned_moves = 5;
        total.lmr_stats.attempts = 12;
        total.lmr_stats.capture_adjustments = 4;
        total.lmr_stats.capture_effects = 3;
        total.lmr_stats.dynamic_searches = 5;
        total.lmr_stats.dynamic_searches_at_modified_depth = 4;
        total.fut_stats.pruned_moves = 7;
        total.correction_stats.record(8);
        total.correction_stats.record(-16);

        let mut additional = SearchStats::default();
        additional.node_stats.main = 25;
        additional.lmp_stats.attempts = 2;
        additional.lmp_stats.history_rejections = 1;
        additional.lmp_stats.pruned_moves = 1;
        additional.lmr_stats.attempts = 3;
        additional.lmr_stats.capture_adjustments = 1;
        additional.lmr_stats.capture_effects = 1;
        additional.lmr_stats.dynamic_searches = 2;
        additional.lmr_stats.dynamic_searches_at_modified_depth = 1;
        additional.fut_stats.pruned_moves = 2;
        additional.correction_stats.record(25);

        total += additional;
        let delta = total - additional;

        assert_eq!(total.total_nodes(), 125);
        assert_eq!(total.lmp_stats.attempts, 10);
        assert_eq!(total.lmp_stats.history_rejections, 4);
        assert_eq!(total.lmp_stats.pruned_moves, 6);
        assert_eq!(total.lmr_stats.attempts, 15);
        assert_eq!(total.lmr_stats.capture_adjustments, 5);
        assert_eq!(total.lmr_stats.capture_effects, 4);
        assert_eq!(total.lmr_stats.dynamic_searches, 7);
        assert_eq!(total.lmr_stats.dynamic_searches_at_modified_depth, 5);
        assert_eq!(total.fut_stats.pruned_moves, 9);
        assert_eq!(total.correction_stats.lookups, 3);
        assert_eq!(total.correction_stats.magnitude_histogram.total(), 3);
        assert_eq!(delta.node_stats.main, 100);
        assert_eq!(delta.lmp_stats.attempts, 8);
        assert_eq!(delta.lmp_stats.history_rejections, 3);
        assert_eq!(delta.lmp_stats.pruned_moves, 5);
        assert_eq!(delta.lmr_stats.attempts, 12);
        assert_eq!(delta.lmr_stats.capture_adjustments, 4);
        assert_eq!(delta.lmr_stats.capture_effects, 3);
        assert_eq!(delta.lmr_stats.dynamic_searches, 5);
        assert_eq!(delta.lmr_stats.dynamic_searches_at_modified_depth, 4);
        assert_eq!(delta.fut_stats.pruned_moves, 7);
        assert_eq!(delta.correction_stats.lookups, 2);
        assert_eq!(delta.correction_stats.positive_corrections, 1);
        assert_eq!(delta.correction_stats.negative_corrections, 1);
        assert_eq!(delta.correction_stats.magnitude_histogram.total(), 2);
    }

    #[test]
    fn grouped_stats_subtraction_saturates() {
        let mut smaller = SearchStats::default();
        smaller.lmr_stats.researches = 1;

        let mut larger = SearchStats::default();
        larger.lmr_stats.researches = 2;

        assert_eq!((smaller - larger).lmr_stats.researches, 0);
    }

    #[test]
    fn histogram_aggregation_and_all_reports_are_valid() {
        let mut stats = SearchStats::default();
        stats.node_stats.main = 10;
        stats.node_stats.quiescence = 5;
        stats.aspiration_stats.searches = 1;
        stats.aspiration_stats.successful_first_windows = 1;
        stats.pvs_stats.null_window_searches = 4;
        stats.pvs_stats.full_window_researches = 1;
        stats.move_stats.main_searched = 8;
        stats.move_stats.quiescence_searched = 4;
        stats.cutoff_stats.beta = 2;
        stats.cutoff_stats.quiescence_beta = 2;
        stats.cutoff_stats.first_move_beta = 1;
        stats.cutoff_stats.first_move_quiescence_beta = 1;
        stats.cutoff_stats.stand_pat = 1;
        stats.move_ordering_stats.cutoff_move_index_sum = 4;
        stats.move_ordering_stats.cutoff_move_index_histogram.bins[0] = 1;
        stats.move_ordering_stats.cutoff_move_index_histogram.bins[2] = 1;
        stats.move_ordering_stats.killer_move_cutoffs = 1;
        stats.history_stats.bonus_updates = 2;
        stats.history_stats.malus_updates = 1;
        stats.history_stats.continuation_bonus_updates = 1;
        for correction in [0, 8, -17, 101] {
            stats.correction_stats.record(correction);
        }
        stats.lmr_stats.attempts = 2;
        stats.lmr_stats.reduction_histogram.bins[1] = 2;
        stats.lmr_stats.attempts_by_depth.bins[4] = 2;
        stats.lmr_stats.researches_by_depth.bins[4] = 1;
        stats.lmr_stats.non_improving_adjustments = 2;
        stats.lmr_stats.non_improving_effects = 1;
        stats.lmr_stats.dynamic_depth_decreases = 1;
        stats.lmr_stats.dynamic_searches = 1;
        stats.lmr_stats.dynamic_searches_at_modified_depth = 1;
        stats.rfp_stats.attempts = 2;
        stats.rfp_stats.cutoffs = 1;
        stats.rfp_stats.attempts_by_depth.bins[3] = 2;
        stats.rfp_stats.cutoffs_by_depth.bins[3] = 1;
        stats.quiescence_stats.normal_nodes = 5;
        stats.q_pruning_stats.delta_attempts = 2;
        stats.q_pruning_stats.delta_prunes = 1;
        stats.terminal_stats.checkmates = 1;
        stats.tt_stats.main.probes = 1;

        let mut combined = SearchStats::default();
        combined += stats;

        assert_eq!(combined.lmr_stats.reduction_histogram.bins[1], 2);
        assert_eq!(combined.rfp_stats.attempts_by_depth.bins[3], 2);
        assert_eq!(
            combined
                .move_ordering_stats
                .cutoff_move_index_histogram
                .highest_nonzero_bucket(),
            Some(2)
        );
        assert_eq!(combined.correction_stats.lookups, 4);
        assert_eq!(
            combined.correction_stats.magnitude_histogram.total(),
            combined.correction_stats.nonzero_corrections()
        );

        combined.print_all(4, 1.0);
    }

    #[test]
    fn instrumented_search_keeps_histogram_totals_consistent() {
        let game = Game::new();
        let mut config = EngineConfig::default();
        config.tt_size = 1;
        config.search.lmp.enabled = true;
        let mut engine = Engine::new(config);

        let result = engine.search(
            &game.board,
            SearchLimits::depth(4, 4),
            &game.repetition_history,
            false,
            false,
        );
        let stats = result.stats;

        assert!(stats.total_nodes() > 0);
        assert_eq!(
            stats.iterative_deepening_stats.nodes_by_depth.total(),
            stats.total_nodes()
        );
        for depth in 1..=4 {
            assert_eq!(stats.iterative_deepening_stats.samples_at_depth(depth), 1);
            assert!(stats.iterative_deepening_stats.nodes_at_depth(depth) > 0);
        }
        assert!(
            stats
                .iterative_deepening_stats
                .estimated_growth_factor(2)
                .is_some()
        );
        assert!(stats.pvs_stats.null_window_searches > 0);
        assert_eq!(
            stats
                .move_ordering_stats
                .cutoff_move_index_histogram
                .total(),
            stats.cutoff_stats.beta
        );
        assert_eq!(
            stats.lmr_stats.reduction_histogram.total(),
            stats.lmr_stats.attempts
        );
        assert_eq!(
            stats.lmr_stats.attempts_by_depth.total(),
            stats.lmr_stats.attempts
        );
        assert_eq!(
            stats.lmr_stats.dynamic_depth_increases
                + stats.lmr_stats.dynamic_depth_decreases
                + stats.lmr_stats.dynamic_depth_unchanged,
            stats.lmr_stats.researches
        );
        assert!(
            stats.lmr_stats.dynamic_searches_at_modified_depth <= stats.lmr_stats.dynamic_searches
        );
        assert_eq!(
            stats.rfp_stats.attempts_by_depth.total(),
            stats.rfp_stats.attempts
        );
        assert_eq!(
            stats.fut_stats.attempts_by_depth.total(),
            stats.fut_stats.attempts
        );
        assert!(stats.lmp_stats.attempts > 0);
        assert_eq!(
            stats.lmp_stats.attempts,
            stats.lmp_stats.history_rejections + stats.lmp_stats.pruned_moves
        );
        assert_eq!(
            stats.null_move_stats.attempts_by_depth.total(),
            stats.null_move_stats.attempts
        );
        assert_eq!(
            stats.tt_stats.main.exact_hits
                + stats.tt_stats.main.lower_bound_hits
                + stats.tt_stats.main.upper_bound_hits,
            stats.tt_stats.main.hits
        );
        assert!(stats.correction_stats.lookups > 0);
        assert_eq!(
            stats.correction_stats.magnitude_histogram.total(),
            stats.correction_stats.nonzero_corrections()
        );
        assert_eq!(
            stats.correction_stats.lookups,
            stats.correction_stats.main_lookups + stats.correction_stats.quiescence_lookups
        );
        assert_eq!(
            stats.correction_stats.positive_magnitude_histogram.total(),
            stats.correction_stats.positive_corrections
        );
        assert_eq!(
            stats.correction_stats.negative_magnitude_histogram.total(),
            stats.correction_stats.negative_corrections
        );
        assert_eq!(
            stats.correction_stats.update_calls,
            stats.correction_stats.exact_updates
                + stats.correction_stats.fail_high_updates
                + stats.correction_stats.fail_low_updates
        );
        assert_eq!(
            stats.correction_stats.update_calls,
            stats.correction_stats.positive_update_deltas
                + stats.correction_stats.negative_update_deltas
                + stats.correction_stats.zero_update_deltas
        );
        assert!(stats.correction_stats.nonzero_corrections() <= stats.correction_stats.lookups);
    }
}
