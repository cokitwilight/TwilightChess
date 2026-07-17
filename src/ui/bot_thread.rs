use crate::board::Board;
use crate::engine::{Engine, SearchLimits, SearchResult};

pub struct BotSearchRequest {
    pub board: Board,
    pub limits: SearchLimits,
    pub repetition_history: Vec<u64>,
    pub engine: Engine,
}

pub struct BotSearchResponse {
    pub engine: Engine,
    pub result: SearchResult,
}
