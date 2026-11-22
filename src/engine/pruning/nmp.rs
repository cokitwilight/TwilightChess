#[inline]
pub fn null_move_reduction(
    beta: i32,
    score: i32,
    depth: i32,
    improving_strong: bool,
    improving_weak: bool,
) -> i32 {
    ((3000
        + (1000 * improving_strong as i32)
        + (250 * improving_weak as i32)
        + (200 * depth)
        + (400 * (score - beta).clamp(0, 500) / 128))
        / 1024)
        .max(2)
}

#[inline]
pub fn calculate_nmp_margin(beta: i32, depth: i32, improvement: i32, tt_pv: bool) -> i32 {
    // tt pv represents a tt pv but not a current node pv
    beta + (-7 * depth) + (150 * tt_pv as i32) - (60 * improvement / 1024) + 300
}
