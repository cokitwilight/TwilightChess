use crate::engine::ordering::HistoryTable;
use crate::engine::tt::{TTEntry, TranspositionTable};
use crate::opening::{OpeningBook, build_opening_book};

#[derive(Debug)]
pub struct Engine {
    pub tt: TranspositionTable<TTEntry>,
    pub qtt: TranspositionTable<TTEntry>,

    pub history: HistoryTable,
    // pub options: EngineOptions,
    pub opening_book: OpeningBook,
}

impl Engine {
    pub fn new() -> Self {
        Self {
            tt: TranspositionTable::new(72),  // IN MB
            qtt: TranspositionTable::new(64), // IN MB
            history: HistoryTable::new(),
            opening_book: build_opening_book(),
        }
    }
}
