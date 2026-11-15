use std::time::Duration;

use crate::board::Move;
use crate::engine::SearchStats;

#[cfg(feature = "debug-uci")]
pub const MAX_PV: usize = crate::engine::MAX_PLY;
#[cfg(not(feature = "debug-uci"))]
pub const MAX_PV: usize = 1;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SearchTermination {
    DepthLimit,
    TimeLimit,
    NodeLimit,
    BookMove,
    /// The iteration callback requested a stop after a fully completed depth.
    IterationStop,
    /// Explicit cancellation by the debugging protocol.
    Cancelled,
}

// what is actually returned from a search, including the best move found, the evaluation score, the depth reached, and the principal variation.
#[derive(Clone, Debug)]
pub struct SearchResult {
    pub best_move: Option<Move>,
    pub eval: i32,
    pub depth_reached: u16,
    /// Deepest root-relative ply visited in the last completed iteration.
    pub seldepth: usize,
    pub stats: SearchStats,
    /// Searched PV prefix. Lightweight debug UCI searches capture continuations;
    /// other entry points supply the root move. TT cutoffs can end the line.
    pub pv: [Option<Move>; MAX_PV],
    pub elapsed: Duration,
    pub termination: SearchTermination,
}
