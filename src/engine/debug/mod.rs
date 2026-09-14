//! Instrumentation-free mirror of the engine search.
//!
//! Keep algorithmic changes synchronized with `engine::search`; this module
//! intentionally duplicates the search so release builds can omit detailed
//! counter updates from the hot path.

mod negamax;
mod quiescence;
mod search;

pub use search::search_without_stats;
