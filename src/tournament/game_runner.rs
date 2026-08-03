use std::time::Duration;

use crate::engine::Engine;
use crate::game::{Game, GameState};
use crate::tournament::opening_suite::OpeningPosition;
use crate::tournament::results::GameResult::{Black, Draw, White};
use crate::tournament::{GameRecord, MatchPlayers, MoveRecord};
use crate::types::Color;
use crate::uci::pgn::{PgnMetadata, game_to_pgn};

pub fn run_game(
    start_fen: String,
    players: MatchPlayers,
    engine_1_color: Color,
    opening_line: Option<OpeningPosition>,
) -> Result<GameRecord, String> {
    let mut game_record = GameRecord::new(start_fen.clone(), engine_1_color);

    if let Some(opening) = &opening_line {
        game_record.opening_name = Some(opening.name.clone());
    }

    let w_config = players.white.config;
    let b_config = players.black.config;

    let mut count = 0;

    let mut game = if let Some(opening) = opening_line {
        opening.create_game()
    } else {
        Game::from_fen(&start_fen).unwrap()
    };

    // NOTE: This might be expensive as it allocates an entire new engine(with tt for example)
    let mut white = Engine::new(w_config.clone());
    let mut black = Engine::new(b_config.clone());

    let mut w_count = 0;
    let mut b_count = 0;

    while game.state() == GameState::Ongoing {
        if count >= 600 {
            // likely infinite loop
            let metadata = PgnMetadata {
                event: "Engine Test".to_string(),
                site: "Local".to_string(),
                date: "2026.07.27".to_string(),
                round: "1".to_string(),
                white: "White".to_string(),
                black: "Black".to_string(),
            };

            let pgn_text = game_to_pgn(&game, &metadata).expect("Game Failed");

            println!("{pgn_text}");
            return Err("Likely Infinite Loop in Run Game".to_string());
        }

        let search_result = match game.board.side_to_move() {
            Color::White => white.search(
                &game.board,
                w_config.limits,
                &game.repetition_history,
                false,
                false,
            ),
            Color::Black => black.search(
                &game.board,
                b_config.limits,
                &game.repetition_history,
                false,
                false,
            ),
        };

        let best_move = search_result
            .best_move
            .expect("No best move in white.search in tournament/run_game");
        // game already keeps track of move history and repetition history

        let mut eval = search_result.eval;

        eval = match game.board.side_to_move() {
            Color::White => eval,
            Color::Black => -eval,
        };

        game_record.move_history.push(MoveRecord {
            mv: best_move,
            eval,
        });

        match game.board.side_to_move() {
            Color::White => {
                w_count += 1;
                game_record.white_time_elapsed += search_result.elapsed;
                game_record.white_stats += search_result.stats;
            }
            Color::Black => {
                b_count += 1;
                game_record.black_time_elapsed += search_result.elapsed;
                game_record.black_stats += search_result.stats;
            }
        }

        match game.play_move(best_move) {
            Ok(()) => {}
            Err(err) => {
                let err_message = format!("Failed to play move {best_move}: {:?}", err);
                return Err(err_message);
            }
        }

        game_record.total_time += search_result.elapsed;
        count += 1;
    }

    match game.state() {
        GameState::Checkmate { winner } => match winner {
            Color::White => game_record.result = Some(White),
            Color::Black => game_record.result = Some(Black),
        },
        GameState::Ongoing => {
            panic!("Some game is still ongoing");
        }
        _ => game_record.result = Some(Draw),
    }

    if w_count != 0 {
        game_record.white_avg_time = game_record.white_time_elapsed / w_count;
    } else {
        game_record.white_avg_time = Duration::ZERO;
    }

    if b_count != 0 {
        game_record.black_avg_time = game_record.black_time_elapsed / b_count;
    } else {
        game_record.black_avg_time = Duration::ZERO;
    }

    game_record.game = game;

    Ok(game_record)
}

#[cfg(test)]
mod tests {
    use crate::{
        board::STARTPOS_FEN,
        uci::pgn::{PgnMetadata, game_to_pgn},
    };

    use super::*;

    #[test]
    #[ignore]
    pub fn test_game_1() {
        let players = MatchPlayers::new("white".to_string(), "black".to_string());

        let game_record = match run_game(
            STARTPOS_FEN.to_string(),
            players.clone(),
            Color::White,
            None,
        ) {
            Ok(g) => g,
            Err(msg) => panic!("{msg}"),
        };

        println!(
            "Result: {:?}\n",
            game_record.result.expect("No result in test_game_1")
        );

        println!("White Bot -- {}", players.white.name);
        game_record
            .white_stats
            .print_all(1, game_record.white_time_elapsed.as_secs_f64());
        println!("");
        println!(
            "Total Time as White: {:.3} seconds",
            game_record.white_time_elapsed.as_secs_f64()
        );
        println!(
            "Avg Time Per Move as White: {:.3} seconds",
            game_record.white_avg_time.as_secs_f64()
        );

        println!("");

        println!("Black Bot -- {}", players.black.name);
        game_record
            .black_stats
            .print_all(1, game_record.black_time_elapsed.as_secs_f64());

        println!("");

        println!(
            "Total Time as Black: {:.3} seconds",
            game_record.black_time_elapsed.as_secs_f64()
        );
        println!(
            "Avg Time Per Move as Black: {:.3} seconds",
            game_record.black_avg_time.as_secs_f64()
        );

        println!("");

        println!(
            "Total run time: {:.3} seconds",
            game_record.total_time.as_secs_f64()
        );

        println!("");

        let metadata = PgnMetadata {
            event: "Engine Test".to_string(),
            site: "Local".to_string(),
            date: "2026.07.27".to_string(),
            round: "1".to_string(),
            white: "White".to_string(),
            black: "Black".to_string(),
        };

        let pgn_text = game_to_pgn(&game_record.game, &metadata).expect("Game Failed");

        println!("{pgn_text}");
    }

    #[test]
    #[ignore]
    pub fn test_game_2() {
        let mut players =
            MatchPlayers::from_depth("10 s".to_string(), "8 s".to_string(), 20, 7, 20, 3);

        players.white.config.limits.soft_time_limit_ms = Some(10000);
        players.black.config.limits.soft_time_limit_ms = Some(8000);

        players.white.config.limits.hard_time_limit_ms = Some(25000);
        players.black.config.limits.hard_time_limit_ms = Some(15000);

        let game_record = match run_game(
            STARTPOS_FEN.to_string(),
            players.clone(),
            Color::White,
            None,
        ) {
            Ok(g) => g,
            Err(msg) => panic!("{msg}"),
        };

        println!("White Bot -- {}", players.white.name);
        game_record
            .white_stats
            .print_all(1, game_record.white_time_elapsed.as_secs_f64());
        println!("");
        println!(
            "Total Time as White: {:.3} seconds",
            game_record.white_time_elapsed.as_secs_f64()
        );
        println!(
            "Avg Time Per Move as White: {:.3} seconds",
            game_record.white_avg_time.as_secs_f64()
        );

        println!("");

        println!("Black Bot -- {}", players.black.name);
        game_record
            .black_stats
            .print_all(1, game_record.black_time_elapsed.as_secs_f64());

        println!("");

        println!(
            "Total Time as Black: {:.3} seconds",
            game_record.black_time_elapsed.as_secs_f64()
        );
        println!(
            "Avg Time Per Move as Black: {:.3} seconds",
            game_record.black_avg_time.as_secs_f64()
        );

        println!("");

        println!(
            "Total run time: {:.3} seconds",
            game_record.total_time.as_secs_f64()
        );

        println!("");

        let metadata = PgnMetadata {
            event: "Engine Test".to_string(),
            site: "Local".to_string(),
            date: "2026.07.27".to_string(),
            round: "1".to_string(),
            white: "10 s".to_string(),
            black: "8 s".to_string(),
        };

        let pgn_text = game_to_pgn(&game_record.game, &metadata).expect("Game Failed");

        println!("{pgn_text}");

        println!("");

        println!(
            "Result: {:?}\n",
            game_record.result.expect("No result in test_game_1")
        );
    }
}
