#[test]
pub fn print_memory_sizes() {
    use std::mem::size_of;

    use crate::{
        board::{Move, MoveType},
        engine::ordering::staged::ScoredMove,
        types::PieceType,
    };
    println!("Move: {}", std::mem::size_of::<Move>());
    println!("MoveType: {}", std::mem::size_of::<MoveType>());
    println!(
        "Option<PieceType>: {}",
        std::mem::size_of::<Option<PieceType>>()
    );
    println!("Scored Move: {}", std::mem::size_of::<ScoredMove>());
    println!(
        "Option<Scored Move>: {}",
        std::mem::size_of::<Option<ScoredMove>>()
    );
}
