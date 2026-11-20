use crate::engine::configs::SingularConfig;

#[inline]
pub fn singular_margin(depth: u16, config: &SingularConfig) -> i32 {
    config.base_margin + config.depth_margin * depth as i32
}

#[inline]
pub fn singular_verification_depth(depth: u16) -> u16 {
    depth.saturating_sub(1) / 2
}
