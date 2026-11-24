use crate::types::Score;

// Compatibility aliases; search code uses the score namespace directly.
pub const CHECKMATE_SCORE: i32 = Score::MATE;
pub const MAX_PLY: usize = 256;

pub const MATE_THRESHOLD: i32 = Score::MATE_IN_MAX;

pub const NEG_INF: i32 = -Score::INFINITE;
pub const POS_INF: i32 = Score::INFINITE;

pub const RFP_MAX_DEPTH: usize = 4;
