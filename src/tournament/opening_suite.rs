//! Backwards-compatible tournament names for the shared opening book.
//!
//! The opening definitions and behavior live in `crate::opening`; this module
//! only keeps the former tournament API available to callers.

pub use crate::opening::{
    OpeningBook as OpeningSuite, OpeningLine as OpeningPosition,
    build_important_opening_book as build_important_opening_suite,
    build_opening_book as build_opening_suite,
    build_suggestion_opening_book as build_suggestion_opening_suite,
};
