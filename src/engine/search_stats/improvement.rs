use std::ops::{AddAssign, Sub};

use super::formatting::{
    metric_count, metric_pct, section, submetric, submetric_count, submetric_pct, subsection,
};

/// Improvement deltas for one exclusive improving category.
#[derive(Clone, Copy, Debug, Default)]
#[cfg_attr(feature = "debug-uci", derive(serde::Serialize))]
pub struct ImprovementGroupStats {
    pub nodes: u64,
    pub improvement_sum: i64,
    pub max_improvement: Option<i32>,
    pub min_improvement: Option<i32>,
}

impl ImprovementGroupStats {
    #[inline]
    pub fn record(&mut self, improvement: i32) {
        self.nodes += 1;
        self.improvement_sum += i64::from(improvement);
        self.max_improvement = Some(
            self.max_improvement
                .map_or(improvement, |v| v.max(improvement)),
        );
        self.min_improvement = Some(
            self.min_improvement
                .map_or(improvement, |v| v.min(improvement)),
        );
    }

    pub fn average_improvement(&self) -> Option<f64> {
        (self.nodes > 0).then(|| self.improvement_sum as f64 / self.nodes as f64)
    }

    fn print(&self, label: &str, samples: u64) {
        subsection(label);
        submetric_count("Nodes", self.nodes);
        submetric_pct("% of calculated nodes", self.nodes, samples);
        submetric(
            "Average improvement (cp)",
            self.average_improvement()
                .map_or_else(|| "n/a".into(), |v| format!("{v:.2}")),
        );
        submetric(
            "Max improvement (cp)",
            self.max_improvement
                .map_or_else(|| "n/a".into(), |v| v.to_string()),
        );
        submetric(
            "Min improvement (cp)",
            self.min_improvement
                .map_or_else(|| "n/a".into(), |v| v.to_string()),
        );
    }
}

impl AddAssign for ImprovementGroupStats {
    fn add_assign(&mut self, rhs: Self) {
        self.nodes += rhs.nodes;
        self.improvement_sum += rhs.improvement_sum;
        self.max_improvement = self.max_improvement.max(rhs.max_improvement);
        self.min_improvement = match (self.min_improvement, rhs.min_improvement) {
            (Some(a), Some(b)) => Some(a.min(b)),
            (a, b) => a.or(b),
        };
    }
}

impl Sub for ImprovementGroupStats {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        // Like the aggregate, exact extrema require an empty RHS snapshot.
        Self {
            nodes: self.nodes.saturating_sub(rhs.nodes),
            improvement_sum: self.improvement_sum - rhs.improvement_sum,
            max_improvement: (rhs.nodes == 0).then_some(self.max_improvement).flatten(),
            min_improvement: (rhs.nodes == 0).then_some(self.min_improvement).flatten(),
        }
    }
}

/// Raw same-side evaluation deltas at negamax's improving check.
/// Unavailable comparisons use the search's existing zero-delta fallback.
#[derive(Clone, Copy, Debug, Default)]
#[cfg_attr(feature = "debug-uci", derive(serde::Serialize))]
pub struct ImprovementStats {
    pub strong: ImprovementGroupStats,
    /// Weak only: strong nodes are excluded even though they satisfy both flags.
    pub weak: ImprovementGroupStats,
    /// Invalid stored ancestor evaluations at nodes reaching the calculation.
    /// Nonexistent ancestors (shallow nodes) are excluded.
    pub invalid_previous_ply_eval: u64,
    pub invalid_previous_2_ply_eval: u64,
    pub invalid_previous_4_ply_eval: u64,
    /// Nodes reaching the calculation without enough ancestor plies.
    /// These overlap: a root node increments all three counters.
    pub ply_below_1: u64,
    pub ply_below_2: u64,
    pub ply_below_4: u64,
    pub improving: u64,
    pub non_improving: u64,
    pub improvement_sum: i64,
    /// Largest absolute evaluation delta, in centipawns.
    pub highest_improvement: Option<u32>,
    pub lowest_improvement: Option<i32>,
}

impl ImprovementStats {
    #[inline]
    pub fn samples(&self) -> u64 {
        self.improving + self.non_improving
    }

    #[inline]
    pub fn record(&mut self, improvement: i32) {
        if improvement > 0 {
            self.improving += 1;
        } else {
            self.non_improving += 1;
        }
        self.improvement_sum += i64::from(improvement);
        let magnitude = improvement.unsigned_abs();
        self.highest_improvement = Some(
            self.highest_improvement
                .map_or(magnitude, |highest| highest.max(magnitude)),
        );
        self.lowest_improvement = Some(
            self.lowest_improvement
                .map_or(improvement, |lowest| lowest.min(improvement)),
        );
    }

    pub fn average_improvement(&self) -> Option<f64> {
        let samples = self.samples();
        (samples > 0).then(|| self.improvement_sum as f64 / samples as f64)
    }

    /// Main-search calls returning or entering qsearch before the calculation.
    pub fn not_reached_nodes(&self, main_nodes: u64) -> u64 {
        main_nodes.saturating_sub(self.samples())
    }

    pub fn not_reached_percentage(&self, main_nodes: u64) -> f64 {
        if main_nodes == 0 {
            0.0
        } else {
            self.not_reached_nodes(main_nodes) as f64 * 100.0 / main_nodes as f64
        }
    }

    pub(super) fn print(&self, main_nodes: u64) {
        if self.samples() == 0 {
            return;
        }

        section("Evaluation Improvement");
        metric_count("Reached calculation", self.samples());
        metric_count(
            "Main nodes not reaching calc",
            self.not_reached_nodes(main_nodes),
        );
        metric_pct("% main nodes reaching calc", self.samples(), main_nodes);
        metric_count("Insufficient history: ply < 1", self.ply_below_1);
        metric_count("Insufficient history: ply < 2", self.ply_below_2);
        metric_count("Insufficient history: ply < 4", self.ply_below_4);
        metric_count("Invalid eval: previous ply", self.invalid_previous_ply_eval);
        metric_count(
            "Invalid eval: 2 plies back",
            self.invalid_previous_2_ply_eval,
        );
        metric_count(
            "Invalid eval: 4 plies back",
            self.invalid_previous_4_ply_eval,
        );
        self.strong.print("Strong improving", self.samples());
        self.weak.print("Weak improving only", self.samples());
        subsection("All calculated nodes");
        submetric_count("Improving (> 0)", self.improving);
        submetric_count("Non-improving (<= 0)", self.non_improving);
        submetric(
            "Average improvement (cp)",
            self.average_improvement()
                .map_or_else(|| "n/a".into(), |v| format!("{v:.2}")),
        );
        submetric(
            "Max magnitude (cp)",
            self.highest_improvement
                .map_or_else(|| "n/a".into(), |v| v.to_string()),
        );
        submetric(
            "Lowest improvement (cp)",
            self.lowest_improvement
                .map_or_else(|| "n/a".into(), |v| v.to_string()),
        );
    }
}

impl AddAssign for ImprovementStats {
    fn add_assign(&mut self, rhs: Self) {
        self.strong += rhs.strong;
        self.weak += rhs.weak;
        self.invalid_previous_ply_eval += rhs.invalid_previous_ply_eval;
        self.invalid_previous_2_ply_eval += rhs.invalid_previous_2_ply_eval;
        self.invalid_previous_4_ply_eval += rhs.invalid_previous_4_ply_eval;
        self.ply_below_1 += rhs.ply_below_1;
        self.ply_below_2 += rhs.ply_below_2;
        self.ply_below_4 += rhs.ply_below_4;
        self.improving += rhs.improving;
        self.non_improving += rhs.non_improving;
        self.improvement_sum += rhs.improvement_sum;
        self.highest_improvement = self.highest_improvement.max(rhs.highest_improvement);
        self.lowest_improvement = match (self.lowest_improvement, rhs.lowest_improvement) {
            (Some(a), Some(b)) => Some(a.min(b)),
            (a, b) => a.or(b),
        };
    }
}

impl Sub for ImprovementStats {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        // Extrema cannot be recovered from cumulative snapshots. Iterative
        // deepening collects this group separately per iteration so its RHS
        // is empty and the exact iteration extrema are retained.
        let keep_extrema = rhs.samples() == 0;
        Self {
            strong: self.strong - rhs.strong,
            weak: self.weak - rhs.weak,
            invalid_previous_ply_eval: self
                .invalid_previous_ply_eval
                .saturating_sub(rhs.invalid_previous_ply_eval),
            invalid_previous_2_ply_eval: self
                .invalid_previous_2_ply_eval
                .saturating_sub(rhs.invalid_previous_2_ply_eval),
            invalid_previous_4_ply_eval: self
                .invalid_previous_4_ply_eval
                .saturating_sub(rhs.invalid_previous_4_ply_eval),
            ply_below_1: self.ply_below_1.saturating_sub(rhs.ply_below_1),
            ply_below_2: self.ply_below_2.saturating_sub(rhs.ply_below_2),
            ply_below_4: self.ply_below_4.saturating_sub(rhs.ply_below_4),
            improving: self.improving.saturating_sub(rhs.improving),
            non_improving: self.non_improving.saturating_sub(rhs.non_improving),
            improvement_sum: self.improvement_sum - rhs.improvement_sum,
            highest_improvement: keep_extrema.then_some(self.highest_improvement).flatten(),
            lowest_improvement: keep_extrema.then_some(self.lowest_improvement).flatten(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn group_summaries_merge_and_iteration_deltas_keep_exact_extrema() {
        let mut before = ImprovementStats::default();
        before.record(10);
        before.strong.record(10);
        before.invalid_previous_ply_eval = 1;
        before.ply_below_1 = 1;
        before.ply_below_2 = 1;
        before.ply_below_4 = 1;
        let mut iteration = ImprovementStats::default();
        for value in [20, 60] {
            iteration.record(value);
            iteration.strong.record(value);
        }
        for value in [5, 15] {
            iteration.record(value);
            iteration.weak.record(value);
        }
        iteration.invalid_previous_2_ply_eval = 2;
        iteration.ply_below_1 = 1;
        iteration.ply_below_2 = 2;
        iteration.ply_below_4 = 3;
        assert_eq!(iteration.strong.average_improvement(), Some(40.0));
        assert_eq!(iteration.strong.min_improvement, Some(20));
        assert_eq!(iteration.strong.max_improvement, Some(60));
        assert_eq!(iteration.weak.average_improvement(), Some(10.0));
        assert_eq!(iteration.weak.min_improvement, Some(5));
        assert_eq!(iteration.weak.max_improvement, Some(15));
        assert_eq!(iteration.not_reached_nodes(10), 6);
        assert_eq!(iteration.not_reached_percentage(10), 60.0);
        assert_eq!(ImprovementStats::default().not_reached_percentage(0), 0.0);
        let delta = iteration - ImprovementStats::default();
        assert_eq!(delta.strong.min_improvement, Some(20));
        assert_eq!(delta.weak.max_improvement, Some(15));
        let mut combined = before;
        combined += iteration;
        assert_eq!(combined.strong.nodes, 3);
        assert_eq!(combined.strong.min_improvement, Some(10));
        assert_eq!(combined.strong.max_improvement, Some(60));
        assert_eq!(
            (
                combined.ply_below_1,
                combined.ply_below_2,
                combined.ply_below_4
            ),
            (2, 3, 4)
        );
        let delta = combined - before;
        assert_eq!(delta.strong.nodes, 2);
        assert_eq!(delta.strong.improvement_sum, 80);
        assert_eq!(delta.strong.min_improvement, None);
        assert_eq!(delta.weak.max_improvement, Some(15));
        assert_eq!(delta.invalid_previous_ply_eval, 0);
        assert_eq!(delta.invalid_previous_2_ply_eval, 2);
        assert_eq!(
            (delta.ply_below_1, delta.ply_below_2, delta.ply_below_4),
            (1, 2, 3)
        );
    }

    #[test]
    fn signed_deltas_classify_zero_and_track_maximum_magnitude() {
        let mut stats = ImprovementStats::default();
        assert_eq!(stats.average_improvement(), None);
        for value in [-40, 0, 10] {
            stats.record(value);
        }
        assert_eq!(stats.improving, 1);
        assert_eq!(stats.non_improving, 2);
        assert_eq!(stats.average_improvement(), Some(-10.0));
        assert_eq!(stats.highest_improvement, Some(40));
        assert_eq!(stats.lowest_improvement, Some(-40));

        let mut positive = ImprovementStats::default();
        positive.record(20);
        positive.record(40);
        assert_eq!(positive.lowest_improvement, Some(20));
        stats += positive;
        assert_eq!(stats.average_improvement(), Some(6.0));
        assert_eq!(stats.highest_improvement, Some(40));
        assert_eq!(stats.lowest_improvement, Some(-40));

        let mut negative = ImprovementStats::default();
        negative.record(-15);
        assert_eq!(negative.highest_improvement, Some(15));
        negative.record(i32::MIN);
        assert_eq!(negative.highest_improvement, Some(i32::MIN.unsigned_abs()));
        let delta = stats - positive;
        assert_eq!(delta.average_improvement(), Some(-10.0));
        assert_eq!(delta.highest_improvement, None);
    }
}
