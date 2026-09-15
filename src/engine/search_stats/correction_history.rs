use crate::engine::search_stats::formatting::{
    count, metric, metric_count, metric_pct, pct, section, subsection,
};
use crate::engine::search_stats::{CORRECTION_MAGNITUDE_BUCKETS, CorrectionMagnitudeHistogram};

pub const CORRECTION_BUCKET_WIDTH_CP: usize = 10;

#[derive(Clone, Copy, Debug, Default)]
pub struct CorrectionHistoryStats {
    pub lookups: u64,
    pub positive_corrections: u64,
    pub negative_corrections: u64,
    pub magnitude_sum: u64,
    pub magnitude_histogram: CorrectionMagnitudeHistogram,
}

impl CorrectionHistoryStats {
    #[inline]
    pub fn record(&mut self, correction: i32) {
        self.lookups += 1;

        match correction.cmp(&0) {
            std::cmp::Ordering::Greater => self.positive_corrections += 1,
            std::cmp::Ordering::Less => self.negative_corrections += 1,
            std::cmp::Ordering::Equal => return,
        }

        let magnitude = correction.unsigned_abs() as usize;
        self.magnitude_sum += magnitude as u64;

        // Bucket 0 is 1-10 cp, bucket 1 is 11-20 cp, and the final
        // bucket absorbs values above the last closed range.
        let bucket =
            ((magnitude - 1) / CORRECTION_BUCKET_WIDTH_CP).min(CORRECTION_MAGNITUDE_BUCKETS - 1);
        self.magnitude_histogram.bins[bucket] += 1;
    }

    #[inline]
    pub fn nonzero_corrections(&self) -> u64 {
        self.positive_corrections + self.negative_corrections
    }

    pub(super) fn print(&self) {
        if self.lookups == 0 {
            return;
        }

        let nonzero = self.nonzero_corrections();

        section("Correction History");
        metric_count("Lookups", self.lookups);
        metric_count("Non-zero corrections", nonzero);
        metric_pct("Non-zero rate", nonzero, self.lookups);
        metric_count("Positive corrections", self.positive_corrections);
        metric_pct(
            "Positive-correction rate",
            self.positive_corrections,
            self.lookups,
        );
        metric_count("Negative corrections", self.negative_corrections);

        if nonzero == 0 {
            return;
        }

        metric(
            "Average non-zero magnitude",
            format!("{:.2} cp", self.magnitude_sum as f64 / nonzero as f64),
        );

        subsection("Correction-magnitude distribution");
        println!("    {:>12} {:>12} {:>10}", "Centipawns", "Uses", "Share");

        for bucket in 0..CORRECTION_MAGNITUDE_BUCKETS {
            let uses = self.magnitude_histogram.bins[bucket];
            if uses == 0 {
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
                "    {:>12} {:>12} {:>10}",
                label,
                count(uses),
                pct(uses, nonzero),
            );
        }
    }
}

impl_counter_stats_ops!(
    CorrectionHistoryStats,
    lookups,
    positive_corrections,
    negative_corrections,
    magnitude_sum,
    magnitude_histogram,
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn records_sign_and_ranged_magnitude_buckets() {
        let mut stats = CorrectionHistoryStats::default();

        for correction in [0, 1, 10, 11, -20, -21, 100, -101, 250] {
            stats.record(correction);
        }

        assert_eq!(stats.lookups, 9);
        assert_eq!(stats.positive_corrections, 5);
        assert_eq!(stats.negative_corrections, 3);
        assert_eq!(stats.nonzero_corrections(), 8);
        assert_eq!(stats.magnitude_sum, 514);
        assert_eq!(stats.magnitude_histogram.total(), 8);
        assert_eq!(stats.magnitude_histogram.bins[0], 2); // 1-10
        assert_eq!(stats.magnitude_histogram.bins[1], 2); // 11-20
        assert_eq!(stats.magnitude_histogram.bins[2], 1); // 21-30
        assert_eq!(stats.magnitude_histogram.bins[9], 1); // 91-100
        assert_eq!(stats.magnitude_histogram.bins[10], 2); // 101+
    }

    #[test]
    fn aggregates_and_subtracts_all_correction_statistics() {
        let mut first = CorrectionHistoryStats::default();
        first.record(7);
        first.record(-18);

        let mut second = CorrectionHistoryStats::default();
        second.record(25);

        let original = first;
        first += second;

        assert_eq!(first.lookups, 3);
        assert_eq!(first.magnitude_histogram.total(), 3);
        assert_eq!((first - second).lookups, original.lookups);
        assert_eq!(
            (first - second).magnitude_histogram,
            original.magnitude_histogram
        );
    }
}
