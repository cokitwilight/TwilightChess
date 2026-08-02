use std::time::Duration;

use crate::tournament::{GameRecord, NotableGame};
use crate::uci::pgn::{PgnMetadata, game_to_pgn};

pub struct TournamentResult {
    // since the engines will swap between black and white keep track of engine wins not color
    pub engine_1_name: String, // this will be what feature is listed as. For example WithNullMovePruning

    pub engine_1_wins: usize,
    pub engine_1_wins_as_white: usize,
    pub engine_1_wins_as_black: usize,

    pub engine_1_losses: usize,
    pub engine_1_losses_as_white: usize,
    pub engine_1_losses_as_black: usize,

    pub engine_1_draws: usize,
    pub engine_1_draws_as_white: usize,
    pub engine_1_draws_as_black: usize,

    pub engine_2_name: String,

    pub engine_2_wins: usize,
    pub engine_2_wins_as_white: usize,
    pub engine_2_wins_as_black: usize,

    pub engine_2_losses: usize,
    pub engine_2_losses_as_white: usize,
    pub engine_2_losses_as_black: usize,

    pub engine_2_draws: usize,
    pub engine_2_draws_as_white: usize,
    pub engine_2_draws_as_black: usize,

    pub avg_engine_1_time_per_game: Duration,
    pub avg_engine_2_time_per_game: Duration,

    pub total_time: Duration,

    pub total_games: usize,
    pub valid_games: usize,

    pub invalid_games: Vec<(Option<GameRecord>, String)>, // to keep track of any potential crashes. Tuple of Option<GameRecord> and Error Message

    pub notable_games: Vec<NotableGame>, // maybe keep track of notable large eval changes or large move numbers

    pub games: Vec<GameRecord>, // all game records
}

impl TournamentResult {
    pub fn new(player_1: String, player_2: String) -> Self {
        Self {
            engine_1_name: player_1,

            engine_1_wins: 0,
            engine_1_wins_as_white: 0,
            engine_1_wins_as_black: 0,

            engine_1_losses: 0,
            engine_1_losses_as_white: 0,
            engine_1_losses_as_black: 0,

            engine_1_draws: 0,
            engine_1_draws_as_white: 0,
            engine_1_draws_as_black: 0,

            engine_2_name: player_2,

            engine_2_wins: 0,
            engine_2_wins_as_white: 0,
            engine_2_wins_as_black: 0,

            engine_2_losses: 0,
            engine_2_losses_as_white: 0,
            engine_2_losses_as_black: 0,

            engine_2_draws: 0,
            engine_2_draws_as_white: 0,
            engine_2_draws_as_black: 0,

            avg_engine_1_time_per_game: Duration::ZERO,
            avg_engine_2_time_per_game: Duration::ZERO,

            total_time: Duration::ZERO,

            total_games: 0,
            valid_games: 0,

            invalid_games: Vec::new(),

            notable_games: Vec::new(),

            games: Vec::new(),
        }
    }
    pub fn print_stats(&self) {
        println!();
        println!("════════════════════════════════════════════════════");
        println!("                  TOURNAMENT RESULT");
        println!("════════════════════════════════════════════════════");

        println!("Games");
        println!("  Requested:                    {:>12}", self.total_games);
        println!("  Valid:                        {:>12}", self.valid_games);
        println!(
            "  Invalid:                      {:>12}",
            self.invalid_games.len()
        );
        println!(
            "  Notable:                      {:>12}",
            self.notable_games.len()
        );

        println!();
        println!("{}", self.engine_1_name);
        println!("  Wins:                         {:>12}", self.engine_1_wins);
        println!(
            "    As White:                   {:>12}",
            self.engine_1_wins_as_white
        );
        println!(
            "    As Black:                   {:>12}",
            self.engine_1_wins_as_black
        );

        println!();
        println!(
            "  Losses:                       {:>12}",
            self.engine_1_losses
        );
        println!(
            "    As White:                   {:>12}",
            self.engine_1_losses_as_white
        );
        println!(
            "    As Black:                   {:>12}",
            self.engine_1_losses_as_black
        );

        println!();
        println!(
            "  Draws:                        {:>12}",
            self.engine_1_draws
        );
        println!(
            "    As White:                   {:>12}",
            self.engine_1_draws_as_white
        );
        println!(
            "    As Black:                   {:>12}",
            self.engine_1_draws_as_black
        );

        println!(
            "  Average time / valid game:    {:>12.3?}",
            self.avg_engine_1_time_per_game
        );

        println!();
        println!("{}", self.engine_2_name);
        println!("  Wins:                         {:>12}", self.engine_2_wins);
        println!(
            "    As White:                   {:>12}",
            self.engine_2_wins_as_white
        );
        println!(
            "    As Black:                   {:>12}",
            self.engine_2_wins_as_black
        );

        println!();
        println!(
            "  Losses:                       {:>12}",
            self.engine_2_losses
        );
        println!(
            "    As White:                   {:>12}",
            self.engine_2_losses_as_white
        );
        println!(
            "    As Black:                   {:>12}",
            self.engine_2_losses_as_black
        );

        println!();
        println!(
            "  Draws:                        {:>12}",
            self.engine_2_draws
        );
        println!(
            "    As White:                   {:>12}",
            self.engine_2_draws_as_white
        );
        println!(
            "    As Black:                   {:>12}",
            self.engine_2_draws_as_black
        );

        println!(
            "  Average time / valid game:    {:>12.3?}",
            self.avg_engine_2_time_per_game
        );

        println!();
        println!("Total Tournament Time:    {:>12.3?}", self.total_time);

        if self.valid_games > 0 {
            let engine_1_score = self.engine_1_wins as f64 + self.engine_1_draws as f64 * 0.5;
            let engine_2_score = self.engine_2_wins as f64 + self.engine_2_draws as f64 * 0.5;

            println!();
            println!("Scores");
            println!(
                "  {}: {:>8.1} / {} ({:>6.2}%)",
                self.engine_1_name,
                engine_1_score,
                self.valid_games,
                engine_1_score / self.valid_games as f64 * 100.0,
            );
            println!(
                "  {}: {:>8.1} / {} ({:>6.2}%)",
                self.engine_2_name,
                engine_2_score,
                self.valid_games,
                engine_2_score / self.valid_games as f64 * 100.0,
            );
        }

        if !self.invalid_games.is_empty() {
            println!();
            println!("Invalid Games");

            for (index, (game, error)) in self.invalid_games.iter().enumerate() {
                println!(
                    "  {:>4}. Record: {:<3} Error: {}",
                    index + 1,
                    if game.is_some() { "Yes" } else { "No" },
                    error,
                );
            }
        }

        if !self.notable_games.is_empty() {
            println!();
            println!("Notable Games");
            println!(
                "  {} notable game record(s) stored.",
                self.notable_games.len()
            );

            let mut notable_games = self.notable_games.clone();

            notable_games.sort_by_key(|n| n.importance);

            notable_games.reverse();

            // for now only list the first 10 notable games
            for i in 0..10 {
                println!();
                if i >= notable_games.len() {
                    break;
                }
                let notable_game = &notable_games[i];
                notable_game.print_summary(i + 1);

                if i == 0 {
                    notable_game
                        .game_record
                        .white_stats
                        .print_all(1, notable_game.game_record.white_time_elapsed.as_secs_f64());

                    println!();

                    notable_game
                        .game_record
                        .black_stats
                        .print_all(1, notable_game.game_record.black_time_elapsed.as_secs_f64());

                    println!();
                }

                let metadata = PgnMetadata {
                    event: "Engine Test".to_string(),
                    site: "Local".to_string(),
                    date: "2026.07.27".to_string(),
                    round: i.to_string(),
                    white: self.engine_1_name.clone(),
                    black: self.engine_2_name.clone(),
                };

                let pgn_text =
                    game_to_pgn(&notable_game.game_record.game, &metadata).expect("Game Failed");
                println!();

                println!("{pgn_text}");
            }
        }

        println!("════════════════════════════════════════════════════");
        println!();
    }
}
