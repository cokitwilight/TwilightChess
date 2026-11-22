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
    /// The solution finder confirmed a solution after a fully completed depth.
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
    /// Searched PV prefix. The regular search currently supplies the root move.
    pub pv: [Option<Move>; MAX_PV],
    pub elapsed: Duration,
    pub termination: SearchTermination,
    /// Completed depths only; an interrupted depth is never added to this history.
    #[cfg(feature = "debug-uci")]
    pub iterations: Vec<SearchIteration>,
}

/// A small snapshot of one completed depth, without copying all search statistics.
/// Node counts and elapsed time are cumulative from the start of the search.
#[cfg(feature = "debug-uci")]
#[derive(Clone, Debug)]
pub struct SearchIteration {
    pub best_move: Option<Move>,
    pub eval: i32,
    pub depth_reached: u16,
    pub seldepth: usize,
    pub main_nodes: u64,
    pub quiescence_nodes: u64,
    pub pv: [Option<Move>; MAX_PV],
    pub elapsed: Duration,
}

#[cfg(feature = "debug-uci")]
impl SearchIteration {
    pub(crate) fn from_result(result: &SearchResult) -> Self {
        Self {
            best_move: result.best_move,
            eval: result.eval,
            depth_reached: result.depth_reached,
            seldepth: result.seldepth,
            main_nodes: result.stats.node_stats.main,
            quiescence_nodes: result.stats.node_stats.quiescence,
            pv: result.pv,
            elapsed: result.elapsed,
        }
    }
}
