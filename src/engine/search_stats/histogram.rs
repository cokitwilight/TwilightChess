use std::ops::{AddAssign, Sub};

pub const MAX_TRACKED_DEPTH: usize = 64;
pub const REDUCTION_BUCKETS: usize = 8;
pub const MOVE_INDEX_BUCKETS: usize = 64;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Histogram<const N: usize> {
    pub bins: [u64; N],
}

impl<const N: usize> Histogram<N> {
    pub fn saturating_sub(self, rhs: Self) -> Self {
        self - rhs
    }

    pub fn total(&self) -> u64 {
        self.bins.iter().sum()
    }

    pub fn highest_nonzero_bucket(&self) -> Option<usize> {
        self.bins.iter().rposition(|&count| count > 0)
    }
}

impl<const N: usize> Default for Histogram<N> {
    fn default() -> Self {
        Self { bins: [0; N] }
    }
}

impl<const N: usize> Sub for Histogram<N> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        let mut bins = [0; N];

        for (index, value) in bins.iter_mut().enumerate() {
            *value = self.bins[index].saturating_sub(rhs.bins[index]);
        }

        Self { bins }
    }
}

impl<const N: usize> AddAssign for Histogram<N> {
    fn add_assign(&mut self, rhs: Self) {
        for (value, rhs_value) in self.bins.iter_mut().zip(rhs.bins) {
            *value += rhs_value;
        }
    }
}

pub type DepthHistogram = Histogram<MAX_TRACKED_DEPTH>;
pub type MoveIndexHistogram = Histogram<MOVE_INDEX_BUCKETS>;
pub type ReductionHistogram = Histogram<REDUCTION_BUCKETS>;
