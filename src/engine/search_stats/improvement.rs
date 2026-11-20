use std::ops::{AddAssign, Sub};

use super::formatting::{metric, metric_count, section};

/// Raw same-side evaluation deltas at negamax's improving check.
/// Unavailable comparisons use the search's existing zero-delta fallback.
#[derive(Clone, Copy, Debug, Default)]
#[cfg_attr(feature = "debug-uci", derive(serde::Serialize))]
pub struct ImprovementStats {
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

    pub(super) fn print(&self) {
        let Some(average) = self.average_improvement() else {
            return;
        };
        section("Evaluation Improvement");
        metric_count("Improving (> 0)", self.improving);
        metric_count("Non-improving (<= 0)", self.non_improving);
        metric("Average improvement (cp)", format!("{average:.2}"));
        metric(
            "Max improvement magnitude (cp)",
            self.highest_improvement
                .map_or_else(|| "n/a".into(), |v| v.to_string()),
        );
        metric(
            "Lowest improvement (cp)",
            self.lowest_improvement
                .map_or_else(|| "n/a".into(), |v| v.to_string()),
        );
    }
}

impl AddAssign for ImprovementStats {
    fn add_assign(&mut self, rhs: Self) {
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
