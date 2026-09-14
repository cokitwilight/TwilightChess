pub fn lmp_threshold(depth: usize) -> usize {
    5 + depth * depth
}

pub fn lmp_history_limit(depth: usize) -> i32 {
    -500 + depth as i32 * 200
}
