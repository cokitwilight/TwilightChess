use crate::board::Move;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TTFlag {
    Exact,
    LowerBound,
    UpperBound,
}

#[derive(Clone, Copy, Debug)]
pub struct TTEntry {
    pub hash: u64,
    pub depth: usize,
    pub eval: i32,
    pub best_move: Option<Move>,
    pub flag: TTFlag,
}
