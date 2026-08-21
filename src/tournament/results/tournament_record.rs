use std::{
    cmp::Reverse,
    io::{self, Write},
    time::Duration,
};

use crate::uci::pgn::{PgnMetadata, game_to_pgn};
use crate::{
    tournament::{GameRecord, NotableGame},
    types::Color,
};

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

                // if i == 0 {
                //     notable_game
                //         .game_record
                //         .white_stats
                //         .print_all(1, notable_game.game_record.white_time_elapsed.as_secs_f64());

                //     println!();

                //     notable_game
                //         .game_record
                //         .black_stats
                //         .print_all(1, notable_game.game_record.black_time_elapsed.as_secs_f64());

                //     println!();
                // }

                // TODO: This currently puts engine 1 name as white despite the fact that it may have been black. Need to fix this to reflect the actual color played by each engine.
                let white = if notable_game.game_record.engine_1_color
                    == Some(crate::types::Color::White)
                {
                    &self.engine_1_name
                } else {
                    &self.engine_2_name
                };

                let black = if notable_game.game_record.engine_1_color
                    == Some(crate::types::Color::Black)
                {
                    &self.engine_1_name
                } else {
                    &self.engine_2_name
                };

                let metadata = PgnMetadata {
                    event: "Engine Test".to_string(),
                    site: "Local".to_string(),
                    date: "2026.07.27".to_string(),
                    round: i.to_string(),
                    white: white.to_string(),
                    black: black.to_string(),
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

const GAME_LIST_PAGE_SIZE: usize = 10;
const DEFAULT_NOTABLE_LIMIT: usize = 10;
const DEFAULT_MOVE_LIST_COUNT: usize = 10;

/// Opens a compact command-line viewer after a tournament has completed.
///
/// The viewer owns the result so it can freely keep indices and references into
/// the stored game collections without affecting the tournament runner.
pub fn review_tournament(tournament: TournamentResult) -> io::Result<()> {
    TournamentViewer::new(tournament).run()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum GameSelection {
    Game(usize),
    Notable(usize),
}

pub struct TournamentViewer {
    tournament: TournamentResult,
    selection: Option<GameSelection>,

    /// Zero means the starting position. A value of N means N plies have been
    /// played and move_history[N - 1] is the currently displayed move.
    ply_cursor: usize,

    /// Indices into tournament.notable_games, sorted from highest to lowest
    /// importance. This leaves the original storage order unchanged.
    notable_order: Vec<usize>,
}

impl TournamentViewer {
    pub fn new(tournament: TournamentResult) -> Self {
        let mut notable_order: Vec<usize> = (0..tournament.notable_games.len()).collect();

        notable_order.sort_by_key(|&index| Reverse(tournament.notable_games[index].importance));

        let selection = if !tournament.games.is_empty() {
            Some(GameSelection::Game(0))
        } else {
            notable_order.first().copied().map(GameSelection::Notable)
        };

        Self {
            tournament,
            selection,
            ply_cursor: 0,
            notable_order,
        }
    }

    pub fn run(mut self) -> io::Result<()> {
        println!();
        println!("════════════════════════════════════════════════════");
        println!("              TOURNAMENT GAME VIEWER");
        println!("════════════════════════════════════════════════════");
        self.tournament.print_compact_summary();
        println!("Type 'help' to show the available commands.");

        if self.selection.is_some() {
            println!();
            self.print_current_game_summary();
            self.print_current_move();
        } else {
            println!("No valid or notable game records are available.");
        }

        let stdin = io::stdin();
        let mut input = String::new();

        loop {
            print!("\nreview> ");
            io::stdout().flush()?;

            input.clear();
            if stdin.read_line(&mut input)? == 0 {
                println!();
                break;
            }

            let mut parts = input.split_whitespace();
            let Some(command) = parts.next() else {
                continue;
            };

            match command.to_ascii_lowercase().as_str() {
                "help" => Self::print_help(),
                "tournament" => self.tournament.print_compact_summary(),
                "elo" => {
                    let side = parts.next().unwrap_or("white");
                    self.tournament.print_elo(side);
                }
                "games" => match parse_optional_usize(parts.next(), 1, "page") {
                    Ok(page) => self.print_games_page(page),
                    Err(message) => println!("{message}"),
                },
                "notables" => {
                    match parse_optional_usize(parts.next(), DEFAULT_NOTABLE_LIMIT, "limit") {
                        Ok(limit) => self.print_notable_games(limit),
                        Err(message) => println!("{message}"),
                    }
                }
                "game" => match parse_required_one_based(parts.next(), "game number") {
                    Ok(number) => self.select_game(number),
                    Err(message) => println!("{message}"),
                },
                "notable" => match parse_required_one_based(parts.next(), "notable rank") {
                    Ok(rank) => self.select_notable(rank),
                    Err(message) => println!("{message}"),
                },
                "next-game" => self.next_game(),
                "previous-game" => self.previous_game(),
                "next" => self.next_move(),
                "previous" => self.previous_move(),
                "move" => match parse_required_usize(parts.next(), "ply number") {
                    Ok(ply) => self.select_ply(ply),
                    Err(message) => println!("{message}"),
                },
                "moves" => {
                    let start = parse_optional_usize(parts.next(), 1, "start ply");
                    let count =
                        parse_optional_usize(parts.next(), DEFAULT_MOVE_LIST_COUNT, "move count");

                    match (start, count) {
                        (Ok(start), Ok(count)) => self.print_move_range(start, count),
                        (Err(message), _) | (_, Err(message)) => println!("{message}"),
                    }
                }
                "current" => {
                    self.print_current_game_summary();
                    self.print_current_move();
                }
                "stats" => {
                    let side = parts.next().unwrap_or("both");
                    self.print_current_game_stats(side);
                }
                "pgn" => self.print_current_game_pgn(),
                "quit" | "exit" => break,
                _ => {
                    println!("Unknown command '{command}'. Type 'help' for commands.");
                }
            }
        }

        Ok(())
    }

    fn print_help() {
        println!();
        println!("Commands");
        println!("  tournament               Print compact tournament totals");
        println!("  elo [color]              Print elo stats for the engines original color");
        println!("  games [page]             List 10 regular games on a page");
        println!("  notables [limit]         List notable games by importance");
        println!("  game <number>            Select a regular game");
        println!("  notable <rank>           Select a ranked notable game");
        println!("  next-game                Select the next game in the current list");
        println!("  previous-game            Select the previous game in the current list");
        println!("  next                     Advance one ply");
        println!("  previous                 Go back one ply");
        println!("  move <ply>               Jump to a ply; use 0 for the start");
        println!("  moves [start] [count]    Print a compact move range");
        println!("  current                  Print the current game and move");
        println!("  stats [white|black|both] Print aggregate game search stats");
        println!("  pgn                      Print the current game as PGN");
        println!("  help                     Print this command list");
        println!("  quit                     Close the viewer");
    }

    fn current_game(&self) -> Option<&GameRecord> {
        match self.selection? {
            GameSelection::Game(index) => self.tournament.games.get(index),
            GameSelection::Notable(index) => self
                .tournament
                .notable_games
                .get(index)
                .map(|notable| &notable.game_record),
        }
    }

    fn select_game(&mut self, one_based_number: usize) {
        let Some(index) = one_based_number.checked_sub(1) else {
            println!("Game numbers begin at 1.");
            return;
        };

        if index >= self.tournament.games.len() {
            println!(
                "Game {one_based_number} does not exist. Stored games: {}.",
                self.tournament.games.len()
            );
            return;
        }

        self.selection = Some(GameSelection::Game(index));
        self.ply_cursor = 0;
        self.print_current_game_summary();
        self.print_current_move();
    }

    fn select_notable(&mut self, one_based_rank: usize) {
        let Some(order_index) = one_based_rank.checked_sub(1) else {
            println!("Notable ranks begin at 1.");
            return;
        };

        let Some(&notable_index) = self.notable_order.get(order_index) else {
            println!(
                "Notable rank {one_based_rank} does not exist. Stored notable games: {}.",
                self.notable_order.len()
            );
            return;
        };

        self.selection = Some(GameSelection::Notable(notable_index));
        self.ply_cursor = 0;
        self.print_current_game_summary();
        self.print_current_move();
    }

    fn next_game(&mut self) {
        match self.selection {
            Some(GameSelection::Game(index)) => {
                if index + 1 >= self.tournament.games.len() {
                    println!("Already at the final regular game.");
                    return;
                }
                self.selection = Some(GameSelection::Game(index + 1));
            }
            Some(GameSelection::Notable(index)) => {
                let Some(rank) = self.notable_rank(index) else {
                    println!("The current notable game is not in the ranking.");
                    return;
                };

                if rank + 1 >= self.notable_order.len() {
                    println!("Already at the final notable game.");
                    return;
                }

                self.selection = Some(GameSelection::Notable(self.notable_order[rank + 1]));
            }
            None => {
                println!("No game is selected.");
                return;
            }
        }

        self.ply_cursor = 0;
        self.print_current_game_summary();
        self.print_current_move();
    }

    fn previous_game(&mut self) {
        match self.selection {
            Some(GameSelection::Game(index)) => {
                if index == 0 {
                    println!("Already at the first regular game.");
                    return;
                }
                self.selection = Some(GameSelection::Game(index - 1));
            }
            Some(GameSelection::Notable(index)) => {
                let Some(rank) = self.notable_rank(index) else {
                    println!("The current notable game is not in the ranking.");
                    return;
                };

                if rank == 0 {
                    println!("Already at the first notable game.");
                    return;
                }

                self.selection = Some(GameSelection::Notable(self.notable_order[rank - 1]));
            }
            None => {
                println!("No game is selected.");
                return;
            }
        }

        self.ply_cursor = 0;
        self.print_current_game_summary();
        self.print_current_move();
    }

    fn next_move(&mut self) {
        let Some(move_count) = self.current_game().map(|game| game.move_history.len()) else {
            println!("No game is selected.");
            return;
        };

        if self.ply_cursor >= move_count {
            println!("Already at the end of the game ({move_count} plies).");
            return;
        }

        self.ply_cursor += 1;
        self.print_current_move();
    }

    fn previous_move(&mut self) {
        if self.current_game().is_none() {
            println!("No game is selected.");
            return;
        }

        if self.ply_cursor == 0 {
            println!("Already at the starting position.");
            return;
        }

        self.ply_cursor -= 1;
        self.print_current_move();
    }

    fn select_ply(&mut self, ply: usize) {
        let Some(move_count) = self.current_game().map(|game| game.move_history.len()) else {
            println!("No game is selected.");
            return;
        };

        if ply > move_count {
            println!("Ply {ply} does not exist. Valid range: 0..={move_count}.");
            return;
        }

        self.ply_cursor = ply;
        self.print_current_move();
    }

    fn print_current_move(&self) {
        let Some(game) = self.current_game() else {
            println!("No game is selected.");
            return;
        };

        let move_count = game.move_history.len();

        if self.ply_cursor == 0 {
            println!("Position: start of game | 0/{move_count} plies");
            println!("Start FEN: {}", game.start_fen);

            if let Some(next) = game.move_history.first() {
                println!(
                    "Next: {} | stored eval {:+} cp",
                    format_move_label(0, &next.mv),
                    next.eval
                );
            }
            return;
        }

        let move_index = self.ply_cursor - 1;
        let Some(record) = game.move_history.get(move_index) else {
            println!("The current move cursor is invalid.");
            return;
        };

        println!(
            "Ply {:>3}/{:<3} | {:<18} | stored eval {:+} cp ({:+.2})",
            self.ply_cursor,
            move_count,
            format_move_label(move_index, &record.mv),
            record.eval,
            record.eval as f64 / 100.0,
        );

        if self.ply_cursor == move_count {
            println!("End of game.");
        }
    }

    fn print_move_range(&self, one_based_start: usize, count: usize) {
        let Some(game) = self.current_game() else {
            println!("No game is selected.");
            return;
        };

        if game.move_history.is_empty() {
            println!("The selected game has no stored moves.");
            return;
        }

        if count == 0 {
            println!("Move count must be greater than zero.");
            return;
        }

        let Some(start_index) = one_based_start.checked_sub(1) else {
            println!("Move ranges begin at ply 1.");
            return;
        };

        if start_index >= game.move_history.len() {
            println!(
                "Start ply {one_based_start} does not exist. Stored plies: {}.",
                game.move_history.len()
            );
            return;
        }

        let end_index = start_index
            .saturating_add(count)
            .min(game.move_history.len());

        println!(
            "Moves {}-{} of {}",
            start_index + 1,
            end_index,
            game.move_history.len()
        );

        for (index, record) in game.move_history[start_index..end_index].iter().enumerate() {
            let absolute_index = start_index + index;
            let cursor = if self.ply_cursor == absolute_index + 1 {
                ">"
            } else {
                " "
            };

            println!(
                "{cursor} {:>3}. {:<18} eval {:+} cp",
                absolute_index + 1,
                format_move_label(absolute_index, &record.mv),
                record.eval,
            );
        }
    }

    fn print_games_page(&self, one_based_page: usize) {
        if self.tournament.games.is_empty() {
            println!("No regular games are stored.");
            return;
        }

        let page_count =
            (self.tournament.games.len() + GAME_LIST_PAGE_SIZE - 1) / GAME_LIST_PAGE_SIZE;

        let Some(page_index) = one_based_page.checked_sub(1) else {
            println!("Pages begin at 1.");
            return;
        };

        if page_index >= page_count {
            println!("Page {one_based_page} does not exist. Last page: {page_count}.");
            return;
        }

        let start = page_index * GAME_LIST_PAGE_SIZE;
        let end = (start + GAME_LIST_PAGE_SIZE).min(self.tournament.games.len());

        println!();
        println!(
            "Games — page {one_based_page}/{page_count} ({} total)",
            self.tournament.games.len()
        );

        for (index, game) in self.tournament.games[start..end].iter().enumerate() {
            let absolute_index = start + index;
            let selected = if self.selection == Some(GameSelection::Game(absolute_index)) {
                ">"
            } else {
                " "
            };
            let (white, black) = self.tournament.engine_names_for_game(game);
            let opening = truncate_text(
                game.opening_name.as_deref().unwrap_or("Unknown opening"),
                24,
            );

            println!(
                "{selected} {:>4}. {:<24} {:>4} plies | W: {:<18} B: {:<18} | {:>7.3}s",
                absolute_index + 1,
                opening,
                game.move_history.len(),
                truncate_text(white, 18),
                truncate_text(black, 18),
                game.total_time.as_secs_f64(),
            );
        }
    }

    fn print_notable_games(&self, limit: usize) {
        if self.notable_order.is_empty() {
            println!("No notable games are stored.");
            return;
        }

        if limit == 0 {
            println!("Notable limit must be greater than zero.");
            return;
        }

        let shown = limit.min(self.notable_order.len());

        println!();
        println!(
            "Notable Games — showing {shown}/{} by importance",
            self.notable_order.len()
        );

        for (rank, &notable_index) in self.notable_order.iter().take(shown).enumerate() {
            let notable = &self.tournament.notable_games[notable_index];

            if self.selection == Some(GameSelection::Notable(notable_index)) {
                println!("> Current notable selection");
            }

            // Reuse the existing reason-specific summary, but omit the PGN and
            // full search-stat dump from the legacy print_stats() path.
            notable.print_summary(rank + 1);
        }
    }

    fn print_current_game_summary(&self) {
        let Some(game) = self.current_game() else {
            println!("No game is selected.");
            return;
        };

        let selection_name = match self.selection {
            Some(GameSelection::Game(index)) => format!(
                "Regular game {} of {}",
                index + 1,
                self.tournament.games.len()
            ),
            Some(GameSelection::Notable(index)) => {
                let rank = self.notable_rank(index).map(|rank| rank + 1).unwrap_or(0);
                let importance = self.tournament.notable_games[index].importance;
                format!(
                    "Notable game {rank} of {} | importance {importance}",
                    self.notable_order.len()
                )
            }
            None => "No game selected".to_string(),
        };

        let (white, black) = self.tournament.engine_names_for_game(game);

        println!();
        println!("────────────────────────────────────────────────────");
        println!("{selection_name}");
        println!(
            "Opening: {}",
            game.opening_name.as_deref().unwrap_or("Unknown")
        );
        println!("White: {white}");
        println!("Black: {black}");
        println!(
            "Moves: {} plies ({} full moves) | Result stored: {}",
            game.move_history.len(),
            (game.move_history.len() + 1) / 2,
            if game.result.is_some() { "Yes" } else { "No" },
        );
        println!(
            "Time: total {:.3}s | White {:.3}s | Black {:.3}s",
            game.total_time.as_secs_f64(),
            game.white_time_elapsed.as_secs_f64(),
            game.black_time_elapsed.as_secs_f64(),
        );
    }

    fn print_current_game_stats(&self, side: &str) {
        let Some(game) = self.current_game() else {
            println!("No game is selected.");
            return;
        };

        let (white_name, black_name) = self.tournament.engine_names_for_game(game);

        println!();
        println!("════════════════ GAME SEARCH STATS ════════════════");

        match side.to_ascii_lowercase().as_str() {
            "white" => {
                println!("White — {white_name}");
                println!(
                    "  Elapsed: {:.3}s | Average move time: {:.3}s",
                    game.white_time_elapsed.as_secs_f64(),
                    game.white_avg_time.as_secs_f64(),
                );
                game.white_stats
                    .print_all(1, game.white_time_elapsed.as_secs_f64());
            }
            "black" => {
                println!("Black — {black_name}");
                println!(
                    "  Elapsed: {:.3}s | Average move time: {:.3}s",
                    game.black_time_elapsed.as_secs_f64(),
                    game.black_avg_time.as_secs_f64(),
                );
                game.black_stats
                    .print_all(1, game.black_time_elapsed.as_secs_f64());
            }
            "both" => {
                println!("White — {white_name}");
                println!(
                    "  Elapsed: {:.3}s | Average move time: {:.3}s",
                    game.white_time_elapsed.as_secs_f64(),
                    game.white_avg_time.as_secs_f64(),
                );
                game.white_stats
                    .print_all(1, game.white_time_elapsed.as_secs_f64());

                println!();
                println!("Black — {black_name}");
                println!(
                    "  Elapsed: {:.3}s | Average move time: {:.3}s",
                    game.black_time_elapsed.as_secs_f64(),
                    game.black_avg_time.as_secs_f64(),
                );
                game.black_stats
                    .print_all(1, game.black_time_elapsed.as_secs_f64());
            }
            _ => println!("Usage: stats [white|black|both]"),
        }
    }

    fn print_current_game_pgn(&self) {
        let Some(game) = self.current_game() else {
            println!("No game is selected.");
            return;
        };

        let (white, black) = self.tournament.engine_names_for_game(game);
        let round = match self.selection {
            Some(GameSelection::Game(index)) => (index + 1).to_string(),
            Some(GameSelection::Notable(index)) => self
                .notable_rank(index)
                .map(|rank| format!("N{}", rank + 1))
                .unwrap_or_else(|| "N?".to_string()),
            None => "?".to_string(),
        };

        let metadata = PgnMetadata {
            event: "Engine Test".to_string(),
            site: "Local".to_string(),
            date: "????.??.??".to_string(),
            round,
            white: white.to_string(),
            black: black.to_string(),
        };

        match game_to_pgn(&game.game, &metadata) {
            Ok(pgn) => {
                println!();
                println!("{pgn}");
            }
            Err(error) => println!("Unable to create PGN: {:?}", error),
        }
    }

    fn notable_rank(&self, notable_index: usize) -> Option<usize> {
        self.notable_order
            .iter()
            .position(|&index| index == notable_index)
    }
}

impl TournamentResult {
    /// Convenience method when the result is no longer needed after review.
    pub fn review(self) -> io::Result<()> {
        review_tournament(self)
    }

    /// A compact tournament-only summary. Unlike print_stats(), this does not
    /// print invalid-game details, notable-game PGNs, or per-game search stats.
    pub fn print_compact_summary(&self) {
        let engine_1_score = self.engine_1_wins as f64 + self.engine_1_draws as f64 * 0.5;
        let engine_2_score = self.engine_2_wins as f64 + self.engine_2_draws as f64 * 0.5;

        println!();
        println!("Tournament Summary");
        println!(
            "  Games: {} requested | {} valid | {} invalid | {} notable",
            self.total_games,
            self.valid_games,
            self.invalid_games.len(),
            self.notable_games.len(),
        );

        if self.valid_games > 0 {
            println!(
                "  {}: {:.1}/{} ({:.2}%) | W-L-D {}-{}-{} | avg {:.3}s",
                self.engine_1_name,
                engine_1_score,
                self.valid_games,
                engine_1_score / self.valid_games as f64 * 100.0,
                self.engine_1_wins,
                self.engine_1_losses,
                self.engine_1_draws,
                self.avg_engine_1_time_per_game.as_secs_f64(),
            );
            println!(
                "  {}: {:.1}/{} ({:.2}%) | W-L-D {}-{}-{} | avg {:.3}s",
                self.engine_2_name,
                engine_2_score,
                self.valid_games,
                engine_2_score / self.valid_games as f64 * 100.0,
                self.engine_2_wins,
                self.engine_2_losses,
                self.engine_2_draws,
                self.avg_engine_2_time_per_game.as_secs_f64(),
            );
        }

        println!(
            "  Total tournament time: {:.3}s",
            self.total_time.as_secs_f64()
        );
    }

    pub fn print_elo(&self, color: &str) {
        let side = match color.to_ascii_lowercase().as_str() {
            "black" => Color::Black,
            _ => Color::White,
        };

        if let Some(elo) = self.elo_stats(side) {
            println!();
            println!("Elo Estimate");
            println!("────────────────────────────────────────────");
            println!(
                "  {} score:                 {:>7.2}%",
                self.engine_1_name,
                elo.score_rate * 100.0
            );
            println!("  Elo difference:             {:+7.1}", elo.elo_difference);
            println!(
                "  95% Elo CI:           [{:+.1}, {:+.1}]",
                elo.elo_ci_low, elo.elo_ci_high
            );
            println!(
                "  Score std. error:            {:>6.2}%",
                elo.score_standard_error * 100.0
            );
            println!(
                "  LOS:                         {:>6.2}%",
                elo.likelihood_of_superiority * 100.0
            );
            println!(
                "  Draw rate:                   {:>6.2}%",
                elo.draw_rate * 100.0
            );
            println!(
                "  Decisive rate:               {:>6.2}%",
                elo.decisive_rate * 100.0
            );
        }
    }

    fn engine_names_for_game<'a>(&'a self, game: &GameRecord) -> (&'a str, &'a str) {
        match game.engine_1_color {
            Some(crate::types::Color::White) => (&self.engine_1_name, &self.engine_2_name),
            Some(crate::types::Color::Black) => (&self.engine_2_name, &self.engine_1_name),
            None => ("Unknown white engine", "Unknown black engine"),
        }
    }
}

fn format_move_label(movement_index: usize, mv: impl std::fmt::Display) -> String {
    let move_number = movement_index / 2 + 1;
    if movement_index % 2 == 0 {
        format!("{move_number}. {mv}")
    } else {
        format!("{move_number}... {mv}")
    }
}

fn truncate_text(text: &str, max_chars: usize) -> String {
    let mut chars = text.chars();
    let prefix: String = chars.by_ref().take(max_chars).collect();

    if chars.next().is_some() && max_chars > 0 {
        let mut shortened: String = prefix.chars().take(max_chars.saturating_sub(1)).collect();
        shortened.push('…');
        shortened
    } else {
        prefix
    }
}

fn parse_required_usize(value: Option<&str>, name: &str) -> Result<usize, String> {
    let Some(value) = value else {
        return Err(format!("Missing {name}."));
    };

    value
        .parse::<usize>()
        .map_err(|_| format!("Invalid {name} '{value}'."))
}

fn parse_required_one_based(value: Option<&str>, name: &str) -> Result<usize, String> {
    let number = parse_required_usize(value, name)?;
    if number == 0 {
        Err(format!("{name} begins at 1."))
    } else {
        Ok(number)
    }
}

fn parse_optional_usize(value: Option<&str>, default: usize, name: &str) -> Result<usize, String> {
    match value {
        Some(value) => value
            .parse::<usize>()
            .map_err(|_| format!("Invalid {name} '{value}'.")),
        None => Ok(default),
    }
}
