use crate::engine::search_stats::formatting::{count, section};
use crate::engine::search_stats::{DepthHistogram, MAX_TRACKED_DEPTH};

/// Per-iteration node totals from completed iterative-deepening depths.
///
/// Depth one is stored in bucket zero. The final bucket combines depth 64 and
/// above. `samples_by_depth` keeps aggregation across searches meaningful.
#[derive(Clone, Copy, Debug, Default)]
#[cfg_attr(feature = "debug-uci", derive(serde::Serialize))]
pub struct IterativeDeepeningStats {
    pub nodes_by_depth: DepthHistogram,
    pub samples_by_depth: DepthHistogram,
    /// Sum of completed iterations' selective depths, for averaging across searches.
    pub seldepth_by_depth: DepthHistogram,
}

impl IterativeDeepeningStats {
    pub(crate) fn record_iteration(&mut self, depth: u16, nodes: u64, seldepth: usize) {
        if depth == 0 {
            return;
        }

        let bucket = depth_bucket(depth);
        self.nodes_by_depth.bins[bucket] += nodes;
        self.samples_by_depth.bins[bucket] += 1;
        self.seldepth_by_depth.bins[bucket] += seldepth as u64;
    }

    /// Total exclusive nodes recorded for this depth across all aggregated
    /// searches.
    pub fn nodes_at_depth(&self, depth: u16) -> u64 {
        depth
            .checked_sub(1)
            .and_then(|depth| self.nodes_by_depth.bins.get(depth as usize))
            .copied()
            .unwrap_or(0)
    }

    pub fn samples_at_depth(&self, depth: u16) -> u64 {
        depth
            .checked_sub(1)
            .and_then(|depth| self.samples_by_depth.bins.get(depth as usize))
            .copied()
            .unwrap_or(0)
    }

    /// Average exclusive node count for a completed iteration at `depth`.
    pub fn average_nodes_at_depth(&self, depth: u16) -> Option<f64> {
        let samples = self.samples_at_depth(depth);
        if samples == 0 {
            None
        } else {
            Some(self.nodes_at_depth(depth) as f64 / samples as f64)
        }
    }

    pub fn average_seldepth_at_depth(&self, depth: u16) -> Option<f64> {
        let samples = self.samples_at_depth(depth);
        if samples == 0 {
            None
        } else {
            Some(self.seldepth_by_depth.bins[depth as usize - 1] as f64 / samples as f64)
        }
    }

    /// Rough node-growth multiple from the preceding completed depth.
    pub fn estimated_growth_factor(&self, depth: u16) -> Option<f64> {
        if depth <= 1 || depth as usize > MAX_TRACKED_DEPTH {
            return None;
        }

        let previous = self.average_nodes_at_depth(depth - 1)?;
        let current = self.average_nodes_at_depth(depth)?;

        if previous > 0.0 {
            Some(current / previous)
        } else {
            None
        }
    }

    pub(super) fn print(&self) {
        if self.samples_by_depth.total() == 0 {
            return;
        }

        section("Iterative Deepening");
        println!("  Counts exclude work from earlier iterative depths.");
        println!(
            "    {:>7} {:>8} {:>18} {:>10} {:>12}",
            "Depth", "SelDepth", "Nodes / Iteration", "Samples", "Growth"
        );

        for bucket in 0..MAX_TRACKED_DEPTH {
            let samples = self.samples_by_depth.bins[bucket];
            if samples == 0 {
                continue;
            }

            let depth = bucket + 1;
            let depth_label = if bucket == MAX_TRACKED_DEPTH - 1 {
                format!("{depth}+")
            } else {
                depth.to_string()
            };
            let average_nodes = self.nodes_by_depth.bins[bucket] as f64 / samples as f64;
            let average_seldepth = self.seldepth_by_depth.bins[bucket] as f64 / samples as f64;
            let seldepth_label = if samples == 1 {
                self.seldepth_by_depth.bins[bucket].to_string()
            } else {
                format!("{average_seldepth:.2}")
            };
            let growth = self
                .estimated_growth_factor(depth as u16)
                .map_or_else(|| "—".to_string(), |factor| format!("{factor:.2}x"));

            println!(
                "    {:>7} {:>8} {:>18} {:>10} {:>12}",
                depth_label,
                seldepth_label,
                count(average_nodes.round() as u64),
                count(samples),
                growth,
            );
        }
    }
}

impl std::ops::Sub for IterativeDeepeningStats {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        Self {
            nodes_by_depth: self.nodes_by_depth - rhs.nodes_by_depth,
            samples_by_depth: self.samples_by_depth - rhs.samples_by_depth,
            seldepth_by_depth: self.seldepth_by_depth - rhs.seldepth_by_depth,
        }
    }
}

impl std::ops::AddAssign for IterativeDeepeningStats {
    fn add_assign(&mut self, rhs: Self) {
        self.nodes_by_depth += rhs.nodes_by_depth;
        self.samples_by_depth += rhs.samples_by_depth;
        self.seldepth_by_depth += rhs.seldepth_by_depth;
    }
}

fn depth_bucket(depth: u16) -> usize {
    usize::from(depth.saturating_sub(1)).min(MAX_TRACKED_DEPTH - 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn records_exclusive_nodes_and_estimates_growth() {
        let mut stats = IterativeDeepeningStats::default();
        stats.record_iteration(1, 100, 3);
        stats.record_iteration(2, 400, 5);

        assert_eq!(stats.nodes_at_depth(1), 100);
        assert_eq!(stats.nodes_at_depth(2), 400);
        assert_eq!(stats.samples_at_depth(1), 1);
        assert_eq!(stats.average_nodes_at_depth(2), Some(400.0));
        assert_eq!(stats.average_seldepth_at_depth(0), None);
        assert_eq!(stats.average_seldepth_at_depth(1), Some(3.0));
        assert_eq!(stats.average_seldepth_at_depth(2), Some(5.0));
        assert_eq!(stats.estimated_growth_factor(1), None);
        assert_eq!(stats.estimated_growth_factor(2), Some(4.0));
    }

    #[test]
    fn aggregation_uses_average_nodes_for_growth() {
        let mut first = IterativeDeepeningStats::default();
        first.record_iteration(1, 100, 2);
        first.record_iteration(2, 400, 4);

        let mut second = IterativeDeepeningStats::default();
        second.record_iteration(1, 200, 3);
        second.record_iteration(2, 600, 7);

        first += second;

        assert_eq!(first.nodes_at_depth(1), 300);
        assert_eq!(first.nodes_at_depth(2), 1_000);
        assert_eq!(first.samples_at_depth(1), 2);
        assert_eq!(first.average_nodes_at_depth(1), Some(150.0));
        assert_eq!(first.average_nodes_at_depth(2), Some(500.0));
        assert_eq!(first.average_seldepth_at_depth(1), Some(2.5));
        assert_eq!(first.average_seldepth_at_depth(2), Some(5.5));
        assert_eq!(first.estimated_growth_factor(2), Some(10.0 / 3.0));

        let delta = first - second;
        assert_eq!(delta.average_seldepth_at_depth(2), Some(4.0));
    }
}
