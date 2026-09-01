use crate::board::Move;
use crate::engine::history::HistoryKey;
use crate::types::PieceType;

#[derive(Clone, Debug, Copy)]
pub struct SearchStackEntry {
    pub mv: Option<Move>, // the incoming mv. The move that produced the node

    pub piece: Option<PieceType>, // the piece type of mv

    pub history_index: Option<HistoryKey>, // the history key of mv

    pub static_eval: Option<i32>, // current static eval. NOT the static eval from the incoming node.
}

impl Default for SearchStackEntry {
    fn default() -> Self {
        Self {
            mv: None,
            piece: None,
            history_index: None,
            static_eval: None,
        }
    }
}
