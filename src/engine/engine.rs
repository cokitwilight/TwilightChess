use crate::engine::configs::EngineConfig;
use crate::engine::ordering::HistoryTable;
use crate::engine::tt::{TTEntry, TranspositionTable};
use crate::opening::{OpeningBook, build_opening_book};

#[derive(Debug)]
pub struct Engine {
    pub tt: TranspositionTable<TTEntry>,

    pub history: HistoryTable,
    // pub options: EngineOptions,
    pub opening_book: OpeningBook,

    pub config: EngineConfig,
}

impl Engine {
    pub fn new(config: EngineConfig) -> Self {
        Self {
            tt: TranspositionTable::new(config.tt_size), // IN MB
            history: HistoryTable::new(),
            opening_book: build_opening_book(),
            config,
        }
    }
}
