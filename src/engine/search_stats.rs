use std::ops::Sub;

use num_format::{Locale, ToFormattedString};

use crate::engine::tt::TableStats;

// used across searches to store information about the search, such as the best move found, the evaluation score, and the principal variation.
#[derive(Clone, Copy, Debug, Default)]
pub struct SearchStats {
    pub nodes: u64,
    pub qnodes: u64,

    pub beta_cutoffs: u64,
    pub stand_pat_cutoffs: u64,

    pub moves_searched: u64,
    pub qmoves_searched: u64,

    pub illegal_moves: u64, // since search uses pseudo moves
    pub qillegal_moves: u64,

    pub aspiration_w_fail_high: u64,
    pub aspiration_w_fail_low: u64,

    pub tt: TableStats,
    pub qtt: TableStats,

    pub killer_cutoffs: u64,
    pub history_cutoffs: u64,

    pub lmr_attempts: u64,
    pub lmr_researched: u64,

    pub rfp_attempts: u64,
    pub rfp_cutoffs: u64,

    pub null_attempts: u64,
    pub null_cutoffs: u64,

    pub delta_prunes: u64,
    pub see_prunes: u64,

    pub repetition_returns: u64,
    pub fifty_returns: u64,
}

impl SearchStats {
    #[inline]
    pub fn total_nodes(&self) -> u64 {
        self.nodes + self.qnodes
    }

    #[inline]
    fn fmt(n: u64) -> String {
        n.to_formatted_string(&Locale::en)
    }

    #[inline]
    fn pct(part: u64, total: u64) -> String {
        if total == 0 {
            "0.00%".to_string()
        } else {
            format!("{:.2}%", part as f64 * 100.0 / total as f64)
        }
    }

    #[inline]
    fn nps(nodes: u64, seconds: f64) -> String {
        if seconds <= 0.0 {
            "0".to_string()
        } else {
            let nps = (nodes as f64 / seconds) as u64;
            nps.to_formatted_string(&Locale::en)
        }
    }

    pub fn print_all(&self, depth: usize, elapsed_secs: f64) {
        println!();
        println!("Depth {}", depth);
        println!("────────────────────────────────────────────");

        self.print_nodes(elapsed_secs);
        self.print_moves();
        self.print_cutoffs();
        self.print_aspiration();
        self.print_ordering();
        self.print_pruning();
        self.print_returns();
        self.print_tts();

        println!("────────────────────────────────────────────");
    }

    pub fn print_nodes(&self, elapsed_secs: f64) {
        let total = self.total_nodes();

        println!("Nodes");
        println!("  {:<22} {:>14}", "Main:", Self::fmt(self.nodes));
        println!("  {:<22} {:>14}", "Quiescence:", Self::fmt(self.qnodes));
        println!("  {:<22} {:>14}", "Total:", Self::fmt(total));
        println!("  {:<22} {:>14}", "NPS:", Self::nps(total, elapsed_secs));
    }

    pub fn print_moves(&self) {
        let total_moves = self.moves_searched + self.qmoves_searched;

        if total_moves == 0 && self.illegal_moves == 0 {
            return;
        }

        println!("Moves");
        println!(
            "  {:<22} {:>14}",
            "Main searched:",
            Self::fmt(self.moves_searched)
        );
        println!(
            "  {:<22} {:>14}",
            "Q searched:",
            Self::fmt(self.qmoves_searched)
        );
        println!("  {:<22} {:>14}", "Total searched:", Self::fmt(total_moves));
        println!(
            "  {:<22} {:>14}",
            "Illegal pseudo:",
            Self::fmt(self.illegal_moves)
        );

        println!(
            "  {:<22} {:>14}",
            "Illegal Q pseudo:",
            Self::fmt(self.qillegal_moves)
        );

        let illegal_total = self.illegal_moves + self.qillegal_moves;
        let pseudo_total = total_moves + illegal_total;

        if pseudo_total > 0 {
            println!(
                "  {:<22} {:>14}",
                "Illegal rate:",
                Self::pct(illegal_total, pseudo_total)
            );
        }

        if self.nodes > 0 {
            println!(
                "  {:<22} {:>14.2}",
                "Moves / node:",
                self.moves_searched as f64 / self.nodes as f64
            );
        }

        if self.qnodes > 0 {
            println!(
                "  {:<22} {:>14.2}",
                "Q moves / qnode:",
                self.qmoves_searched as f64 / self.qnodes as f64
            );
        }
    }

    pub fn print_cutoffs(&self) {
        let has_cutoffs = self.beta_cutoffs > 0 || self.stand_pat_cutoffs > 0;

        if !has_cutoffs {
            return;
        }

        println!("Cutoffs");
        println!(
            "  {:<22} {:>14}",
            "Beta cutoffs:",
            Self::fmt(self.beta_cutoffs)
        );
        println!(
            "  {:<22} {:>14}",
            "Stand-pat cutoffs:",
            Self::fmt(self.stand_pat_cutoffs)
        );

        if self.moves_searched > 0 {
            println!(
                "  {:<22} {:>14}",
                "Beta / main moves:",
                Self::pct(self.beta_cutoffs, self.moves_searched)
            );
        }

        if self.qnodes > 0 {
            println!(
                "  {:<22} {:>14}",
                "Stand-pat / qnodes:",
                Self::pct(self.stand_pat_cutoffs, self.qnodes)
            );
        }
    }

    pub fn print_aspiration(&self) {
        let total_fails = self.aspiration_w_fail_high + self.aspiration_w_fail_low;

        if total_fails == 0 {
            return;
        }

        println!("Aspiration Windows");
        println!(
            "  {:<22} {:>14}",
            "Fail high:",
            Self::fmt(self.aspiration_w_fail_high)
        );
        println!(
            "  {:<22} {:>14}",
            "Fail low:",
            Self::fmt(self.aspiration_w_fail_low)
        );
        println!("  {:<22} {:>14}", "Total fails:", Self::fmt(total_fails));

        println!(
            "  {:<22} {:>14}",
            "Fail-high share:",
            Self::pct(self.aspiration_w_fail_high, total_fails)
        );
    }

    pub fn print_ordering(&self) {
        let has_ordering = self.killer_cutoffs > 0 || self.history_cutoffs > 0;

        if !has_ordering {
            return;
        }

        println!("Move Ordering");
        println!(
            "  {:<22} {:>14}",
            "Killer cutoffs:",
            Self::fmt(self.killer_cutoffs)
        );
        println!(
            "  {:<22} {:>14}",
            "History cutoffs:",
            Self::fmt(self.history_cutoffs)
        );

        if self.beta_cutoffs > 0 {
            println!(
                "  {:<22} {:>14}",
                "Killer / beta:",
                Self::pct(self.killer_cutoffs, self.beta_cutoffs)
            );
            println!(
                "  {:<22} {:>14}",
                "History / beta:",
                Self::pct(self.history_cutoffs, self.beta_cutoffs)
            );
        }
    }

    pub fn print_pruning(&self) {
        let has_lmr = self.lmr_attempts > 0 || self.lmr_researched > 0;
        let has_null = self.null_attempts > 0 || self.null_cutoffs > 0;
        let has_q_prunes = self.delta_prunes > 0 || self.see_prunes > 0;
        let has_rfp_prunes = self.rfp_attempts > 0 || self.rfp_cutoffs > 0;

        if !has_lmr && !has_null && !has_q_prunes && !has_rfp_prunes {
            return;
        }

        println!("Pruning / Reductions");

        if has_lmr {
            println!(
                "  {:<22} {:>14}",
                "LMR attempts:",
                Self::fmt(self.lmr_attempts)
            );
            println!(
                "  {:<22} {:>14}",
                "LMR re-searches:",
                Self::fmt(self.lmr_researched)
            );

            if self.lmr_attempts > 0 {
                println!(
                    "  {:<22} {:>14}",
                    "LMR re-search rate:",
                    Self::pct(self.lmr_researched, self.lmr_attempts)
                );
            }
        }

        if has_null {
            println!(
                "  {:<22} {:>14}",
                "Null attempts:",
                Self::fmt(self.null_attempts)
            );
            println!(
                "  {:<22} {:>14}",
                "Null cutoffs:",
                Self::fmt(self.null_cutoffs)
            );

            if self.null_attempts > 0 {
                println!(
                    "  {:<22} {:>14}",
                    "Null cutoff rate:",
                    Self::pct(self.null_cutoffs, self.null_attempts)
                );
            }
        }

        if has_q_prunes {
            println!(
                "  {:<22} {:>14}",
                "Delta prunes:",
                Self::fmt(self.delta_prunes)
            );
            println!("  {:<22} {:>14}", "SEE prunes:", Self::fmt(self.see_prunes));

            let total_q_prunes = self.delta_prunes + self.see_prunes;
            println!(
                "  {:<22} {:>14}",
                "Total q prunes:",
                Self::fmt(total_q_prunes)
            );
        }
        if has_rfp_prunes {
            println!(
                "  {:<22} {:>14}",
                "RFP attempts:",
                Self::fmt(self.rfp_attempts)
            );
            println!(
                "  {:<22} {:>14}",
                "RFP cutoffs:",
                Self::fmt(self.rfp_cutoffs)
            );
        }
    }

    pub fn print_returns(&self) {
        let has_returns = self.repetition_returns > 0 || self.fifty_returns > 0;

        if !has_returns {
            return;
        }

        println!("Draw Returns");
        println!(
            "  {:<22} {:>14}",
            "Repetition:",
            Self::fmt(self.repetition_returns)
        );
        println!(
            "  {:<22} {:>14}",
            "Fifty move:",
            Self::fmt(self.fifty_returns)
        );
    }

    pub fn print_tts(&self) {
        if self.tt.probes == 0 && self.qtt.probes == 0 {
            return;
        }
        println!("Transposition Tables");

        println!("  Negamax TT");
        self.tt.print_stats_indented("    ");

        println!("  Quiescence TT");
        self.qtt.print_stats_indented("    ");
    }
}

impl Sub for SearchStats {
    type Output = SearchStats;

    fn sub(self, rhs: SearchStats) -> SearchStats {
        SearchStats {
            nodes: self.nodes.saturating_sub(rhs.nodes),
            qnodes: self.qnodes.saturating_sub(rhs.qnodes),

            beta_cutoffs: self.beta_cutoffs.saturating_sub(rhs.beta_cutoffs),
            stand_pat_cutoffs: self.stand_pat_cutoffs.saturating_sub(rhs.stand_pat_cutoffs),

            moves_searched: self.nodes.saturating_sub(rhs.moves_searched),
            qmoves_searched: self.qmoves_searched.saturating_sub(rhs.qmoves_searched),

            illegal_moves: self.illegal_moves.saturating_sub(rhs.illegal_moves),
            qillegal_moves: self.qillegal_moves.saturating_sub(rhs.qillegal_moves),

            aspiration_w_fail_high: self
                .aspiration_w_fail_high
                .saturating_sub(rhs.aspiration_w_fail_high),
            aspiration_w_fail_low: self
                .aspiration_w_fail_low
                .saturating_sub(rhs.aspiration_w_fail_low),

            tt: self.tt - rhs.tt,
            qtt: self.qtt - rhs.qtt,

            killer_cutoffs: self.killer_cutoffs.saturating_sub(rhs.killer_cutoffs),
            history_cutoffs: self.history_cutoffs.saturating_sub(rhs.history_cutoffs),

            lmr_attempts: self.lmr_attempts.saturating_sub(rhs.lmr_attempts),
            lmr_researched: self.lmr_researched.saturating_sub(rhs.lmr_researched),

            rfp_attempts: self.rfp_attempts.saturating_sub(rhs.rfp_attempts),
            rfp_cutoffs: self.rfp_cutoffs.saturating_sub(rhs.rfp_cutoffs),

            null_attempts: self.null_attempts.saturating_sub(rhs.null_attempts),
            null_cutoffs: self.null_cutoffs.saturating_sub(rhs.null_cutoffs),

            delta_prunes: self.delta_prunes.saturating_sub(rhs.delta_prunes),
            see_prunes: self.see_prunes.saturating_sub(rhs.see_prunes),

            repetition_returns: self
                .repetition_returns
                .saturating_sub(rhs.repetition_returns),
            fifty_returns: self.fifty_returns.saturating_sub(rhs.fifty_returns),
        }
    }
}

pub fn median_f64(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }

    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let mid = sorted.len() / 2;

    if sorted.len() % 2 == 0 {
        (sorted[mid - 1] + sorted[mid]) / 2.0
    } else {
        sorted[mid]
    }
}

pub fn fmt_nps(nps: f64) -> String {
    if !nps.is_finite() || nps <= 0.0 {
        "0".to_string()
    } else {
        let nps = nps as u64;
        nps.to_formatted_string(&Locale::en)
    }
}
