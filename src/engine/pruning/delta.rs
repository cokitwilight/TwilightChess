use crate::engine::configs::DeltaPruneConfig;

#[inline]
pub fn delta_pruning_bound(stand_pat: i32, captured_value: i32, config: &DeltaPruneConfig) -> i32 {
    stand_pat + captured_value + config.margin
}
