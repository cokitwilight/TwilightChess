use crate::engine::configs::EngineConfig;
use crate::engine::history::HistoryTables;
use crate::engine::pruning::LmrTable;
use crate::engine::tt::{TTEntry, TranspositionTable};
use crate::opening::{OpeningBook, build_opening_book};

#[derive(Debug)]
pub struct Engine {
    pub tt: TranspositionTable<TTEntry>,

    pub history: HistoryTables,
    pub lmr_table: LmrTable,
    pub opening_book: OpeningBook,

    pub config: EngineConfig,
}

impl Engine {
    pub fn new(config: EngineConfig) -> Self {
        Self {
            tt: TranspositionTable::new(config.tt_size), // IN MB
            history: HistoryTables::new(),
            lmr_table: LmrTable::new(&config.search.lmr),
            opening_book: build_opening_book(),
            config,
        }
    }
}
