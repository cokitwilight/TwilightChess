use crate::engine::SearchStackEntry;
use crate::types::is_valid;

/// Same-side raw evaluation change, or zero when the comparison is unavailable.
#[inline]
pub fn calculate_eval_improvement(
    raw_static_eval: i32,
    stack: &[SearchStackEntry],
    ply: usize,
    in_check: bool,
) -> i32 {
    if in_check || ply < 2 || !is_valid(raw_static_eval) {
        return 0;
    }

    let previous_eval = stack[ply - 2].static_eval;
    if is_valid(previous_eval) {
        raw_static_eval.saturating_sub(previous_eval)
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Score;

    #[test]
    fn unavailable_evaluations_do_not_create_improvement() {
        let mut stack = [SearchStackEntry::default(); 3];
        assert_eq!(stack[0].static_eval, Score::NONE);
        assert_eq!(calculate_eval_improvement(80, &stack, 2, false), 0);
        stack[0].static_eval = 50;
        assert_eq!(calculate_eval_improvement(80, &stack, 2, false), 30);
        assert_eq!(calculate_eval_improvement(80, &stack, 2, true), 0);
        assert_eq!(calculate_eval_improvement(Score::NONE, &stack, 2, false), 0);
    }
}
