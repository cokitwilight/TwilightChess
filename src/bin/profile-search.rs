use chess_final::board::{Board, STARTPOS_FEN};
use chess_final::engine::configs::EngineConfig;
use chess_final::engine::{Engine, SearchLimits};

fn main() {
    let board = Board::from_fen(STARTPOS_FEN).expect("valid FEN");

    let mut config = EngineConfig::standard();
    config.tt_size = 128;
    config.limits = SearchLimits::depth_and_time(64, 20, 10_000);

    let mut engine = Engine::new(config);
    let history = vec![board.hash()];

    let result = engine.search(
        &board,
        config.limits,
        &history,
        false, // Disable the opening book.
        false, // Disable search printing.
    );

    println!(
        "depth={} nodes={} elapsed={:?} best={:?}",
        result.depth_reached,
        result.stats.total_nodes(),
        result.elapsed,
        result.best_move,
    );
}
