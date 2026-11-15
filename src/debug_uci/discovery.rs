//! First passing depth and optional consecutive-depth confirmation.

use serde::Serialize;

use super::metrics::SearchMetrics;
use crate::engine::MAX_PLY;

#[derive(Clone, Copy, Debug, Serialize)]
#[cfg_attr(feature = "debug-uci", derive(serde::Deserialize))]
#[cfg_attr(feature = "debug-uci", serde(deny_unknown_fields))]
pub struct FindSolutionOptions {
    pub start_depth: u16,
    pub max_depth: u16,
    /// Additional consecutive passing depths after the initial hit, capped at max_depth.
    pub confirm_depths: u16,
}

impl FindSolutionOptions {
    pub fn validate(&self) -> Result<(), String> {
        if self.start_depth == 0
            || self.max_depth as usize >= MAX_PLY
            || self.start_depth > self.max_depth
        {
            return Err(format!(
                "find-solution requires 1 <= start-depth <= max-depth <= {}",
                MAX_PLY - 1
            ));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct SolutionHit {
    pub depth: u16,
    pub nodes: u64,
    pub elapsed_ns: u64,
    pub elapsed_ms: f64,
    pub best_move: Option<String>,
}

impl From<&SearchMetrics> for SolutionHit {
    fn from(m: &SearchMetrics) -> Self {
        Self {
            depth: m.completed_depth,
            nodes: m.nodes,
            elapsed_ns: m.elapsed_ns,
            elapsed_ms: m.elapsed_ms,
            best_move: m.best_move.clone(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DiscoveryStatus {
    Solved,
    DepthCap,
    BudgetLimit,
    Unscored,
}

#[derive(Debug, Serialize)]
pub struct DiscoveryResult {
    pub status: DiscoveryStatus,
    pub options: FindSolutionOptions,
    /// First observed hit at/above start_depth, even if later iterations miss.
    pub first_pass: Option<SolutionHit>,
    /// Start of the final successful streak. Present only when solved.
    pub solution: Option<SolutionHit>,
    pub confirmation_depth: Option<u16>,
    pub confirmations_checked: u16,
    /// All available depths passed, but max_depth shortened the requested confirmation.
    pub confirmation_capped: bool,
}

pub(crate) struct DiscoveryTracker {
    options: FindSolutionOptions,
    first_pass: Option<SolutionHit>,
    streak: Option<SolutionHit>,
    last_depth: Option<u16>,
    solved: bool,
}

impl DiscoveryTracker {
    pub fn new(options: FindSolutionOptions) -> Self {
        Self {
            options,
            first_pass: None,
            streak: None,
            last_depth: None,
            solved: false,
        }
    }

    pub fn observe(&mut self, metrics: &SearchMetrics, passing: bool) -> bool {
        let depth = metrics.completed_depth;
        if depth < self.options.start_depth {
            return false;
        }
        if !passing {
            self.streak = None;
        } else {
            let hit = SolutionHit::from(metrics);
            if self.first_pass.is_none() {
                self.first_pass = Some(hit.clone());
            }
            if self.last_depth.is_none_or(|previous| previous + 1 != depth) {
                self.streak = None;
            }
            let streak = self.streak.get_or_insert(hit);
            let target = streak
                .depth
                .saturating_add(self.options.confirm_depths)
                .min(self.options.max_depth);
            self.solved = depth >= target;
        }
        self.last_depth = Some(depth);
        self.solved
    }

    pub fn finish(self, stopped_depth: u16, cap_reached: bool, scorable: bool) -> DiscoveryResult {
        let status = if !scorable {
            DiscoveryStatus::Unscored
        } else if self.solved {
            DiscoveryStatus::Solved
        } else if cap_reached {
            DiscoveryStatus::DepthCap
        } else {
            DiscoveryStatus::BudgetLimit
        };
        let solution = self.streak.filter(|_| self.solved);
        let confirmations_checked = solution.as_ref().map_or(0, |hit| stopped_depth - hit.depth);
        DiscoveryResult {
            status,
            options: self.options,
            first_pass: self.first_pass,
            confirmation_depth: solution.as_ref().map(|_| stopped_depth),
            confirmation_capped: solution.is_some()
                && confirmations_checked < self.options.confirm_depths,
            confirmations_checked,
            solution,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample(depth: u16) -> SearchMetrics {
        let mut m = SearchMetrics::unsearched();
        m.completed_depth = depth;
        m.nodes = u64::from(depth) * 100;
        m.elapsed_ns = u64::from(depth) * 1000;
        m
    }

    #[test]
    fn ignores_passes_below_start_and_stops_at_first_eligible_hit() {
        let mut tracker = DiscoveryTracker::new(FindSolutionOptions {
            start_depth: 3,
            max_depth: 8,
            confirm_depths: 0,
        });
        assert!(!tracker.observe(&sample(1), true));
        assert!(!tracker.observe(&sample(2), true));
        assert!(tracker.observe(&sample(3), true));
        let result = tracker.finish(3, false, true);
        assert_eq!(result.status, DiscoveryStatus::Solved);
        assert_eq!(result.first_pass.unwrap().depth, 3);
        assert_eq!(result.solution.unwrap().nodes, 300);
        assert_eq!(result.confirmation_depth, Some(3));
        assert!(!result.confirmation_capped);
    }

    #[test]
    fn a_miss_resets_the_streak_but_retains_the_first_observed_hit() {
        let mut tracker = DiscoveryTracker::new(FindSolutionOptions {
            start_depth: 2,
            max_depth: 9,
            confirm_depths: 2,
        });
        for (depth, pass) in [(2, true), (3, false), (4, true), (5, true)] {
            assert!(!tracker.observe(&sample(depth), pass));
        }
        assert!(tracker.observe(&sample(6), true));
        let result = tracker.finish(6, false, true);
        assert_eq!(result.first_pass.unwrap().depth, 2);
        assert_eq!(result.solution.unwrap().depth, 4);
        assert_eq!(result.confirmation_depth, Some(6));
        assert_eq!(result.confirmations_checked, 2);
    }

    #[test]
    fn confirmation_checks_all_available_depths_and_stops_at_the_cap() {
        let options = FindSolutionOptions {
            start_depth: 4,
            max_depth: 5,
            confirm_depths: 1000,
        };
        let mut tracker = DiscoveryTracker::new(options);
        assert!(!tracker.observe(&sample(4), true));
        assert!(tracker.observe(&sample(5), true));
        let result = tracker.finish(5, true, true);
        assert_eq!(result.status, DiscoveryStatus::Solved);
        assert_eq!(result.confirmations_checked, 1);
        assert!(result.confirmation_capped);
        let mut tracker = DiscoveryTracker::new(options);
        assert!(!tracker.observe(&sample(4), false));
        assert!(tracker.observe(&sample(5), true));
        assert_eq!(tracker.finish(5, true, true).confirmations_checked, 0);
    }

    #[test]
    fn budget_interruption_is_not_confirmation_and_a_miss_at_the_cap_is_unsolved() {
        let options = FindSolutionOptions {
            start_depth: 2,
            max_depth: 4,
            confirm_depths: 2,
        };
        let mut tracker = DiscoveryTracker::new(options);
        assert!(!tracker.observe(&sample(2), true));
        let interrupted = tracker.finish(2, false, true);
        assert_eq!(interrupted.status, DiscoveryStatus::BudgetLimit);
        assert!(interrupted.first_pass.is_some());
        assert!(interrupted.solution.is_none());
        let mut tracker = DiscoveryTracker::new(options);
        assert!(!tracker.observe(&sample(2), true));
        assert!(!tracker.observe(&sample(3), true));
        assert!(!tracker.observe(&sample(4), false));
        let exhausted = tracker.finish(4, true, true);
        assert_eq!(exhausted.status, DiscoveryStatus::DepthCap);
        assert!(exhausted.solution.is_none());
        assert_eq!(
            DiscoveryTracker::new(options)
                .finish(0, false, false)
                .status,
            DiscoveryStatus::Unscored
        );
    }
}
