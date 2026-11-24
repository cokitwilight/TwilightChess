use crate::engine::config::MAX_PLY;

/// Names for search-score values. Arithmetic and packed TT storage remain integers.
/// Ordinary evaluations must stay strictly inside +/-TB_WIN_IN_MAX; higher
/// magnitudes are reserved for tablebase and mate results.
pub struct Score;

impl Score {
    pub const ZERO: i32 = 0;
    pub const MATE: i32 = 30_000;
    pub const INFINITE: i32 = Self::MATE + 1;
    pub const NONE: i32 = Self::INFINITE + 1;
    pub const MATE_IN_MAX: i32 = Self::MATE - MAX_PLY as i32;
    pub const TB_WIN: i32 = Self::MATE_IN_MAX - 1;
    pub const TB_WIN_IN_MAX: i32 = Self::TB_WIN - MAX_PLY as i32;
}

#[inline]
pub const fn mate_in(ply: usize) -> i32 {
    Score::MATE - ply as i32
}

#[inline]
pub const fn mated_in(ply: usize) -> i32 {
    -Score::MATE + ply as i32
}

#[inline]
pub const fn tb_win_in(ply: usize) -> i32 {
    Score::TB_WIN - ply as i32
}

#[inline]
pub const fn tb_loss_in(ply: usize) -> i32 {
    -Score::TB_WIN + ply as i32
}

/// Like Reckless, these range checks require the caller to reject NONE first.
/// Negative infinity deliberately counts as a loss before any move is searched.
#[inline]
pub const fn is_win(score: i32) -> bool {
    score >= Score::TB_WIN_IN_MAX
}

#[inline]
pub const fn is_loss(score: i32) -> bool {
    score <= -Score::TB_WIN_IN_MAX
}

#[inline]
pub const fn is_decisive(score: i32) -> bool {
    is_win(score) || is_loss(score)
}

/// Tests availability, not whether a score is finite or an ordinary evaluation.
#[inline]
pub const fn is_valid(score: i32) -> bool {
    score != Score::NONE
}

/// Identifies an actual mate score, excluding tablebases and sentinels.
#[inline]
pub const fn is_mate(score: i32) -> bool {
    (score >= Score::MATE_IN_MAX && score <= Score::MATE)
        || (score <= -Score::MATE_IN_MAX && score >= -Score::MATE)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decisive_ranges_include_tablebases_but_mate_range_does_not() {
        for ply in [0, 1, MAX_PLY] {
            assert!(is_win(mate_in(ply)));
            assert!(is_loss(mated_in(ply)));
            assert!(is_mate(mate_in(ply)));
            assert!(is_mate(mated_in(ply)));
            assert!(is_win(tb_win_in(ply)));
            assert!(is_loss(tb_loss_in(ply)));
            assert!(!is_mate(tb_win_in(ply)));
            assert!(!is_mate(tb_loss_in(ply)));
        }
        assert!(!is_decisive(Score::TB_WIN_IN_MAX - 1));
        assert!(!is_decisive(-Score::TB_WIN_IN_MAX + 1));
        assert!(!is_decisive(Score::ZERO));
    }

    #[test]
    fn missing_scores_are_distinct_from_zero_and_infinity() {
        assert!(!is_valid(Score::NONE));
        assert!(is_valid(Score::ZERO));
        assert!(is_valid(Score::INFINITE));
        assert!(is_loss(-Score::INFINITE));
        assert!(!is_mate(Score::NONE));
        assert!(!is_mate(Score::INFINITE));
        assert!(!is_mate(-Score::INFINITE));
        assert_eq!(i32::from(Score::NONE as i16), Score::NONE);
    }
}
