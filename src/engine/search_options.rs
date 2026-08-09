use crate::board::Move;

#[derive(Clone, Copy, Debug)]
pub struct SearchOptions {
    pub allow_null_move: bool,
    pub allow_singular: bool,
    pub excluded_move: Option<Move>,
}

impl SearchOptions {
    pub const NORMAL: Self = Self {
        allow_null_move: true,
        allow_singular: true,
        excluded_move: None,
    };

    pub fn singular_verification(excluded_move: Move) -> Self {
        Self {
            allow_null_move: false,
            allow_singular: false,
            excluded_move: Some(excluded_move),
        }
    }
}
