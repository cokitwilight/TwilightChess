use num_format::{Locale, ToFormattedString};

use super::{DepthHistogram, MAX_TRACKED_DEPTH};

#[inline]
pub(super) fn count(value: u64) -> String {
    value.to_formatted_string(&Locale::en)
}

#[inline]
pub(super) fn pct(part: u64, total: u64) -> String {
    if total == 0 {
        "0.00%".to_string()
    } else {
        format!("{:.2}%", part as f64 * 100.0 / total as f64)
    }
}

#[inline]
pub(super) fn nps(nodes: u64, seconds: f64) -> String {
    if seconds <= 0.0 {
        "0".to_string()
    } else {
        count((nodes as f64 / seconds) as u64)
    }
}

pub(super) fn print_depth_histogram(
    attempts: &DepthHistogram,
    outcomes: &DepthHistogram,
    outcome_label: &str,
) {
    if attempts.total() == 0 && outcomes.total() == 0 {
        return;
    }

    println!(
        "    {:>7} {:>14} {:>14} {:>10}",
        "Depth", "Attempts", outcome_label, "Rate"
    );

    for depth in 0..MAX_TRACKED_DEPTH {
        let attempt_count = attempts.bins[depth];
        let outcome_count = outcomes.bins[depth];

        if attempt_count == 0 && outcome_count == 0 {
            continue;
        }

        let depth_label = if depth == MAX_TRACKED_DEPTH - 1 {
            format!("{depth}+")
        } else {
            depth.to_string()
        };

        println!(
            "    {:>7} {:>14} {:>14} {:>10}",
            depth_label,
            count(attempt_count),
            count(outcome_count),
            pct(outcome_count, attempt_count),
        );
    }
}
