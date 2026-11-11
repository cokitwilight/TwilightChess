
#[inline]
pub fn null_move_reduction(depth: u16) -> u16 {
    if depth >= 6 { 3 } else { 2 }
}

#[inline]
pub fn calculate_nmp_margin(beta: i32) -> i32 {
    // TEMP: Only return beta for now
    beta
}
