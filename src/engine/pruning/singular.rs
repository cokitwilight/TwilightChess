use crate::engine::{configs::SingularConfig, tt::TTFlag};

#[inline]
pub fn singular_margin(depth: u16, config: &SingularConfig, tt_bound: TTFlag) -> i32 {
    let div = match tt_bound {
        TTFlag::Exact => 2,
        TTFlag::LowerBound => 1,
        TTFlag::UpperBound => panic!("Invalid call to singular margin for upper bound!"),
    };

    (config.base_margin + config.depth_margin * depth as i32) / div
}

#[inline]
pub fn singular_verification_depth(depth: u16) -> u16 {
    depth.saturating_sub(1) / 2
}
