use std::ops::{AddAssign, Sub};

use num_format::{Locale, ToFormattedString};

use crate::engine::tt::TTInsertResult;

#[derive(Clone, Copy, Debug, Default)]
pub struct TableStats {
    pub probes: u64,
    pub hits: u64,
    pub usable: u64,
    pub depth_rejected_hits: u64,
    pub exact_hits: u64,
    pub lower_bound_hits: u64,
    pub upper_bound_hits: u64,
    pub move_hits: u64,
    pub exact_returns: u64,
    pub bound_cutoffs: u64,
    pub stores: u64,
    pub replacements: u64,
    pub collisions: u64,
}

impl TableStats {
    pub fn record_insert(&mut self, result: TTInsertResult) {
        self.replacements += u64::from(result.replaced);
        self.collisions += u64::from(result.collision);
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

    pub fn print_stats_indented(&self, indent: &str) {
        println!("{}{:<22} {:>14}", indent, "Probes:", Self::fmt(self.probes));
        println!("{}{:<22} {:>14}", indent, "Hits:", Self::fmt(self.hits));
        println!("{}{:<22} {:>14}", indent, "Usable:", Self::fmt(self.usable));
        println!(
            "{}{:<22} {:>14}",
            indent,
            "Depth-rejected hits:",
            Self::fmt(self.depth_rejected_hits)
        );
        println!(
            "{}{:<22} {:>14}",
            indent,
            "Exact hits:",
            Self::fmt(self.exact_hits)
        );
        println!(
            "{}{:<22} {:>14}",
            indent,
            "Lower-bound hits:",
            Self::fmt(self.lower_bound_hits)
        );
        println!(
            "{}{:<22} {:>14}",
            indent,
            "Upper-bound hits:",
            Self::fmt(self.upper_bound_hits)
        );
        println!(
            "{}{:<22} {:>14}",
            indent,
            "Move hits:",
            Self::fmt(self.move_hits)
        );
        println!(
            "{}{:<22} {:>14}",
            indent,
            "Exact returns:",
            Self::fmt(self.exact_returns)
        );
        println!(
            "{}{:<22} {:>14}",
            indent,
            "Bound cutoffs:",
            Self::fmt(self.bound_cutoffs)
        );
        println!("{}{:<22} {:>14}", indent, "Stores:", Self::fmt(self.stores));
        println!(
            "{}{:<22} {:>14}",
            indent,
            "Replacements:",
            Self::fmt(self.replacements)
        );
        println!(
            "{}{:<22} {:>14}",
            indent,
            "Collisions:",
            Self::fmt(self.collisions)
        );

        if self.probes > 0 {
            println!(
                "{}{:<22} {:>14}",
                indent,
                "Hit rate:",
                Self::pct(self.hits, self.probes)
            );
        }

        if self.hits > 0 {
            println!(
                "{}{:<22} {:>14}",
                indent,
                "Usable / hits:",
                Self::pct(self.usable, self.hits)
            );
        }

        if self.usable > 0 {
            let returns = self.exact_returns + self.bound_cutoffs;

            println!(
                "{}{:<22} {:>14}",
                indent,
                "Return / usable:",
                Self::pct(returns, self.usable)
            );
        }
    }
}

impl Sub for TableStats {
    type Output = TableStats;

    fn sub(self, rhs: TableStats) -> TableStats {
        TableStats {
            probes: self.probes.saturating_sub(rhs.probes),
            hits: self.hits.saturating_sub(rhs.hits),
            usable: self.usable.saturating_sub(rhs.usable),
            depth_rejected_hits: self
                .depth_rejected_hits
                .saturating_sub(rhs.depth_rejected_hits),
            exact_hits: self.exact_hits.saturating_sub(rhs.exact_hits),
            lower_bound_hits: self.lower_bound_hits.saturating_sub(rhs.lower_bound_hits),
            upper_bound_hits: self.upper_bound_hits.saturating_sub(rhs.upper_bound_hits),
            move_hits: self.move_hits.saturating_sub(rhs.move_hits),
            exact_returns: self.exact_returns.saturating_sub(rhs.exact_returns),
            bound_cutoffs: self.bound_cutoffs.saturating_sub(rhs.bound_cutoffs),
            stores: self.stores.saturating_sub(rhs.stores),
            replacements: self.replacements.saturating_sub(rhs.replacements),
            collisions: self.collisions.saturating_sub(rhs.collisions),
        }
    }
}

impl AddAssign for TableStats {
    fn add_assign(&mut self, rhs: Self) {
        self.probes += rhs.probes;
        self.hits += rhs.hits;
        self.usable += rhs.usable;
        self.depth_rejected_hits += rhs.depth_rejected_hits;
        self.exact_hits += rhs.exact_hits;
        self.lower_bound_hits += rhs.lower_bound_hits;
        self.upper_bound_hits += rhs.upper_bound_hits;
        self.move_hits += rhs.move_hits;
        self.exact_returns += rhs.exact_returns;
        self.bound_cutoffs += rhs.bound_cutoffs;
        self.stores += rhs.stores;
        self.replacements += rhs.replacements;
        self.collisions += rhs.collisions;
    }
}
