pub mod book;
mod catalog;

pub use book::{BookMove, OpeningBook, OpeningLine, find_legal_move_from_uci};
pub use catalog::{
    IMPORTANT_OPENINGS, SUGGESTION_OPENINGS, build_important_opening_book, build_opening_book,
    build_suggestion_opening_book,
};
