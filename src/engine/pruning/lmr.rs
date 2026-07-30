pub fn lmr_reduction(depth: u16, move_index: usize) -> u16 {
    if depth < 3 || move_index < 3 {
        return 0;
    }

    let r = match (depth, move_index) {
        (12.., 14..) => 4,
        (10.., 18..) => 3,
        (7.., 14..) => 2,
        (5.., 7..) => 1,
        _ => 0,
    };

    r.min(depth - 2)
}
