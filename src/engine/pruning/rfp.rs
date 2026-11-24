use crate::engine::configs::RFPConfig;

#[inline]
pub fn rfp_margin(depth: i32, improvement: i32, _config: &RFPConfig) -> i32 {
    ((10 * depth * depth) - (64 * improvement / 1024) + (50 * depth)).max(10 * depth)
}
