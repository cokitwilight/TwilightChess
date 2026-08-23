use std::ops::{AddAssign, Sub};

use super::{MOVE_INDEX_BUCKETS, MoveIndexHistogram, count, pct};

#[derive(Clone, Copy, Debug, Default)]
pub struct MoveOrderingStats {
    pub tt_move_cutoffs: u64,
    pub winning_capture_cutoffs: u64,
    pub killer_move_cutoffs: u64,
    pub history_move_cutoffs: u64,
    pub losing_capture_cutoffs: u64,
    pub cutoff_move_index_sum: u64,
    pub cutoff_move_index_max: u64,
    pub cutoff_move_index_histogram: MoveIndexHistogram,
}

impl MoveOrderingStats {
    pub(super) fn print(&self) {
        let total_cutoffs = self.cutoff_move_index_histogram.total();
        if total_cutoffs == 0 {
            return;
        }

        println!("Move Ordering");
        println!(
            "  {:<22} {:>14}",
            "TT-move cutoffs:",
            count(self.tt_move_cutoffs)
        );
        println!(
            "  {:<22} {:>14}",
            "Winning captures:",
            count(self.winning_capture_cutoffs)
        );
        println!(
            "  {:<22} {:>14}",
            "Killer moves:",
            count(self.killer_move_cutoffs)
        );
        println!(
            "  {:<22} {:>14}",
            "History moves:",
            count(self.history_move_cutoffs)
        );
        println!(
            "  {:<22} {:>14}",
            "Losing captures:",
            count(self.losing_capture_cutoffs)
        );
        println!(
            "  {:<22} {:>14.2}",
            "Average cutoff move:",
            self.cutoff_move_index_sum as f64 / total_cutoffs as f64
        );

        let maximum = self
            .cutoff_move_index_histogram
            .highest_nonzero_bucket()
            .map_or(0, |bucket| bucket + 1);
        println!("  {:<22} {:>14}", "Maximum cutoff move:", maximum);

        println!("  Cutoff Position");
        println!(
            "    {:>7} {:>14} {:>10} {:>12}",
            "Move", "Cutoffs", "Share", "Cumulative"
        );

        let mut cumulative = 0;
        for index in 0..MOVE_INDEX_BUCKETS {
            let cutoffs = self.cutoff_move_index_histogram.bins[index];
            if cutoffs == 0 {
                continue;
            }

            cumulative += cutoffs;
            let move_label = if index == MOVE_INDEX_BUCKETS - 1 {
                format!("{}+", index + 1)
            } else {
                (index + 1).to_string()
            };

            println!(
                "    {:>7} {:>14} {:>10} {:>12}",
                move_label,
                count(cutoffs),
                pct(cutoffs, total_cutoffs),
                pct(cumulative, total_cutoffs),
            );
        }
    }
}

impl Sub for MoveOrderingStats {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        let cutoff_move_index_histogram =
            self.cutoff_move_index_histogram - rhs.cutoff_move_index_histogram;
        let cutoff_move_index_max = cutoff_move_index_histogram
            .highest_nonzero_bucket()
            .map_or(0, |bucket| (bucket + 1) as u64);

        Self {
            tt_move_cutoffs: self.tt_move_cutoffs.saturating_sub(rhs.tt_move_cutoffs),
            winning_capture_cutoffs: self
                .winning_capture_cutoffs
                .saturating_sub(rhs.winning_capture_cutoffs),
            killer_move_cutoffs: self
                .killer_move_cutoffs
                .saturating_sub(rhs.killer_move_cutoffs),
            history_move_cutoffs: self
                .history_move_cutoffs
                .saturating_sub(rhs.history_move_cutoffs),
            losing_capture_cutoffs: self
                .losing_capture_cutoffs
                .saturating_sub(rhs.losing_capture_cutoffs),
            cutoff_move_index_sum: self
                .cutoff_move_index_sum
                .saturating_sub(rhs.cutoff_move_index_sum),
            cutoff_move_index_max,
            cutoff_move_index_histogram,
        }
    }
}

impl AddAssign for MoveOrderingStats {
    fn add_assign(&mut self, rhs: Self) {
        self.tt_move_cutoffs += rhs.tt_move_cutoffs;
        self.winning_capture_cutoffs += rhs.winning_capture_cutoffs;
        self.killer_move_cutoffs += rhs.killer_move_cutoffs;
        self.history_move_cutoffs += rhs.history_move_cutoffs;
        self.losing_capture_cutoffs += rhs.losing_capture_cutoffs;
        self.cutoff_move_index_sum += rhs.cutoff_move_index_sum;
        self.cutoff_move_index_max = self.cutoff_move_index_max.max(rhs.cutoff_move_index_max);
        self.cutoff_move_index_histogram += rhs.cutoff_move_index_histogram;
    }
}
