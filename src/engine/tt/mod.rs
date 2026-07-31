pub mod entry;
pub mod table;
pub mod table_stats;

pub use entry::{TTEntry, TTFlag, TTNodeType, score_from_tt, score_to_tt};
pub use table::TranspositionTable;
pub use table_stats::TableStats;
