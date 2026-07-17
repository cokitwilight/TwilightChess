use chess_final::board::{Board, MoveList};
use chess_final::moves::all_legal_moves;

use std::time::Instant;

pub fn perft(board: &mut Board, depth: usize) -> u64 {
    if depth == 0 {
        return 1;
    }

    let mut moves = MoveList::new();
    all_legal_moves(board, board.side_to_move, &mut moves);

    if depth == 1 {
        return moves.len() as u64;
    }

    let mut nodes = 0u64;

    for &mv in moves.iter() {
        let undo = board.make_move(mv);

        nodes += perft(board, depth - 1);

        board.undo_move(undo);
    }

    nodes
}

pub fn divide_perft(board: &mut Board, depth: usize) -> u64 {
    assert!(depth >= 1, "divide_perft depth must be at least 1");

    let mut moves = MoveList::new();
    all_legal_moves(board, board.side_to_move, &mut moves);

    let mut total = 0u64;

    for &mv in moves.iter() {
        let undo = board.make_move(mv);

        let nodes = perft(board, depth - 1);

        board.undo_move(undo);

        println!("{:?}: {}", mv, nodes);

        total += nodes;
    }

    println!("Total: {}", total);

    total
}

pub fn bench_perft(name: &str, fen: &str, depth: usize, expected: u64) {
    let mut board = Board::from_fen(fen).unwrap();

    let start = Instant::now();
    let nodes = perft(&mut board, depth);
    let elapsed = start.elapsed();

    assert_eq!(nodes, expected);

    let seconds = elapsed.as_secs_f64();
    let nps = nodes as f64 / seconds;

    println!(
        "{} depth {}: {} nodes in {:.3}s ({:.0} nps)",
        name, depth, nodes, seconds, nps
    );

    board.assert_hash();
}

pub fn bench_perft_repeated(name: &str, fen: &str, depth: u32, runs: usize) {
    let mut results = Vec::new();
    let mut expected_nodes = None;

    // Warmup
    {
        let mut board = Board::from_fen(fen).unwrap();
        let _ = perft(&mut board, 1);
    }

    for _ in 0..runs {
        let mut board = Board::from_fen(fen).unwrap();

        let start = Instant::now();
        let nodes = perft(&mut board, depth as usize);
        let elapsed = start.elapsed();

        if let Some(expected) = expected_nodes {
            assert_eq!(nodes, expected, "Perft node count changed between runs");
        } else {
            expected_nodes = Some(nodes);
        }

        results.push((nodes, elapsed.as_secs_f64()));
    }

    let nodes = expected_nodes.unwrap();
    let avg_time = results.iter().map(|(_, t)| t).sum::<f64>() / runs as f64;
    let best_time = results
        .iter()
        .map(|(_, t)| *t)
        .fold(f64::INFINITY, f64::min);

    let avg_nps = nodes as f64 / avg_time;
    let best_nps = nodes as f64 / best_time;

    println!("Position: {name}");
    println!("Depth: {depth}");
    println!("Nodes: {nodes}");
    println!("Runs: {runs}");
    println!("Avg time: {:.3}s", avg_time);
    println!("Best time: {:.3}s", best_time);
    println!("Avg NPS: {:.2}M", avg_nps / 1_000_000.0);
    println!("Best NPS: {:.2}M", best_nps / 1_000_000.0);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_perft_test(fen: &str, expected: &[(usize, u64)]) {
        for &(depth, expected_nodes) in expected {
            let mut board = Board::from_fen(fen).unwrap();

            board.assert_hash();

            let nodes = perft(&mut board, depth);

            board.assert_hash();

            assert_eq!(
                nodes, expected_nodes,
                "Perft failed at depth {} for FEN:\n{}",
                depth, fen
            );
        }
    }

    #[test]
    fn perft_start_position() {
        // Trusted values:
        // depth 1 = 20
        // depth 2 = 400
        // depth 3 = 8,902
        // depth 4 = 197,281
        // depth 5 = 4,865,609
        //
        // Depth 6 = 119,060,324, but you probably do not want that in normal unit tests.
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

        run_perft_test(
            fen,
            &[
                (1, 20),
                (2, 400),
                (3, 8_902),
                (4, 197_281),
                // Uncomment once your movegen is fast enough:
                // (5, 4_865_609),
            ],
        );
    }

    #[test]
    fn perft_kiwipete() {
        // Famous "Kiwipete" position.
        //
        // This position is especially useful because it tests:
        // - castling
        // - captures
        // - checks
        // - pinned pieces
        // - complicated sliding attacks
        let fen = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1";

        run_perft_test(
            fen,
            &[
                (1, 48),
                (2, 2_039),
                (3, 97_862),
                // Uncomment once your movegen is fast enough:
                (4, 4_085_603),
                //(5, 193_690_690),
            ],
        );
    }
    #[test]
    #[ignore] // this test is very slow, so ignore it by default.
    fn perft_kiwipete_depth_5() {
        let fen = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1";
        let mut board = Board::from_fen(fen).unwrap();

        let start = std::time::Instant::now();
        let nodes = perft(&mut board, 5);
        let elapsed = start.elapsed();

        println!(
            "kiwipete depth 5: nodes={} expected={} time={:.3}s nps={:.0}",
            nodes,
            193_690_690u64,
            elapsed.as_secs_f64(),
            nodes as f64 / elapsed.as_secs_f64()
        );

        assert_eq!(nodes, 193_690_690);
        board.assert_hash();
    }

    #[test]
    fn perft_position_3() {
        // Chess Programming Wiki Position 3.
        //
        // This is useful for testing:
        // - promotions
        // - endgame move generation
        // - checks
        let fen = "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1";

        run_perft_test(
            fen,
            &[
                (1, 14),
                (2, 191),
                (3, 2_812),
                (4, 43_238),
                // Uncomment once your movegen is fast enough:
                // (5, 674_624),
                // (6, 11_030_083),
            ],
        );
    }

    #[test]
    fn perft_position_4() {
        // Chess Programming Wiki Position 4.
        //
        // This position has many promotions and tactical edge cases.
        let fen = "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1";

        run_perft_test(
            fen,
            &[
                (1, 6),
                (2, 264),
                (3, 9_467),
                // Uncomment once your movegen is fast enough:
                // (4, 422_333),
                // (5, 15_833_292),
            ],
        );
    }

    #[test]
    fn perft_position_4_mirrored() {
        // Mirrored version of Position 4.
        //
        // Very useful because if this fails but Position 4 passes,
        // you probably have a color-direction bug.
        let fen = "r2q1rk1/pP1p2pp/Q4n2/bbp1p3/Np6/1B3NBn/pPPP1PPP/R3K2R b KQ - 0 1";

        run_perft_test(
            fen,
            &[
                (1, 6),
                (2, 264),
                (3, 9_467),
                // Uncomment once your movegen is fast enough:
                // (4, 422_333),
                // (5, 15_833_292),
            ],
        );
    }

    #[test]
    fn perft_position_5() {
        // Chess Programming Wiki Position 5.
        let fen = "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8";

        run_perft_test(
            fen,
            &[
                (1, 44),
                (2, 1_486),
                (3, 62_379),
                // Uncomment once your movegen is fast enough:
                // (4, 2_103_487),
                // (5, 89_941_194),
            ],
        );
    }

    #[test]
    fn perft_position_6() {
        // Chess Programming Wiki Position 6.
        let fen = "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10";

        run_perft_test(
            fen,
            &[
                (1, 46),
                (2, 2_079),
                (3, 89_890),
                // Uncomment once your movegen is fast enough:
                // (4, 3_894_594),
                // (5, 164_075_551),
            ],
        );
    }

    #[test]
    fn perft_bench_kiwipete() {
        bench_perft_repeated(
            "Kiwipete",
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
            4,
            5,
        );
    }
}
