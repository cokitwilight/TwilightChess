use std::ops::{AddAssign, Sub};

use crate::engine::tt::TableStats;

use super::formatting::{section, submetric_count, submetric_pct, subsection};

#[derive(Clone, Copy, Debug, Default)]
pub struct TranspositionTableStats {
    pub main: TableStats,
    pub quiescence: TableStats,
}

impl TranspositionTableStats {
    pub(super) fn print(&self) {
        if self.main.probes == 0 && self.quiescence.probes == 0 {
            return;
        }

        section("Transposition Tables");
        print_table("Main search", &self.main);
        print_table("Quiescence", &self.quiescence);
    }
}

fn print_table(name: &str, stats: &TableStats) {
    if stats.probes == 0 && stats.stores == 0 {
        return;
    }

    subsection(name);
    submetric_count("Probes", stats.probes);
    submetric_count("Hits", stats.hits);
    submetric_pct("Hit rate", stats.hits, stats.probes);
    submetric_count("Usable hits", stats.usable);
    submetric_pct("Usable-hit rate", stats.usable, stats.hits);
    submetric_count("Depth-rejected hits", stats.depth_rejected_hits);
    submetric_count("Hits with a move", stats.move_hits);
    submetric_count("Exact hits", stats.exact_hits);
    submetric_count("Lower-bound hits", stats.lower_bound_hits);
    submetric_count("Upper-bound hits", stats.upper_bound_hits);
    submetric_count("Exact returns", stats.exact_returns);
    submetric_count("Bound cutoffs", stats.bound_cutoffs);
    submetric_pct(
        "Return rate / usable hits",
        stats.exact_returns + stats.bound_cutoffs,
        stats.usable,
    );
    submetric_count("Stores", stats.stores);
    submetric_count("Replacements", stats.replacements);
    submetric_count("Collisions", stats.collisions);
}

impl Sub for TranspositionTableStats {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        Self {
            main: self.main - rhs.main,
            quiescence: self.quiescence - rhs.quiescence,
        }
    }
}

impl AddAssign for TranspositionTableStats {
    fn add_assign(&mut self, rhs: Self) {
        self.main += rhs.main;
        self.quiescence += rhs.quiescence;
    }
}
