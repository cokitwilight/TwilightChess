pub mod continuation;
pub mod history;
pub mod killer;
pub mod main_history;

pub use continuation::ContinuationHistory;
pub use history::{HistoryKey, HistoryTables, history_bonus, history_malus, update};
pub use killer::KillerTable;
pub use main_history::MainHistory;
