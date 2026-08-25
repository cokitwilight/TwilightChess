use std::ops::{AddAssign, Sub};

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
