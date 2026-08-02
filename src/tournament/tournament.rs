use std::time::{Duration, Instant};

use crate::board::STARTPOS_FEN;
use crate::tournament::results::GameResult;
use crate::tournament::{MatchPlayers, NotableGame, NotableReason, TournamentResult, run_game};
use crate::types::Color;

pub fn play_games(
    num_games: usize,
    mut players: MatchPlayers, /* Later: OpeningSuite, Start_fen */
    expected: Color, // For now expected is the starting players color. For example white would be engine 1 or MatchPlayers.white
) -> TournamentResult {
    if num_games == 0 {
        panic!("Invalid num_games in play_games");
    }

    let mut result = TournamentResult::new(players.white.name.clone(), players.black.name.clone());

    let mut engine_1_current_color = Color::White;

    let mut engine_1_total_time: Duration = Duration::ZERO;
    let mut engine_2_total_time: Duration = Duration::ZERO;

    let mut expected_winner = expected; // for now this value will be swapped for easy checks

    // check the logic here as there could be a hidden bug but regardless times should ultimately be what total games becomes
    result.total_games = num_games;

    for i in 0..num_games {
        let start = Instant::now();
        match run_game(STARTPOS_FEN.to_string(), players.clone()) {
            Ok(g) => {
                let Some(game_result) = g.result.clone() else {
                    result
                        .invalid_games
                        .push((Some(g), "No valid result in game".to_string()));
                    players.swap();
                    engine_1_current_color = engine_1_current_color.opposite();
                    expected_winner = expected_winner.opposite();
                    continue;
                };

                let mut notable_game = NotableGame {
                    game_record: g.clone(),
                    reasons: Vec::new(),
                    importance: 0,
                };

                result.games.push(g.clone());
                result.valid_games += 1;

                let game = g;

                // eval is from whites perspective
                let mut smallest_eval = 0;
                let mut largest_eval = 0;

                let mut average_eval = 0;
                let mut total = 0;

                let game_length = game.move_history.len();

                let median_eval = game.move_history[game_length / 2].eval;

                // look relatively only at the middle game.
                // Later make this relative
                let start = ((game_length / 5) - 3).max(1);
                let end = ((game_length / 2) - 5).max(start + 5);

                let mut swings: Vec<(i32, usize)> = Vec::new();
                for i in start..end {
                    if i >= game_length {
                        break;
                    }
                    let previous_eval = game.move_history[i - 1].eval; // start will always be atleast 1
                    let eval = game.move_history[i].eval;

                    if (previous_eval - eval).abs() >= 150 {
                        if swings.len() == 0 {
                            let reason = NotableReason::LargeEvalSwing {
                                from_eval: previous_eval,
                                to_eval: eval,
                                swing: (eval - previous_eval).abs(),
                                ply: i + 1,
                            };

                            notable_game.reasons.push(reason);
                            notable_game.importance += 10;
                            swings.push(((eval - previous_eval).abs(), i + 1));
                        } else {
                            // since there are no previous ways to add reasons the only reason here would be the orginal large eval swing
                            // which we dont want to double count
                            notable_game.reasons.clear();
                            notable_game.importance = 0; // clear importance too. It will be reset afterwards anyways
                            swings.push(((eval - previous_eval).abs(), i + 1));
                        }
                    }

                    total += eval;

                    if eval < smallest_eval {
                        smallest_eval = eval;
                    } else if eval > largest_eval {
                        largest_eval = eval;
                    }
                }

                if !swings.is_empty() {
                    notable_game.importance += swings.len() as i32 * 7;

                    let reason = NotableReason::MultipleEvalSwings {
                        count: swings.len(),
                        moves: swings,
                    };

                    notable_game.reasons.push(reason);
                }

                if end > start {
                    average_eval = total / (end - start) as i32;
                }

                if (largest_eval - smallest_eval).abs() >= 250 {
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

                        if average_eval < -200 {
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

                        if average_eval > 200 {
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
                continue;
            }
        }

        players.swap();
        engine_1_current_color = engine_1_current_color.opposite();
        expected_winner = expected_winner.opposite();

        let elapsed = start.elapsed();

        println!(
            "Game: {}. Time: {:.2} seconds",
            i + 1,
            elapsed.as_secs_f64()
        );
    }
    if result.valid_games > 0 {
        result.avg_engine_1_time_per_game = engine_1_total_time / result.valid_games as u32;

        result.avg_engine_2_time_per_game = engine_2_total_time / result.valid_games as u32;
    }

    result.total_time = engine_1_total_time + engine_2_total_time;

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore]
    pub fn test_tournament_different_depth() {
        let mut match_players = MatchPlayers::from_depth(
            "Depth10-Qdepth-5".to_string(),
            "Depth9-Qdepth6".to_string(),
            10,
            5,
            9,
            6,
        );

        match_players.white.config.search.fut.enabled = false;
        match_players.black.config.search.fut.enabled = false;

        let result = play_games(10, match_players, Color::White);

        result.print_stats();
    }

    #[test]
    #[ignore]
    pub fn test_tournament_different_configs() {
        let mut match_players = MatchPlayers::from_depth(
            "Null Move on".to_string(),
            "Null Move off".to_string(),
            20,
            6,
            20,
            6,
        );

        match_players.white.config.limits.soft_time_limit_ms = Some(1000);
        match_players.black.config.limits.soft_time_limit_ms = Some(1000);

        match_players.white.config.search.fut.enabled = false;
        match_players.black.config.search.fut.enabled = false;

        match_players.white.config.search.delta.enabled = false;
        match_players.black.config.search.delta.enabled = false;

        // RFP is causing a missed mate issue
        // might be quiescence rewrite
        match_players.white.config.search.rfp.enabled = false;
        match_players.black.config.search.rfp.enabled = false;

        match_players.white.config.search.null_move.enabled = true;
        match_players.black.config.search.null_move.enabled = false;

        match_players.white.config.search.lmr.enabled = true;
        match_players.black.config.search.lmr.enabled = false;

        let result = play_games(200, match_players, Color::White);

        result.print_stats();
    }

    #[test]
    #[ignore]
    pub fn test_tournament_same_nodes() {
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

        let result = play_games(100, match_players, Color::White);

        result.print_stats();
    }
}
