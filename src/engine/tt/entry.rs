use crate::board::Move;

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
