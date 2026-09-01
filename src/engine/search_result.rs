use std::time::Duration;

use crate::board::Move;
use crate::engine::SearchStats;

pub const MAX_PV: usize = 1;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SearchTermination {
    DepthLimit,
    TimeLimit,
    NodeLimit,
    BookMove,
}

// what is actually returned from a search, including the best move found, the evaluation score, the depth reached, and the principal variation.
#[derive(Clone, Debug)]
pub struct SearchResult {
    pub best_move: Option<Move>,
    pub eval: i32,
    pub depth_reached: u16,
    pub stats: SearchStats,
    pub pv: [Option<Move>; MAX_PV],
    pub elapsed: Duration,
    pub termination: SearchTermination,
}
