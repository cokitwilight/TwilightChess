use std::time::{Duration, Instant};

use crate::opening::OpeningBook;
use crate::tournament::results::GameResult;
use crate::tournament::{MatchPlayers, NotableGame, NotableReason, TournamentResult, run_game};
use crate::types::Color;

pub fn play_games(
    opening_suite: OpeningBook,
    num_games_per_opening: usize, // how many games to play for each opening in the suite. input 1 = 1 as white, 1 as black. input 2 = 2 as white, 2 as black, etc.
    max_games: usize,
    mut players: MatchPlayers, /* Later: OpeningSuite, Start_fen */
    expected: Color, // For now expected is the starting players color. For example white would be engine 1 or MatchPlayers.white
) -> TournamentResult {
    if num_games_per_opening == 0 {
        panic!("Invalid num_games_per_opening in play_games");
    }

    let mut result = TournamentResult::new(players.white.name.clone(), players.black.name.clone());

    let mut engine_1_current_color = Color::White;

    let mut engine_1_total_time: Duration = Duration::ZERO;
    let mut engine_2_total_time: Duration = Duration::ZERO;

    let mut expected_winner = expected; // for now this value will be swapped for easy checks

    let total_opening_games = num_games_per_opening.saturating_mul(2);

    result.total_games = total_opening_games * opening_suite.openings.len();

    let mut count = 1;

    for opening in opening_suite.openings.iter() {
        if count > max_games {
            break;
        }
        for _ in 0..total_opening_games {
            let start = Instant::now();
            let search_elapsed: Duration;
            match run_game(
                opening.game.starting_fen.clone(),
                players.clone(),
                engine_1_current_color,
                Some(opening.clone()),
                false,
            ) {
                Ok(g) => {
                    search_elapsed = g.total_time;
                    let Some(game_result) = g.result.clone() else {
                        result
                            .invalid_games
                            .push((Some(g), "No valid result in game".to_string()));
                        players.swap();
                        engine_1_current_color = engine_1_current_color.opposite();
                        expected_winner = expected_winner.opposite();
                        count += 1;
                        continue;
                    };

                    let mut notable_game = NotableGame {
                        game_record: g.clone(),
                        reasons: Vec::new(),
                        importance: 0,
                    };

                    if g.game.move_history.is_empty() {
                        result.invalid_games.push((
                            Some(g.clone()),
                            "Completed game has empty move history".to_string(),
                        ));

                        players.swap();
                        engine_1_current_color = engine_1_current_color.opposite();
                        expected_winner = expected_winner.opposite();
                        count += 1;
                        continue;
                    }

                    result.games.push(g.clone());
                    result.valid_games += 1;

                    let game = g;

                    // eval is from whites perspective
                    let mut smallest_eval = 30000;
                    let mut largest_eval = -30000;

                    let mut average_eval = 0;
                    let mut total = 0;

                    let game_length = game.move_history.len();

                    let median_eval = game.move_history[game_length / 2].eval;

                    // look relatively only at the middle game.
                    // Later make this relative
                    let start = (game_length / 5).saturating_sub(3).max(1);

                    let end = (game_length / 2)
                        .saturating_sub(5)
                        .max(start.saturating_add(5))
                        .min(game_length);

                    let mut swings: Vec<(i32, usize)> = Vec::new();
                    for i in start..end {
                        if i >= game_length {
                            break;
                        }
                        let previous_eval = game.move_history[i - 1].eval; // start will always be atleast 1
                        let eval = game.move_history[i].eval;

                        if (previous_eval - eval).abs() >= 150 {
                            swings.push(((eval - previous_eval).abs(), i + 1));
                        }

                        total += eval;

                        if eval < smallest_eval {
                            smallest_eval = eval;
                        } else if eval > largest_eval {
                            largest_eval = eval;
                        }
                    }

                    if !swings.is_empty() {
                        notable_game.importance += swings.len() as i32 * 10;

                        if swings.len() == 1 {
                            let i = swings[0].1 - 1;
                            let from_eval = game.move_history[i - 1].eval;
                            let to_eval = game.move_history[i].eval;
                            let reason = NotableReason::LargeEvalSwing {
                                from_eval,
                                to_eval,
                                swing: swings[0].0,
                                ply: swings[0].1,
                            };
                            notable_game.reasons.push(reason);
                        } else {
                            let reason = NotableReason::MultipleEvalSwings {
                                count: swings.len(),
                                moves: swings,
                            };

                            notable_game.reasons.push(reason);
                        }
                    }

                    if end > start {
                        average_eval = total / (end - start) as i32;
                    }

                    if largest_eval - smallest_eval >= 250 {
                        // large eval difference in middle game
                        let reason = NotableReason::AbnormalEval;

                        notable_game.reasons.push(reason);
                        notable_game.importance += 3;
                    }

                    if average_eval != 0 && (average_eval - median_eval).abs() >= 150 {
                        // changing evals
                        let reason = NotableReason::AbnormalEval;

                        if !notable_game.reasons.contains(&reason) {
                            notable_game.reasons.push(reason);
                        }
                        notable_game.importance += 2;
                    }

                    // in plies
                    if game_length <= 50 {
                        let reason = NotableReason::ShortGame;

                        notable_game.reasons.push(reason);

                        match game_result {
                            GameResult::Draw => {
                                notable_game.importance += 2;
                            }
                            _ => {
                                notable_game.importance += 5;
                            }
                        }
                    }

                    // in plies
                    if game_length >= 140 {
                        let reason = NotableReason::LongGame;

                        notable_game.reasons.push(reason);
                        notable_game.importance += 2;
                    }

                    match game_result {
                        GameResult::White => {
                            // white player won
                            if engine_1_current_color == Color::White {
                                result.engine_1_wins += 1;
                                result.engine_1_wins_as_white += 1;

                                result.engine_2_losses += 1;
                                result.engine_2_losses_as_black += 1;

                                engine_1_total_time += game.white_time_elapsed;
                                engine_2_total_time += game.black_time_elapsed;
                            } else {
                                result.engine_2_wins += 1;
                                result.engine_2_wins_as_white += 1;

                                result.engine_1_losses += 1;
                                result.engine_1_losses_as_black += 1;

                                engine_1_total_time += game.black_time_elapsed;
                                engine_2_total_time += game.white_time_elapsed;
                            }

                            if expected_winner != Color::White {
                                // upset
                                let reason = NotableReason::Upset;

                                notable_game.reasons.push(reason);
                                notable_game.importance += 5;
                            }

                            if average_eval < -150 {
                                // comeback
                                let reason = NotableReason::Comeback { average_eval };

                                notable_game.reasons.push(reason);
                                notable_game.importance += 10;
                            }
                        }
                        GameResult::Black => {
                            // black won
                            if engine_1_current_color == Color::Black {
                                result.engine_1_wins += 1;
                                result.engine_1_wins_as_black += 1;

                                result.engine_2_losses += 1;
                                result.engine_2_losses_as_white += 1;

                                engine_1_total_time += game.black_time_elapsed;
                                engine_2_total_time += game.white_time_elapsed;
                            } else {
                                result.engine_2_wins += 1;
                                result.engine_2_wins_as_black += 1;

                                result.engine_1_losses += 1;
                                result.engine_1_losses_as_white += 1;

                                engine_1_total_time += game.white_time_elapsed;
                                engine_2_total_time += game.black_time_elapsed;
                            }

                            if expected_winner != Color::Black {
                                // upset
                                let reason = NotableReason::Upset;

                                notable_game.reasons.push(reason);
                                notable_game.importance += 5;
                            }

                            if average_eval > 150 {
                                // comeback
                                let reason = NotableReason::Comeback { average_eval };

                                notable_game.reasons.push(reason);
                                notable_game.importance += 10;
                            }
                        }
                        GameResult::Draw => {
                            // draw
                            if engine_1_current_color == Color::White {
                                result.engine_1_draws += 1;
                                result.engine_1_draws_as_white += 1;

                                result.engine_2_draws += 1;
                                result.engine_2_draws_as_black += 1;

                                engine_1_total_time += game.white_time_elapsed;
                                engine_2_total_time += game.black_time_elapsed;
                            } else {
                                result.engine_1_draws += 1;
                                result.engine_1_draws_as_black += 1;

                                result.engine_2_draws += 1;
                                result.engine_2_draws_as_white += 1;

                                engine_1_total_time += game.black_time_elapsed;
                                engine_2_total_time += game.white_time_elapsed;
                            }
                        }
                    }
                    if notable_game.importance > 0 {
                        result.notable_games.push(notable_game);
                    }
                }
                Err(msg) => {
                    result.invalid_games.push((None, msg));
                    players.swap();
                    engine_1_current_color = engine_1_current_color.opposite();
                    expected_winner = expected_winner.opposite();
                    count += 1;
                    continue;
                }
            }

            players.swap();
            engine_1_current_color = engine_1_current_color.opposite();
            expected_winner = expected_winner.opposite();
            count += 1;

            let elapsed = start.elapsed();

            println!(
                "Game: {}. Wall time: {:.2} seconds. Engine search time: {:.2} seconds",
                count - 1,
                elapsed.as_secs_f64(),
                search_elapsed.as_secs_f64(),
            );
        }
    }

    if result.valid_games > 0 {
        result.avg_engine_1_time_per_game = engine_1_total_time / result.valid_games as u32;

        result.avg_engine_2_time_per_game = engine_2_total_time / result.valid_games as u32;
    }

    result.total_time = engine_1_total_time + engine_2_total_time;

    result
}
