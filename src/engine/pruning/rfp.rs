use crate::engine::configs::RFPConfig;

#[inline]
pub fn rfp_margin(depth: usize, config: &RFPConfig) -> i32 {
    config.margin_factor * depth as i32
}
