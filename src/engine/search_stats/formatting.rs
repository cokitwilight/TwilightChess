use num_format::{Locale, ToFormattedString};

use crate::engine::search_stats::{DepthHistogram, MAX_TRACKED_DEPTH};

pub(super) const REPORT_WIDTH: usize = 64;

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

pub(super) fn report_header(depth: u16) {
    println!();
    println!("Search Statistics — Depth {depth}");
    println!("{}", "─".repeat(REPORT_WIDTH));
}

pub(super) fn report_footer() {
    println!("{}", "─".repeat(REPORT_WIDTH));
}

pub(super) fn section(title: &str) {
    println!();
    println!("{title}");
}

pub(super) fn subsection(title: &str) {
    println!("  {title}");
}

pub(super) fn metric(label: &str, value: impl std::fmt::Display) {
    println!("  {label:<30} {value:>16}");
}

pub(super) fn submetric(label: &str, value: impl std::fmt::Display) {
    println!("    {label:<28} {value:>16}");
}

pub(super) fn metric_count(label: &str, value: u64) {
    metric(label, count(value));
}

pub(super) fn submetric_count(label: &str, value: u64) {
    submetric(label, count(value));
}

pub(super) fn metric_pct(label: &str, part: u64, total: u64) {
    metric(label, pct(part, total));
}

pub(super) fn submetric_pct(label: &str, part: u64, total: u64) {
    submetric(label, pct(part, total));
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
        "    {:>7} {:>12} {:>12} {:>10}",
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
            "    {:>7} {:>12} {:>12} {:>10}",
            depth_label,
            count(attempt_count),
            count(outcome_count),
            pct(outcome_count, attempt_count),
        );
    }
}
