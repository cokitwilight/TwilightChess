#[test]
pub fn print_memory_sizes() {
    use crate::board::{Move, MoveType};
    use crate::engine::staged::ScoredMove;
    use crate::engine::tt::{TTEntry, TTFlag, table::TTCluster};
    use crate::types::PieceType;

    use std::mem::{align_of, offset_of, size_of, size_of_val};

    macro_rules! print_layout {
        ($ty:ty, $value:expr, $($field:ident),+ $(,)?) => {{
            let value = $value;
            let mut fields = [$(
                (stringify!($field), offset_of!($ty, $field), size_of_val(&value.$field)),
            )+];
            // Rust may reorder fields, so report them in actual memory order.
            fields.sort_by_key(|&(_, offset, _)| offset);

            println!("{} (alignment: {} bytes)", stringify!($ty), align_of::<$ty>());
            let mut end = 0;
            for (name, offset, size) in fields {
                if offset > end {
                    println!("PADDING: {} bytes", offset - end);
                }
                end = offset + size;
                println!("{name}: bytes {offset}..{end}");
            }
            if size_of::<$ty>() > end {
                println!("PADDING: {} bytes", size_of::<$ty>() - end);
            }
            println!("TOTAL: {} bytes\n", size_of::<$ty>());
        }};
    }

    println!("Move: {}", std::mem::size_of::<Move>());
    println!("Option<Move>: {}", std::mem::size_of::<Option<Move>>());
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

    println!(
        "Option<TTEntry>: {}",
        std::mem::size_of::<Option<TTEntry>>()
    );

    // Option<TTSlot>

    println!();

    let entry = TTEntry::new(0, false, TTFlag::Exact, 0, 0, 0, 0, None);

    print_layout!(
        TTEntry,
        entry,
        key,
        flag,
        depth,
        eval,
        static_eval,
        best_move
    );

    println!("TTCluster: {}", std::mem::size_of::<TTCluster>());
}
