use std::ops::Sub;

use num_format::{Locale, ToFormattedString};

#[derive(Clone, Copy, Debug, Default)]
pub struct TableStats {
    pub probes: u64,
    pub hits: u64,
    pub usable: u64,
    pub exact_returns: u64,
    pub bound_cutoffs: u64,
    pub stores: u64,
}

impl TableStats {
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
            exact_returns: self.exact_returns.saturating_sub(rhs.exact_returns),
            bound_cutoffs: self.bound_cutoffs.saturating_sub(rhs.bound_cutoffs),
            stores: self.stores.saturating_sub(rhs.stores),
        }
    }
}
