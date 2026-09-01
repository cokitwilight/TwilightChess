pub mod iterative;
pub mod negamax;
pub mod quiescence;
pub mod search;

pub use search::{adjusted_depth_for_phase, is_insufficient_material};
