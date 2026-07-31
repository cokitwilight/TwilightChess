use crate::board::Move;
use crate::engine::MATE_THRESHOLD;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TTFlag {
    Exact,
    LowerBound,
    UpperBound,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TTNodeType {
    Main,
    Quiescence,
}

#[derive(Clone, Copy, Debug)]
pub struct TTEntry {
    pub depth: u16,
    pub eval: i32,
    pub best_move: Option<Move>,
    pub flag: TTFlag,
    pub node_type: TTNodeType,
}

#[inline(always)]
pub fn score_to_tt(score: i32, ply: usize) -> i32 {
    let ply = ply as i32;

    if score >= MATE_THRESHOLD {
        // Winning mate: convert root-relative distance
        // into position-relative distance.
        score + ply
    } else if score <= -MATE_THRESHOLD {
        // Losing mate.
        score - ply
    } else {
        score
    }
}

#[inline(always)]
pub fn score_from_tt(score: i32, ply: usize) -> i32 {
    let ply = ply as i32;

    if score >= MATE_THRESHOLD {
        score - ply
    } else if score <= -MATE_THRESHOLD {
        score + ply
    } else {
        score
    }
}
