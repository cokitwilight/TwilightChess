use crate::engine::history::correction_limit_cp;
use crate::engine::search_stats::formatting::{
    count, metric, metric_count, metric_pct, pct, section, submetric_count, submetric_pct,
    subsection,
};
use crate::engine::search_stats::{CORRECTION_MAGNITUDE_BUCKETS, CorrectionMagnitudeHistogram};

pub const CORRECTION_BUCKET_WIDTH_CP: usize = 10;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CorrectionUpdateKind {
    Exact,
    FailHigh,
    FailLow,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct CorrectionHistoryStats {
    pub lookups: u64,
    pub main_lookups: u64,
    pub quiescence_lookups: u64,
    pub positive_corrections: u64,
    pub negative_corrections: u64,
    pub magnitude_sum: u64,
    pub positive_magnitude_sum: u64,
    pub negative_magnitude_sum: u64,
    pub magnitude_histogram: CorrectionMagnitudeHistogram,
    pub positive_magnitude_histogram: CorrectionMagnitudeHistogram,
    pub negative_magnitude_histogram: CorrectionMagnitudeHistogram,
    pub positive_applied_limit_lookups: u64,
    pub negative_applied_limit_lookups: u64,
    pub update_opportunities: u64,
    pub update_calls: u64,
    pub exact_updates: u64,
    pub fail_high_updates: u64,
    pub fail_low_updates: u64,
    pub positive_update_deltas: u64,
    pub negative_update_deltas: u64,
    pub zero_update_deltas: u64,
    pub positive_update_delta_sum: u64,
    pub negative_update_delta_magnitude_sum: u64,
}

impl CorrectionHistoryStats {
    #[inline]
    pub fn record(&mut self, correction: i32) {
        self.record_main_lookup(correction);
    }

    #[inline]
    pub fn record_main_lookup(&mut self, correction: i32) {
        self.main_lookups += 1;
        self.record_lookup(correction);
    }

    #[inline]
    pub fn record_quiescence_lookup(&mut self, correction: i32) {
        self.quiescence_lookups += 1;
        self.record_lookup(correction);
    }

    #[inline]
    fn record_lookup(&mut self, correction: i32) {
        self.lookups += 1;
        let applied_limit = correction_limit_cp();
        self.positive_applied_limit_lookups += u64::from(correction == applied_limit);
        self.negative_applied_limit_lookups += u64::from(correction == -applied_limit);

        let magnitude = match correction.cmp(&0) {
            std::cmp::Ordering::Greater => {
                self.positive_corrections += 1;
                let magnitude = correction as usize;
                self.positive_magnitude_sum += magnitude as u64;
                magnitude
            }
            std::cmp::Ordering::Less => {
                self.negative_corrections += 1;
                let magnitude = correction.unsigned_abs() as usize;
                self.negative_magnitude_sum += magnitude as u64;
                magnitude
            }
            std::cmp::Ordering::Equal => return,
        };

        self.magnitude_sum += magnitude as u64;

        // Bucket 0 is 1-10 cp, bucket 1 is 11-20 cp, and the final
        // bucket absorbs values above the last closed range.
        let bucket =
            ((magnitude - 1) / CORRECTION_BUCKET_WIDTH_CP).min(CORRECTION_MAGNITUDE_BUCKETS - 1);
        self.magnitude_histogram.bins[bucket] += 1;

        if correction > 0 {
            self.positive_magnitude_histogram.bins[bucket] += 1;
        } else {
            self.negative_magnitude_histogram.bins[bucket] += 1;
        }
    }

    #[inline]
    pub fn record_update_opportunity(&mut self) {
        self.update_opportunities += 1;
    }

    #[inline]
    pub fn record_update(&mut self, kind: CorrectionUpdateKind, delta: i32) {
        self.update_calls += 1;

        match kind {
            CorrectionUpdateKind::Exact => self.exact_updates += 1,
            CorrectionUpdateKind::FailHigh => self.fail_high_updates += 1,
            CorrectionUpdateKind::FailLow => self.fail_low_updates += 1,
        }

        match delta.cmp(&0) {
            std::cmp::Ordering::Greater => {
                self.positive_update_deltas += 1;
                self.positive_update_delta_sum += delta as u64;
            }
            std::cmp::Ordering::Less => {
                self.negative_update_deltas += 1;
                self.negative_update_delta_magnitude_sum += delta.unsigned_abs() as u64;
            }
            std::cmp::Ordering::Equal => self.zero_update_deltas += 1,
        }
    }

    #[inline]
    pub fn nonzero_corrections(&self) -> u64 {
        self.positive_corrections + self.negative_corrections
    }

    #[inline]
    pub fn zero_corrections(&self) -> u64 {
        self.lookups.saturating_sub(self.nonzero_corrections())
    }

    pub(super) fn print(&self) {
        if self.lookups == 0 && self.update_opportunities == 0 {
            return;
        }

        let nonzero = self.nonzero_corrections();
        let zero = self.zero_corrections();

        section("Correction History");
        metric("Sign convention", "positive favors side to move");
        metric_count("Lookups", self.lookups);
        submetric_count("Main search", self.main_lookups);
        submetric_count("Quiescence", self.quiescence_lookups);
        metric_count("Non-zero corrections", nonzero);
        metric_pct("Non-zero rate", nonzero, self.lookups);
        metric_count("Positive corrections", self.positive_corrections);
        metric_pct(
            "Positive rate / lookups",
            self.positive_corrections,
            self.lookups,
        );
        metric_pct(
            "Positive share / non-zero",
            self.positive_corrections,
            nonzero,
        );
        metric_count("Negative corrections", self.negative_corrections);
        metric_pct(
            "Negative rate / lookups",
            self.negative_corrections,
            self.lookups,
        );
        metric_pct(
            "Negative share / non-zero",
            self.negative_corrections,
            nonzero,
        );
        metric_count("Zero corrections", zero);
        metric_pct("Zero rate / lookups", zero, self.lookups);

        if nonzero > 0 {
            metric(
                "Average non-zero magnitude",
                format!("{:.2} cp", self.magnitude_sum as f64 / nonzero as f64),
            );
        }
        if self.positive_corrections > 0 {
            metric(
                "Average positive correction",
                format!(
                    "{:.2} cp",
                    self.positive_magnitude_sum as f64 / self.positive_corrections as f64
                ),
            );
        }
        if self.negative_corrections > 0 {
            metric(
                "Average negative magnitude",
                format!(
                    "{:.2} cp",
                    self.negative_magnitude_sum as f64 / self.negative_corrections as f64
                ),
            );
        }

        subsection("Limit usage");
        let applied_limit = correction_limit_cp();
        metric(
            "Applied correction range",
            format!("-{applied_limit}..+{applied_limit} cp"),
        );
        submetric_count(
            &format!("At +{applied_limit} cp ceiling"),
            self.positive_applied_limit_lookups,
        );
        submetric_pct(
            "Rate / positive lookups",
            self.positive_applied_limit_lookups,
            self.positive_corrections,
        );
        submetric_count(
            &format!("At -{applied_limit} cp ceiling"),
            self.negative_applied_limit_lookups,
        );
        submetric_pct(
            "Rate / negative lookups",
            self.negative_applied_limit_lookups,
            self.negative_corrections,
        );
        if nonzero > 0 {
            subsection("Correction magnitude by sign (side-to-move perspective)");
            println!(
                "    {:>12} {:>12} {:>10} {:>12} {:>10}",
                "Centipawns", "Positive", "Pos share", "Negative", "Neg share"
            );

            for bucket in 0..CORRECTION_MAGNITUDE_BUCKETS {
                let positive = self.positive_magnitude_histogram.bins[bucket];
                let negative = self.negative_magnitude_histogram.bins[bucket];
                if positive == 0 && negative == 0 {
                    continue;
                }

                let lower = bucket * CORRECTION_BUCKET_WIDTH_CP + 1;
                let label = if bucket == CORRECTION_MAGNITUDE_BUCKETS - 1 {
                    format!("{lower}+")
                } else {
                    let upper = (bucket + 1) * CORRECTION_BUCKET_WIDTH_CP;
                    format!("{lower}-{upper}")
                };

                println!(
                    "    {:>12} {:>12} {:>10} {:>12} {:>10}",
                    label,
                    count(positive),
                    pct(positive, self.positive_corrections),
                    count(negative),
                    pct(negative, self.negative_corrections),
                );
            }
        }

        if self.update_opportunities > 0 {
            subsection("Updates");
            submetric_count("Eligible nodes", self.update_opportunities);
            submetric_count("Update calls", self.update_calls);
            submetric_pct(
                "Update rate / eligible",
                self.update_calls,
                self.update_opportunities,
            );
            submetric_count("Exact", self.exact_updates);
            submetric_pct("Exact share", self.exact_updates, self.update_calls);
            submetric_count("Fail-high", self.fail_high_updates);
            submetric_pct("Fail-high share", self.fail_high_updates, self.update_calls);
            submetric_count("Fail-low", self.fail_low_updates);
            submetric_pct("Fail-low share", self.fail_low_updates, self.update_calls);
            submetric_count("Positive target deltas", self.positive_update_deltas);
            submetric_pct(
                "Positive-delta share",
                self.positive_update_deltas,
                self.update_calls,
            );
            submetric_count("Negative target deltas", self.negative_update_deltas);
            submetric_pct(
                "Negative-delta share",
                self.negative_update_deltas,
                self.update_calls,
            );
            submetric_count("Zero target deltas", self.zero_update_deltas);

            if self.positive_update_deltas > 0 {
                metric(
                    "Average positive target delta",
                    format!(
                        "{:.2} cp",
                        self.positive_update_delta_sum as f64 / self.positive_update_deltas as f64
                    ),
                );
            }
            if self.negative_update_deltas > 0 {
                metric(
                    "Average negative delta magnitude",
                    format!(
                        "{:.2} cp",
                        self.negative_update_delta_magnitude_sum as f64
                            / self.negative_update_deltas as f64
                    ),
                );
            }
        }
    }
}

impl_counter_stats_ops!(
    CorrectionHistoryStats,
    lookups,
    main_lookups,
    quiescence_lookups,
    positive_corrections,
    negative_corrections,
    magnitude_sum,
    positive_magnitude_sum,
    negative_magnitude_sum,
    magnitude_histogram,
    positive_magnitude_histogram,
    negative_magnitude_histogram,
    positive_applied_limit_lookups,
    negative_applied_limit_lookups,
    update_opportunities,
    update_calls,
    exact_updates,
    fail_high_updates,
    fail_low_updates,
    positive_update_deltas,
    negative_update_deltas,
    zero_update_deltas,
    positive_update_delta_sum,
    negative_update_delta_magnitude_sum,
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn records_sign_and_separate_magnitude_buckets() {
        let mut stats = CorrectionHistoryStats::default();

        for correction in [0, 1, 10, 11, -20, -21, 100, -101, 250] {
            stats.record_main_lookup(correction);
        }

        assert_eq!(stats.lookups, 9);
        assert_eq!(stats.main_lookups, 9);
        assert_eq!(stats.quiescence_lookups, 0);
        assert_eq!(stats.positive_corrections, 5);
        assert_eq!(stats.negative_corrections, 3);
        assert_eq!(stats.zero_corrections(), 1);
        assert_eq!(stats.nonzero_corrections(), 8);
        assert_eq!(stats.magnitude_sum, 514);
        assert_eq!(stats.positive_magnitude_sum, 372);
        assert_eq!(stats.negative_magnitude_sum, 142);
        assert_eq!(stats.magnitude_histogram.total(), 8);
        assert_eq!(stats.positive_magnitude_histogram.total(), 5);
        assert_eq!(stats.negative_magnitude_histogram.total(), 3);
        assert_eq!(stats.positive_magnitude_histogram.bins[0], 2);
        assert_eq!(stats.positive_magnitude_histogram.bins[1], 1);
        assert_eq!(stats.negative_magnitude_histogram.bins[1], 1);
        assert_eq!(stats.negative_magnitude_histogram.bins[2], 1);
        assert_eq!(stats.positive_magnitude_histogram.bins[9], 1);
        assert_eq!(stats.positive_magnitude_histogram.bins[10], 1);
        assert_eq!(stats.negative_magnitude_histogram.bins[10], 1);
    }

    #[test]
    fn records_lookup_source_and_limit_usage() {
        let mut stats = CorrectionHistoryStats::default();
        stats.record_quiescence_lookup(-correction_limit_cp());

        assert_eq!(stats.lookups, 1);
        assert_eq!(stats.main_lookups, 0);
        assert_eq!(stats.quiescence_lookups, 1);
        assert_eq!(stats.negative_applied_limit_lookups, 1);
    }

    #[test]
    fn records_update_kind_and_direction() {
        let mut stats = CorrectionHistoryStats::default();
        for _ in 0..4 {
            stats.record_update_opportunity();
        }

        stats.record_update(CorrectionUpdateKind::Exact, 12);
        stats.record_update(CorrectionUpdateKind::FailHigh, 30);
        stats.record_update(CorrectionUpdateKind::FailLow, -20);

        assert_eq!(stats.update_opportunities, 4);
        assert_eq!(stats.update_calls, 3);
        assert_eq!(stats.exact_updates, 1);
        assert_eq!(stats.fail_high_updates, 1);
        assert_eq!(stats.fail_low_updates, 1);
        assert_eq!(stats.positive_update_deltas, 2);
        assert_eq!(stats.negative_update_deltas, 1);
        assert_eq!(stats.positive_update_delta_sum, 42);
        assert_eq!(stats.negative_update_delta_magnitude_sum, 20);
    }

    #[test]
    fn aggregates_and_subtracts_all_correction_statistics() {
        let mut first = CorrectionHistoryStats::default();
        first.record_main_lookup(7);
        first.record_main_lookup(-18);
        first.record_update_opportunity();
        first.record_update(CorrectionUpdateKind::Exact, -9);

        let mut second = CorrectionHistoryStats::default();
        second.record_quiescence_lookup(25);
        second.record_update_opportunity();
        second.record_update(CorrectionUpdateKind::FailHigh, 11);

        let original = first;
        first += second;

        assert_eq!(first.lookups, 3);
        assert_eq!(first.main_lookups, 2);
        assert_eq!(first.quiescence_lookups, 1);
        assert_eq!(first.magnitude_histogram.total(), 3);
        assert_eq!(first.update_calls, 2);

        let delta = first - second;
        assert_eq!(delta.lookups, original.lookups);
        assert_eq!(delta.main_lookups, original.main_lookups);
        assert_eq!(delta.magnitude_histogram, original.magnitude_histogram);
        assert_eq!(delta.update_calls, original.update_calls);
    }
}
