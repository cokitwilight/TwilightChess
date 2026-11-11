use crate::engine::configs::SEEConfig;

#[inline]
pub fn calculate_see_threshold(config: &SEEConfig) -> i32 {
    -config.margin // for now
}
