use std::time::{Duration, Instant};

use chess_final::board::{Board, STARTPOS_FEN};
use chess_final::engine::configs::EngineConfig;
use chess_final::engine::{Engine, SearchLimits, SearchTermination};

fn timed_search(limit_ms: u64) -> (Duration, chess_final::engine::SearchResult) {
    let board = Board::from_fen(STARTPOS_FEN).expect("valid start position");
    let mut config = EngineConfig::default();
    config.tt_size = 1;
    config.limits = SearchLimits::depth_and_time(64, 6, limit_ms);
    let mut engine = Engine::new(config);

    let start = Instant::now();
    let result = engine.search(&board, config.limits, &vec![board.hash()], false, false);

    (start.elapsed(), result)
}

#[test]
fn diagnostic_search_elapsed_tracks_the_configured_limit_and_wall_clock() {
    for limit_ms in [20, 80] {
        let (wall_elapsed, result) = timed_search(limit_ms);
        eprintln!(
            "limit={limit_ms}ms result={:?} wall={:?} depth={} nodes={}",
            result.elapsed,
            wall_elapsed,
            result.depth_reached,
            result.stats.total_nodes(),
        );

        assert!(result.best_move.is_some());
        assert_eq!(result.termination, SearchTermination::TimeLimit);
        assert!(result.elapsed >= Duration::from_millis(limit_ms));
        assert!(wall_elapsed.abs_diff(result.elapsed) < Duration::from_millis(20));
    }
}

#[test]
fn expired_limit_still_returns_a_fallback_move_and_full_elapsed_time() {
    let (wall_elapsed, result) = timed_search(0);

    assert!(result.best_move.is_some());
    assert_eq!(result.depth_reached, 0);
    assert_eq!(result.termination, SearchTermination::TimeLimit);
    assert!(wall_elapsed.abs_diff(result.elapsed) < Duration::from_millis(20));
}

#[test]
fn depth_limit_can_finish_before_a_larger_time_limit() {
    let board = Board::from_fen(STARTPOS_FEN).expect("valid start position");
    let mut config = EngineConfig::default();
    config.tt_size = 1;
    config.limits = SearchLimits::depth_and_time(1, 6, 1_000);
    let mut engine = Engine::new(config);

    let result = engine.search(&board, config.limits, &vec![board.hash()], false, false);

    assert_eq!(result.depth_reached, 1);
    assert_eq!(result.termination, SearchTermination::DepthLimit);
    assert_eq!(*result.pv.first().unwrap(), result.best_move);
    assert!(result.elapsed < Duration::from_millis(1_000));
}
