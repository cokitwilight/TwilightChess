use crate::engine::configs::EngineConfig;
use crate::engine::history::HistoryTables;
use crate::engine::tt::{TTEntry, TranspositionTable};
use crate::opening::{OpeningBook, build_opening_book};

#[derive(Debug)]
pub struct Engine {
    pub tt: TranspositionTable<TTEntry>,

    pub history: HistoryTables,
    // pub options: EngineOptions,
    pub opening_book: OpeningBook,

    pub config: EngineConfig,
}

impl Engine {
    pub fn new(config: EngineConfig) -> Self {
        Self {
            tt: TranspositionTable::new(config.tt_size), // IN MB
            history: HistoryTables::new(),
            opening_book: build_opening_book(),
            config,
        }
    }
}
