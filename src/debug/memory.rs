#[test]
pub fn print_memory_sizes() {
    use crate::board::{Move, MoveType};
    use crate::engine::staged::ScoredMove;
    use crate::engine::tt::{TTEntry, TTFlag, TTNodeType, table::TTCluster, table::TTSlot};
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

    println!("Option<TTSlot>: {}", std::mem::size_of::<Option<TTSlot>>());

    // Option<TTSlot>

    println!();

    let entry = TTEntry {
        depth: 0,
        eval: 0,
        best_move: None,
        flag: TTFlag::Exact,
        node_type: TTNodeType::Main,
    };
    print_layout!(TTEntry, entry, depth, eval, best_move, flag, node_type);

    let slot = TTSlot {
        key: 0,
        entry,
        generation: 0,
    };
    print_layout!(TTSlot, slot, key, entry, generation);

    println!("TTCluster: {}", std::mem::size_of::<TTCluster>());
}
