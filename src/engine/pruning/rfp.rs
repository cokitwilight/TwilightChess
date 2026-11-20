use crate::engine::configs::RFPConfig;

#[inline]
pub fn rfp_margin(depth: i32, _improvement: i32, config: &RFPConfig) -> i32 {
    config.margin_factor * depth
}
