use crate::board::Move;
use crate::engine::SearchStats;

// what is actually returned from a search, including the best move found, the evaluation score, the depth reached, and the principal variation.
#[derive(Clone, Debug)]
pub struct SearchResult {
    pub best_move: Option<Move>,
    pub eval: i32,
    pub depth_reached: usize,
    pub stats: SearchStats,
    pub pv: Vec<Move>,
}
