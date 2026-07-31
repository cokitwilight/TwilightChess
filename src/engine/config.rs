pub const CHECKMATE_SCORE: i32 = 30_000;
pub const MAX_PLY: i32 = 256;

pub const MATE_THRESHOLD: i32 = CHECKMATE_SCORE - MAX_PLY;

pub const NEG_INF: i32 = -1_000_000_000;
pub const POS_INF: i32 = 1_000_000_000;

pub const MAX_Q_DEPTH: usize = 4;

pub const RFP_MAX_DEPTH: usize = 4;
