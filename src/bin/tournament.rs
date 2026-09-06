use std::env;
use std::process::ExitCode;

use chess_final::engine::configs::EngineConfig;
use chess_final::opening::{
    build_important_opening_book, build_opening_book, build_suggestion_opening_book,
};
use chess_final::tournament::{MatchPlayers, play_games};
use chess_final::types::Color;

const TOURNAMENTS: &[(&str, fn())] = &[
    (
        "test_tournament_different_depth",
        tournament_different_depth,
    ),
    (
        "test_tournament_different_configs",
        tournament_different_configs,
    ),
    ("test_tournament_same_nodes", tournament_same_nodes),
    ("test_tournament_lmr", tournament_lmr),
    ("test_tournament_null", tournament_null),
    ("test_tournament_rfp", tournament_rfp),
    ("test_tournament_fut", tournament_fut),
    ("test_tournament_see", tournament_see),
    ("test_tournament_singular", tournament_singular),
    ("test_tournament_suggestion", tournament_suggestion),
];

fn main() -> ExitCode {
    let mut args = env::args().skip(1);
    let Some(tournament_name) = args.next() else {
        print_usage();
        return ExitCode::FAILURE;
    };

    if tournament_name == "--help" || tournament_name == "-h" {
        print_usage();
        return ExitCode::SUCCESS;
    }

    if args.next().is_some() {
        eprintln!("Only one tournament name may be supplied.\n");
        print_usage();
        return ExitCode::FAILURE;
    }

    let Some((_, run)) = TOURNAMENTS
        .iter()
        .find(|(name, _)| *name == tournament_name)
    else {
        eprintln!("Unknown tournament: {tournament_name}\n");
        print_usage();
        return ExitCode::FAILURE;
    };

    run();
    ExitCode::SUCCESS
}

fn print_usage() {
    eprintln!("Usage: cargo run --release --bin tournament -- <tournament-name>");
    eprintln!("\nAvailable tournaments:");
    for (name, _) in TOURNAMENTS {
        eprintln!("  {name}");
    }
}

fn tournament_different_depth() {
    let mut match_players =
        MatchPlayers::from_depth("Depth 11".to_string(), "Depth 10".to_string(), 11, 6, 10, 6);

    let opening_suite = build_important_opening_book();

    match_players.white.config.search.fut.enabled = true;
    match_players.black.config.search.fut.enabled = true;

    match_players.white.config.limits.soft_time_limit_ms = Some(5000000);
    match_players.black.config.limits.soft_time_limit_ms = Some(5000000);

    let result = play_games(opening_suite, 1, 5, match_players, Color::White);

    result.print_stats();
}

fn tournament_different_configs() {
    let mut match_players = MatchPlayers::from_depth(
        "Singular on".to_string(),
        "Singular off".to_string(),
        10,
        6,
        10,
        6,
    );

    match_players.white.config.limits.soft_time_limit_ms = Some(1000);
    match_players.black.config.limits.soft_time_limit_ms = Some(1000);

    // match_players.white.config.search.fut.enabled = false;
    // match_players.black.config.search.fut.enabled = false;

    // // might be quiescence rewrite
    // match_players.white.config.search.rfp.enabled = false;
    // match_players.black.config.search.rfp.enabled = false;

    // match_players.white.config.search.null_move.enabled = false;
    // match_players.black.config.search.null_move.enabled = false;

    // match_players.white.config.search.lmr.enabled = false;
    // match_players.black.config.search.lmr.enabled = false;

    match_players.white.config.search.singular.enabled = true;
    match_players.white.config.search.singular.minimum_depth = 6;
    match_players.black.config.search.singular.enabled = false;

    let opening_suite = build_opening_book();

    let result = play_games(opening_suite, 1, 300, match_players, Color::White);

    result.review().expect("IO Error");
}

fn tournament_same_nodes() {
    let mut match_players = MatchPlayers::from_depth(
        "Null Move on".to_string(),
        "Null Move off".to_string(),
        20,
        6,
        20,
        6,
    );

    match_players.white.config.limits.max_nodes = Some(1000000);
    match_players.black.config.limits.max_nodes = Some(1000000);

    match_players.white.config.search.fut.enabled = false;
    match_players.black.config.search.fut.enabled = false;

    match_players.white.config.search.delta.enabled = false;
    match_players.black.config.search.delta.enabled = false;

    match_players.white.config.search.rfp.enabled = false;
    match_players.black.config.search.rfp.enabled = false;

    match_players.white.config.search.null_move.enabled = true;
    match_players.black.config.search.null_move.enabled = false;

    match_players.white.config.search.lmr.enabled = true;
    match_players.black.config.search.lmr.enabled = true;

    let opening_suite = build_important_opening_book();

    let result = play_games(opening_suite, 1, 5, match_players, Color::White);

    result.print_stats();
}

fn tournament_lmr() {
    let mut match_players = MatchPlayers::from_depth(
        "LMR History on".to_string(),
        "LMR History off".to_string(),
        24,
        6,
        24,
        6,
    );

    match_players.white.config = EngineConfig::standard();
    match_players.black.config = EngineConfig::standard();

    match_players.white.config.search.lmr.enabled = true;
    match_players.white.config.search.lmr.history_enabled = true;
    match_players.white.config.search.lmr.history_scale = 94;
    match_players.black.config.search.lmr.enabled = true;
    match_players.black.config.search.lmr.history_enabled = false;

    let opening_suite = build_opening_book();

    let result = play_games(opening_suite, 6, 1200, match_players, Color::White);

    result.review().expect("IO Error");
}

fn tournament_null() {
    let mut match_players =
        MatchPlayers::from_depth("Null on".to_string(), "Null off".to_string(), 12, 6, 12, 6);

    match_players.white.config = EngineConfig::standard();
    match_players.black.config = EngineConfig::standard();

    match_players.white.config.search.null_move.enabled = true;
    match_players.black.config.search.null_move.enabled = false;

    let opening_suite = build_opening_book();

    let result = play_games(opening_suite, 8, 880, match_players, Color::White);

    result.review().expect("IO Error");
}

fn tournament_rfp() {
    let mut match_players =
        MatchPlayers::from_depth("RFP on".to_string(), "RFP off".to_string(), 12, 6, 12, 6);

    match_players.white.config = EngineConfig::standard();
    match_players.black.config = EngineConfig::standard();

    match_players.white.config.search.rfp.enabled = true;
    match_players.black.config.search.rfp.enabled = false;

    let opening_suite = build_opening_book();

    let result = play_games(opening_suite, 5, 1000, match_players, Color::White);

    result.review().expect("IO Error");
}

fn tournament_fut() {
    let mut match_players = MatchPlayers::from_depth(
        "FUT History on".to_string(),
        "FUT History off".to_string(),
        20,
        6,
        20,
        6,
    );

    match_players.white.config = EngineConfig::standard();
    match_players.black.config = EngineConfig::standard();

    match_players.white.config.search.fut.enabled = true;
    match_players.white.config.search.fut.history_enabled = true;
    match_players.black.config.search.fut.enabled = true;
    match_players.black.config.search.fut.history_enabled = false;

    let opening_suite = build_opening_book();

    let result = play_games(opening_suite, 8, 880, match_players, Color::White);

    result.review().expect("IO Error");
}

fn tournament_see() {
    let mut match_players =
        MatchPlayers::from_depth("SEE 250".to_string(), "SEE 200".to_string(), 12, 6, 12, 6);

    match_players.white.config = EngineConfig::standard();
    match_players.black.config = EngineConfig::standard();

    match_players.white.config.search.see.enabled = true;
    match_players.black.config.search.see.enabled = true;

    match_players.white.config.search.see.margin = 250;
    match_players.black.config.search.see.margin = 200;

    let opening_suite = build_opening_book();

    let result = play_games(opening_suite, 2, 150, match_players, Color::White);

    result.review().expect("IO Error");
}

fn tournament_singular() {
    let mut match_players = MatchPlayers::from_depth(
        "Singular On".to_string(),
        "Singular Off".to_string(),
        20,
        6,
        20,
        6,
    );

    match_players.white.config = EngineConfig::standard();
    match_players.black.config = EngineConfig::standard();

    match_players.white.config.search.singular.enabled = true;
    match_players.black.config.search.singular.enabled = false;

    let opening_suite = build_opening_book();

    let result = play_games(opening_suite, 5, 1000, match_players, Color::White);

    result.review().expect("IO Error");
}

fn tournament_suggestion() {
    let mut match_players =
        MatchPlayers::from_depth("Engine 1".to_string(), "Engine 2".to_string(), 30, 6, 30, 6);

    match_players.white.config.limits.soft_time_limit_ms = Some(1000);
    match_players.black.config.limits.soft_time_limit_ms = Some(1000);

    let opening_suite = build_suggestion_opening_book();

    let result = play_games(opening_suite, 1, 100, match_players, Color::White);

    result.review().expect("IO Error");
}
