pub mod eval;
pub mod king;
pub mod knight;
pub mod material;
pub mod mobility;
pub mod pawn;
pub mod phase;
pub mod pst;
pub mod queen;
pub mod rook;
pub mod sliders;

pub use eval::{evaluation_for_turn, lazy_eval_for_turn};
