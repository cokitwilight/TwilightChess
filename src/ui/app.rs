use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::thread;
use std::time::Duration;

use crate::bitboard::{Square, square_to_algebraic};
use crate::board::{Move, MoveList};
use crate::bot::Bot;
use crate::engine::configs::EngineConfig;
use crate::engine::{Engine, SearchLimits, SearchResult, SearchStats, SearchTermination};
use crate::eval::debug::evaluation_breakdown;
use crate::game::{Game, GameState};
use crate::types::{Color, PieceType};
use crate::uci::{PgnMetadata, game_to_pgn};
use crate::ui::{
    BoardAction, BotSearchRequest, BotSearchResponse, PromotionPicker, draw_board_sized,
};

use eframe::egui;

const DEFAULT_BOT_DEPTH: u16 = 25;
const DEFAULT_Q_BOT_DEPTH: u16 = 4;

const DEFAULT_BOT_TIME_MS: u64 = 2_000;

const OPENING_PLY_LIMIT: usize = 20;
const ENDGAME_PHASE_MAX: i32 = 12;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoardOrientation {
    WhiteBottom,
    BlackBottom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    MainMenu,
    BotSetup,
    Game,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GameScreenAction {
    None,
    PrintEval,
    TogglePgn,
    PrintSearchStats,
    NewGame,
    MainMenu,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppMode {
    PlayerVsPlayer,
    PlayerVsBot { human: Color },
}

#[derive(Clone, Debug)]
struct PendingPromotion {
    to: Square,
    color: Color,
    moves: Vec<Move>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum SearchStatsReport {
    #[default]
    Total,
    Opening,
    Middlegame,
    Endgame,
}

impl SearchStatsReport {
    fn label(self) -> &'static str {
        match self {
            Self::Total => "Total",
            Self::Opening => "Opening",
            Self::Middlegame => "Middlegame",
            Self::Endgame => "Endgame",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum GamePhase {
    #[default]
    Opening,
    Middlegame,
    Endgame,
}

#[derive(Clone, Debug, Default)]
struct SearchStatsAggregate {
    stats: SearchStats,
    elapsed: Duration,
    move_count: usize,
}

impl SearchStatsAggregate {
    fn record(&mut self, result: &SearchResult) {
        self.stats += result.stats;
        self.elapsed += result.elapsed;

        if result.best_move.is_some() {
            self.move_count += 1;
        }
    }

    fn print_report(&self, report: SearchStatsReport, game_ply_count: usize) {
        let full_move_count = game_ply_count.div_ceil(2);
        let label = report.label();

        println!();
        println!("{label} Bot Search Statistics");
        println!("Bot moves in report: {}", self.move_count);
        println!("Completed game length: {game_ply_count} plies ({full_move_count} full moves)");
        println!("Search time in report: {:.3}s", self.elapsed.as_secs_f64());
        println!(
            "Phase rules: opening = first {} full moves or book; endgame = phase <= {}; middlegame = between",
            OPENING_PLY_LIMIT / 2,
            ENDGAME_PHASE_MAX,
        );

        self.stats.print_all_with_title(
            &format!("{label} Search Counters"),
            self.elapsed.as_secs_f64(),
        );
    }
}

/// Aggregated engine-search data for one UI game against the bot.
#[derive(Clone, Debug, Default)]
struct BotGameSearchStats {
    total: SearchStatsAggregate,
    opening: SearchStatsAggregate,
    middlegame: SearchStatsAggregate,
    endgame: SearchStatsAggregate,
    current_phase: GamePhase,
}

impl BotGameSearchStats {
    fn record(&mut self, game_ply_count: usize, board_phase: i32, result: &SearchResult) {
        let phase = self.phase_for_search(game_ply_count, board_phase, result.termination);
        self.current_phase = phase;

        self.total.record(result);
        match phase {
            GamePhase::Opening => self.opening.record(result),
            GamePhase::Middlegame => self.middlegame.record(result),
            GamePhase::Endgame => self.endgame.record(result),
        }
    }

    fn phase_for_search(
        &self,
        game_ply_count: usize,
        board_phase: i32,
        termination: SearchTermination,
    ) -> GamePhase {
        match self.current_phase {
            GamePhase::Opening
                if termination == SearchTermination::BookMove
                    || game_ply_count < OPENING_PLY_LIMIT =>
            {
                GamePhase::Opening
            }
            GamePhase::Opening | GamePhase::Middlegame if board_phase <= ENDGAME_PHASE_MAX => {
                GamePhase::Endgame
            }
            GamePhase::Opening | GamePhase::Middlegame => GamePhase::Middlegame,
            GamePhase::Endgame => GamePhase::Endgame,
        }
    }

    fn aggregate(&self, report: SearchStatsReport) -> &SearchStatsAggregate {
        match report {
            SearchStatsReport::Total => &self.total,
            SearchStatsReport::Opening => &self.opening,
            SearchStatsReport::Middlegame => &self.middlegame,
            SearchStatsReport::Endgame => &self.endgame,
        }
    }

    fn print_report(&self, report: SearchStatsReport, game_ply_count: usize) {
        self.aggregate(report).print_report(report, game_ply_count);
    }
}

pub struct ChessApp {
    screen: Screen,
    mode: Option<AppMode>,
    game: Option<Game>,

    selected_square: Option<Square>,
    selected_moves: MoveList,

    pending_promotion: Option<PendingPromotion>,

    bot: Option<Bot>,
    bot_rx: Option<Receiver<BotSearchResponse>>,

    show_pgn: bool,
    pgn_text: String,

    bot_game_search_stats: BotGameSearchStats,
    selected_search_stats_report: SearchStatsReport,
}

impl Default for ChessApp {
    fn default() -> Self {
        Self {
            screen: Screen::MainMenu,
            mode: None,
            game: None,

            selected_square: None,
            selected_moves: MoveList::new(),

            pending_promotion: None,

            bot: None,
            bot_rx: None,

            show_pgn: false,
            pgn_text: String::new(),

            bot_game_search_stats: BotGameSearchStats::default(),
            selected_search_stats_report: SearchStatsReport::default(),
        }
    }
}

impl ChessApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self::default()
    }

    fn start_player_vs_player(&mut self) {
        self.mode = Some(AppMode::PlayerVsPlayer);
        self.game = Some(Game::new());

        self.bot = None;
        self.bot_rx = None;

        self.clear_pgn_display();
        self.clear_search_stats();
        self.clear_selection();
        self.screen = Screen::Game;
    }

    fn start_player_vs_bot(&mut self, human: Color) {
        let bot_color = human.opposite();

        self.mode = Some(AppMode::PlayerVsBot { human });
        self.game = Some(Game::new());

        self.bot = Some(Bot::new(
            bot_color,
            SearchLimits::depth_and_time(
                DEFAULT_BOT_DEPTH,
                DEFAULT_Q_BOT_DEPTH,
                DEFAULT_BOT_TIME_MS,
            ),
        ));
        self.bot_rx = None;

        self.clear_pgn_display();
        self.clear_search_stats();
        self.clear_selection();
        self.screen = Screen::Game;
    }

    fn return_to_main_menu(&mut self) {
        self.screen = Screen::MainMenu;
        self.mode = None;
        self.game = None;

        self.bot = None;
        self.bot_rx = None;

        self.clear_pgn_display();
        self.clear_search_stats();
        self.clear_selection();
    }

    /// This is the function the board drawing code should call later.
    ///
    /// Player vs Player:
    /// - White to move => white bottom
    /// - Black to move => black bottom
    ///
    /// Player vs Bot:
    /// - Human player always stays on bottom
    pub fn board_orientation(&self) -> BoardOrientation {
        match self.mode {
            Some(AppMode::PlayerVsPlayer) => {
                let Some(game) = &self.game else {
                    return BoardOrientation::WhiteBottom;
                };

                match game.board().side_to_move() {
                    Color::White => BoardOrientation::WhiteBottom,
                    Color::Black => BoardOrientation::BlackBottom,
                }
            }

            Some(AppMode::PlayerVsBot { human, .. }) => match human {
                Color::White => BoardOrientation::WhiteBottom,
                Color::Black => BoardOrientation::BlackBottom,
            },

            None => BoardOrientation::WhiteBottom,
        }
    }

    fn show_main_menu(&mut self, ui: &mut egui::Ui) {
        egui::CentralPanel::default().show_inside(ui, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(120.0);

                ui.heading("Rust Chess");
                ui.add_space(24.0);

                if ui
                    .add_sized([240.0, 44.0], egui::Button::new("Player vs Player"))
                    .clicked()
                {
                    self.start_player_vs_player();
                }

                ui.add_space(12.0);

                if ui
                    .add_sized([240.0, 44.0], egui::Button::new("Player vs Bot"))
                    .clicked()
                {
                    self.screen = Screen::BotSetup;
                }

                ui.add_space(16.0);

                ui.label("Bitboard rewrite");
            });
        });
    }

    fn show_bot_setup(&mut self, ui: &mut egui::Ui) {
        egui::CentralPanel::default().show_inside(ui, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(120.0);

                ui.heading("Choose Your Color");
                ui.add_space(24.0);

                if ui
                    .add_sized([240.0, 44.0], egui::Button::new("Play as White"))
                    .clicked()
                {
                    self.start_player_vs_bot(Color::White);
                }

                ui.add_space(12.0);

                if ui
                    .add_sized([240.0, 44.0], egui::Button::new("Play as Black"))
                    .clicked()
                {
                    self.start_player_vs_bot(Color::Black);
                }

                ui.add_space(24.0);

                if ui.button("Back").clicked() {
                    self.screen = Screen::MainMenu;
                }
            });
        });
    }

    fn show_game(&mut self, ui: &mut egui::Ui) {
        match self.mode {
            Some(AppMode::PlayerVsPlayer) => {
                self.show_player_vs_player_game(ui);
            }

            Some(AppMode::PlayerVsBot { .. }) => {
                self.show_player_vs_bot_game(ui);
            }

            None => {
                ui.vertical_centered(|ui| {
                    ui.heading("No game mode selected");

                    if ui.button("Return to Main Menu").clicked() {
                        self.return_to_main_menu();
                    }
                });
            }
        }
    }
    fn show_player_vs_player_game(&mut self, ui: &mut egui::Ui) {
        let Some(game) = &self.game else {
            let mut action = GameScreenAction::None;

            ui.vertical_centered(|ui| {
                ui.heading("No active game");

                ui.add_space(16.0);

                if ui.button("New Game").clicked() {
                    action = GameScreenAction::NewGame;
                }

                if ui.button("Return to Main Menu").clicked() {
                    action = GameScreenAction::MainMenu;
                }
            });

            match action {
                GameScreenAction::None
                | GameScreenAction::PrintEval
                | GameScreenAction::TogglePgn
                | GameScreenAction::PrintSearchStats => {}
                GameScreenAction::NewGame => self.restart_current_game(),
                GameScreenAction::MainMenu => self.return_to_main_menu(),
            }

            return;
        };

        let mode = self.mode.unwrap_or(AppMode::PlayerVsPlayer);
        let orientation = self.board_orientation();

        let state = game.state();
        let side_to_move = game.board().side_to_move();

        let selected_square = self.selected_square;
        let selected_moves = self.selected_moves.clone();
        let selected_move_count = selected_moves.len();

        let mut board_action = None;
        let mut action = GameScreenAction::None;

        let show_pgn = self.show_pgn;
        let pgn_text = &mut self.pgn_text;

        let available = ui.available_size();

        let side_panel_width = 240.0;
        let gap = 20.0;

        let board_max_width = (available.x - side_panel_width - gap).max(200.0);
        let board_max_height = available.y.max(200.0);

        let board_size = board_max_width.min(board_max_height).min(820.0);

        let promotion_picker = self
            .pending_promotion
            .as_ref()
            .map(|pending| PromotionPicker {
                square: pending.to,
                color: pending.color,
            });

        ui.horizontal_top(|ui| {
            board_action = draw_board_sized(
                ui,
                game,
                orientation,
                selected_square,
                &selected_moves,
                promotion_picker,
                board_size,
            );

            ui.add_space(gap);

            ui.allocate_ui_with_layout(
                egui::vec2(side_panel_width, board_size),
                egui::Layout::top_down(egui::Align::LEFT),
                |ui| {
                    action = Self::show_game_status_panel(
                        ui,
                        mode,
                        state,
                        side_to_move,
                        orientation,
                        selected_square,
                        selected_move_count,
                        show_pgn,
                        pgn_text,
                    );
                },
            );
        });

        // Apply board click after UI closure to avoid borrow conflicts.
        if state == GameState::Ongoing {
            if let Some(action) = board_action {
                self.handle_board_action(action);
            }
        } else if board_action.is_some() {
            self.clear_selection();
        }

        match action {
            GameScreenAction::None => {}
            GameScreenAction::PrintEval => self.print_current_eval(),
            GameScreenAction::TogglePgn => self.toggle_pgn_display(),
            GameScreenAction::PrintSearchStats => {}
            GameScreenAction::NewGame => self.restart_current_game(),
            GameScreenAction::MainMenu => self.return_to_main_menu(),
        }
    }
    fn restart_current_game(&mut self) {
        let Some(mode) = self.mode else {
            self.return_to_main_menu();
            return;
        };

        self.game = Some(Game::new());
        self.bot_rx = None;

        match mode {
            AppMode::PlayerVsPlayer => {
                self.bot = None;
            }
            AppMode::PlayerVsBot { human } => {
                let bot_color = human.opposite();
                self.bot = Some(Bot::new(
                    bot_color,
                    SearchLimits::depth_and_time(
                        DEFAULT_BOT_DEPTH,
                        DEFAULT_Q_BOT_DEPTH,
                        DEFAULT_BOT_TIME_MS,
                    ),
                ));
            }
        }

        // Keep the same mode:
        // - PlayerVsPlayer stays PlayerVsPlayer
        // - PlayerVsBot keeps the same human color
        self.mode = Some(mode);
        self.clear_pgn_display();
        self.clear_search_stats();
        self.clear_selection();
        self.screen = Screen::Game;
    }
    fn show_game_status_panel(
        ui: &mut egui::Ui,
        mode: AppMode,
        state: GameState,
        side_to_move: Color,
        orientation: BoardOrientation,
        selected_square: Option<Square>,
        selected_move_count: usize,
        show_pgn: bool,
        pgn_text: &mut String,
    ) -> GameScreenAction {
        let mut action = GameScreenAction::None;

        ui.heading(match mode {
            AppMode::PlayerVsPlayer => "Player vs Player",
            AppMode::PlayerVsBot { .. } => "Player vs Bot",
        });

        ui.add_space(8.0);

        match state {
            GameState::Ongoing => {
                ui.label(format!("Turn: {:?}", side_to_move));
                ui.label(format!("View: {:?}", orientation));

                ui.add_space(16.0);

                match selected_square {
                    Some(sq) => {
                        ui.label(format!("Selected: {}", square_to_algebraic(sq)));
                        ui.label(format!("Legal moves: {}", selected_move_count));
                    }
                    None => {
                        ui.label("Selected: none");
                    }
                }
            }

            _ => {
                ui.heading(Self::game_state_title(state));
                ui.label(Self::game_state_description(state));

                ui.add_space(16.0);

                ui.label("Game over.");
            }
        }

        ui.add_space(16.0);

        if ui.button("Print Eval").clicked() {
            action = GameScreenAction::PrintEval;
        }

        if state != GameState::Ongoing {
            ui.add_space(8.0);

            let pgn_button_text = if show_pgn { "Hide PGN" } else { "Show PGN" };
            if ui.button(pgn_button_text).clicked() {
                action = GameScreenAction::TogglePgn;
            }

            if show_pgn {
                ui.add_space(8.0);
                ui.label("PGN:");

                egui::ScrollArea::vertical()
                    .max_height(260.0)
                    .show(ui, |ui| {
                        ui.add(
                            egui::TextEdit::multiline(pgn_text)
                                .desired_width(f32::INFINITY)
                                .desired_rows(10),
                        );
                    });
            }
        }

        ui.add_space(24.0);

        if ui.button("New Game").clicked() {
            action = GameScreenAction::NewGame;
        }

        if ui.button("Return to Main Menu").clicked() {
            action = GameScreenAction::MainMenu;
        }

        action
    }

    fn play_ui_move(&mut self, mv: Move) {
        let Some(game) = &mut self.game else {
            self.clear_selection();
            return;
        };

        match game.play_move(mv) {
            Ok(()) => {
                self.clear_selection();
            }
            Err(err) => {
                eprintln!("Failed to play move {:?}: {:?}", mv, err);
                self.clear_selection();
            }
        }
    }

    // *******************
    // **** BOT LOGIC ****
    // *******************

    fn show_player_vs_bot_game(&mut self, ui: &mut egui::Ui) {
        let Some(game) = &self.game else {
            let mut action = GameScreenAction::None;

            ui.vertical_centered(|ui| {
                ui.heading("No active game");

                ui.add_space(16.0);

                if ui.button("New Game").clicked() {
                    action = GameScreenAction::NewGame;
                }

                if ui.button("Return to Main Menu").clicked() {
                    action = GameScreenAction::MainMenu;
                }
            });

            match action {
                GameScreenAction::None
                | GameScreenAction::PrintEval
                | GameScreenAction::TogglePgn
                | GameScreenAction::PrintSearchStats => {}
                GameScreenAction::NewGame => self.restart_current_game(),
                GameScreenAction::MainMenu => self.return_to_main_menu(),
            }

            return;
        };

        let mode = self.mode.unwrap_or(AppMode::PlayerVsBot {
            human: Color::White,
        });

        let orientation = self.board_orientation();
        let state = game.state();
        let side_to_move = game.board().side_to_move();

        let selected_square = self.selected_square;
        let selected_moves = self.selected_moves.clone();
        let selected_move_count = selected_moves.len();

        let bot_thinking = self.is_bot_thinking();
        let human_can_interact = self.human_can_interact();

        let bot_color = self.bot.as_ref().map(|bot| bot.color);

        let mut board_action = None;
        let mut action = GameScreenAction::None;

        let show_pgn = self.show_pgn;
        let pgn_text = &mut self.pgn_text;
        let selected_search_stats_report = &mut self.selected_search_stats_report;

        let available = ui.available_size();

        let side_panel_width = 260.0;
        let gap = 20.0;

        let board_max_width = (available.x - side_panel_width - gap).max(200.0);
        let board_max_height = available.y.max(200.0);
        let board_size = board_max_width.min(board_max_height).min(820.0);

        let promotion_picker = self
            .pending_promotion
            .as_ref()
            .map(|pending| PromotionPicker {
                square: pending.to,
                color: pending.color,
            });

        ui.horizontal_top(|ui| {
            board_action = draw_board_sized(
                ui,
                game,
                orientation,
                selected_square,
                &selected_moves,
                promotion_picker,
                board_size,
            );

            ui.add_space(gap);

            ui.allocate_ui_with_layout(
                egui::vec2(side_panel_width, board_size),
                egui::Layout::top_down(egui::Align::LEFT),
                |ui| {
                    action = Self::show_bot_status_panel(
                        ui,
                        mode,
                        state,
                        side_to_move,
                        orientation,
                        selected_square,
                        selected_move_count,
                        bot_color,
                        bot_thinking,
                        show_pgn,
                        pgn_text,
                        selected_search_stats_report,
                    );
                },
            );
        });

        if human_can_interact {
            if let Some(action) = board_action {
                self.handle_board_action(action);
            }
        } else if board_action.is_some() && self.pending_promotion.is_none() {
            // Ignore clicks while bot is thinking / bot to move.
            self.clear_selection();
        }

        match action {
            GameScreenAction::None => {}
            GameScreenAction::PrintEval => self.print_current_eval(),
            GameScreenAction::TogglePgn => self.toggle_pgn_display(),
            GameScreenAction::PrintSearchStats => self.print_selected_search_stats(),
            GameScreenAction::NewGame => self.restart_current_game(),
            GameScreenAction::MainMenu => self.return_to_main_menu(),
        }
    }

    fn show_bot_status_panel(
        ui: &mut egui::Ui,
        mode: AppMode,
        state: GameState,
        side_to_move: Color,
        orientation: BoardOrientation,
        selected_square: Option<Square>,
        selected_move_count: usize,
        bot_color: Option<Color>,
        bot_thinking: bool,
        show_pgn: bool,
        pgn_text: &mut String,
        selected_search_stats_report: &mut SearchStatsReport,
    ) -> GameScreenAction {
        let mut action = GameScreenAction::None;

        ui.heading("Player vs Bot");
        ui.add_space(8.0);

        let human_color = match mode {
            AppMode::PlayerVsBot { human } => Some(human),
            AppMode::PlayerVsPlayer => None,
        };

        if let Some(human) = human_color {
            ui.label(format!("You: {:?}", human));
        }

        if let Some(bot) = bot_color {
            ui.label(format!("Bot: {:?}", bot));
        }

        ui.label(format!("View: {:?}", orientation));
        ui.add_space(12.0);

        match state {
            GameState::Ongoing => {
                ui.label(format!("Turn: {:?}", side_to_move));

                if bot_thinking {
                    ui.label("Bot is thinking...");
                } else if Some(side_to_move) == human_color {
                    ui.label("Your move.");
                } else {
                    ui.label("Waiting for bot...");
                }

                ui.add_space(16.0);

                match selected_square {
                    Some(sq) => {
                        ui.label(format!("Selected: {}", square_to_algebraic(sq)));
                        ui.label(format!("Legal moves: {}", selected_move_count));
                    }
                    None => {
                        ui.label("Selected: none");
                    }
                }
            }

            _ => {
                ui.heading(Self::game_state_title(state));
                ui.label(Self::game_state_description(state));

                ui.add_space(16.0);

                ui.label("Game over.");
            }
        }

        ui.add_space(16.0);

        if ui.button("Print Eval").clicked() {
            action = GameScreenAction::PrintEval;
        }

        if state != GameState::Ongoing {
            ui.add_space(8.0);

            let pgn_button_text = if show_pgn { "Hide PGN" } else { "Show PGN" };
            if ui.button(pgn_button_text).clicked() {
                action = GameScreenAction::TogglePgn;
            }

            ui.label("Search stats report:");
            egui::ComboBox::from_id_salt("search_stats_report")
                .selected_text(selected_search_stats_report.label())
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        selected_search_stats_report,
                        SearchStatsReport::Total,
                        "Total",
                    );
                    ui.selectable_value(
                        selected_search_stats_report,
                        SearchStatsReport::Opening,
                        "Opening",
                    );
                    ui.selectable_value(
                        selected_search_stats_report,
                        SearchStatsReport::Middlegame,
                        "Middlegame",
                    );
                    ui.selectable_value(
                        selected_search_stats_report,
                        SearchStatsReport::Endgame,
                        "Endgame",
                    );
                });

            if ui.button("Print Selected Search Stats").clicked() {
                action = GameScreenAction::PrintSearchStats;
            }

            if show_pgn {
                ui.add_space(8.0);
                ui.label("PGN:");

                egui::ScrollArea::vertical()
                    .max_height(260.0)
                    .show(ui, |ui| {
                        ui.add(
                            egui::TextEdit::multiline(pgn_text)
                                .desired_width(f32::INFINITY)
                                .desired_rows(10),
                        );
                    });
            }
        }

        ui.add_space(24.0);

        if ui.button("New Game").clicked() {
            action = GameScreenAction::NewGame;
        }

        if ui.button("Return to Main Menu").clicked() {
            action = GameScreenAction::MainMenu;
        }

        action
    }

    fn start_bot_search_if_needed(&mut self) {
        if !matches!(self.mode, Some(AppMode::PlayerVsBot { .. })) {
            return;
        }

        // Already waiting for a bot result.
        if self.bot_rx.is_some() {
            return;
        }

        let Some(game) = &self.game else {
            return;
        };

        if game.state() != GameState::Ongoing {
            return;
        }

        let Some(bot) = &mut self.bot else {
            return;
        };

        if !bot.can_start_search(game) {
            return;
        }

        let Some(engine) = bot.take_engine() else {
            return;
        };

        let request = BotSearchRequest {
            board: game.board().clone(),
            limits: bot.limits,
            repetition_history: game.repetition_history().to_vec(),
            engine,
        };

        let (tx, rx) = mpsc::channel();

        bot.thinking = true;
        self.bot_rx = Some(rx);

        self.selected_square = None;
        self.selected_moves.clear();
        self.pending_promotion = None;

        thread::spawn(move || {
            let BotSearchRequest {
                board,
                limits,
                repetition_history,
                mut engine,
            } = request;
            println!("----------- Start -----------");
            let result = engine.search(&board, limits, &repetition_history, true, true);
            println!("------------ End ------------");

            let _ = tx.send(BotSearchResponse { engine, result });
        });
    }

    fn poll_bot_result(&mut self) {
        let Some(rx) = self.bot_rx.take() else {
            return;
        };

        match rx.try_recv() {
            Ok(response) => {
                self.apply_bot_search_response(response);
            }

            Err(TryRecvError::Empty) => {
                self.bot_rx = Some(rx);
            }

            Err(TryRecvError::Disconnected) => {
                // Search thread failed or panicked. Keep UI recoverable.
                if let Some(bot) = &mut self.bot {
                    bot.thinking = false;

                    if bot.engine.is_none() {
                        bot.engine = Some(Engine::new(EngineConfig::default()));
                    }
                }
            }
        }
    }

    fn apply_bot_search_response(&mut self, response: BotSearchResponse) {
        let BotSearchResponse { engine, result } = response;

        // Every completed engine.search result contributes to this game's
        // aggregate, including zero-node opening-book results.
        if let Some(game) = &self.game {
            self.bot_game_search_stats.record(
                game.move_history.len(),
                game.board().phase(),
                &result,
            );
        }

        let bot_color = self.bot.as_ref().map(|bot| bot.color);

        if let Some(bot) = &mut self.bot {
            bot.restore_engine(engine);
        }

        let Some(best_move) = result.best_move else {
            // panic!("No Best Move in apply_bot_search_response!");
            self.clear_selection();
            return;
        };

        let Some(game) = &mut self.game else {
            self.clear_selection();
            return;
        };

        // Ignore stale results if the position is no longer on the bot's turn.
        if let Some(bot_color) = bot_color {
            if game.state() == GameState::Ongoing && game.board().side_to_move() == bot_color {
                if let Err(err) = game.play_move(best_move) {
                    eprintln!("Bot failed to play move {:?}: {:?}", best_move, err);
                }
            }
        }

        self.clear_selection();
    }

    fn open_promotion_picker(&mut self, to: Square, promotion_moves: Vec<Move>) {
        let Some(game) = &self.game else {
            self.clear_selection();
            return;
        };

        self.pending_promotion = Some(PendingPromotion {
            to,
            color: game.board().side_to_move(),
            moves: promotion_moves,
        });
    }

    // *******************
    // **** PROMOTION ****
    // *******************

    fn finish_promotion(&mut self, promotion: PieceType) {
        let Some(pending) = self.pending_promotion.take() else {
            return;
        };

        let Some(mv) = pending
            .moves
            .iter()
            .copied()
            .find(|mv| mv.promotion() == Some(promotion))
        else {
            self.clear_selection();
            return;
        };

        self.play_ui_move(mv);
    }

    fn cancel_promotion(&mut self) {
        // Keep the pawn selected and keep legal moves visible.
        self.pending_promotion = None;
    }

    // **************************
    // **** HELPER FUNCTIONS ****
    // **************************

    fn print_current_eval(&self) {
        let Some(game) = &self.game else {
            return;
        };

        let breakdown = evaluation_breakdown(game.board());
        println!("Current eval breakdown:\n{:#?}", breakdown);
    }

    fn toggle_pgn_display(&mut self) {
        if self.show_pgn {
            self.show_pgn = false;
            return;
        }

        let Some(mode) = self.mode else {
            return;
        };

        let Some(game) = &self.game else {
            return;
        };

        if game.state() == GameState::Ongoing {
            return;
        }

        let metadata = Self::pgn_metadata(mode);

        match game_to_pgn(game, &metadata) {
            Ok(pgn) => {
                self.pgn_text = pgn;
                self.show_pgn = true;
            }
            Err(err) => {
                eprintln!("Failed to generate PGN: {:?}", err);
            }
        }
    }

    fn pgn_metadata(mode: AppMode) -> PgnMetadata {
        let mut metadata = PgnMetadata::default();

        match mode {
            AppMode::PlayerVsPlayer => {
                metadata.event = "Player vs Player".to_string();
                metadata.white = "White".to_string();
                metadata.black = "Black".to_string();
            }
            AppMode::PlayerVsBot {
                human: Color::White,
            } => {
                metadata.event = "Player vs Bot".to_string();
                metadata.white = "Player".to_string();
                metadata.black = "Bot".to_string();
            }
            AppMode::PlayerVsBot {
                human: Color::Black,
            } => {
                metadata.event = "Player vs Bot".to_string();
                metadata.white = "Bot".to_string();
                metadata.black = "Player".to_string();
            }
        }

        metadata
    }

    fn clear_pgn_display(&mut self) {
        self.show_pgn = false;
        self.pgn_text.clear();
    }

    fn print_selected_search_stats(&self) {
        let Some(game) = &self.game else {
            return;
        };

        if game.state() == GameState::Ongoing {
            return;
        }

        self.bot_game_search_stats
            .print_report(self.selected_search_stats_report, game.move_history.len());
    }

    fn clear_search_stats(&mut self) {
        self.bot_game_search_stats = BotGameSearchStats::default();
        self.selected_search_stats_report = SearchStatsReport::default();
    }

    fn clear_selection(&mut self) {
        self.selected_square = None;
        self.selected_moves.clear();
        self.pending_promotion = None;
    }

    fn select_square(&mut self, sq: Square) {
        self.pending_promotion = None;

        let Some(game) = &mut self.game else {
            self.clear_selection();
            return;
        };

        let Some(piece) = game.board().piece_at(sq) else {
            self.clear_selection();
            return;
        };

        self.selected_square = Some(sq);

        // Highlight any piece, but only show legal moves for side-to-move pieces.
        if piece.color == game.board().side_to_move() {
            self.selected_moves = game.legal_moves_from(sq);
        } else {
            self.selected_moves.clear();
        }
    }

    fn handle_board_click(&mut self, sq: Square) {
        let Some(game) = &self.game else {
            self.clear_selection();
            return;
        };

        if game.state() != GameState::Ongoing {
            self.clear_selection();
            return;
        }

        let target_moves = self.selected_moves_to(sq);

        if !target_moves.is_empty() {
            let promotion_moves: Vec<Move> = target_moves
                .iter()
                .copied()
                .filter(|mv| mv.promotion().is_some())
                .collect();

            if !promotion_moves.is_empty() {
                self.open_promotion_picker(sq, promotion_moves);
                return;
            }

            self.play_ui_move(target_moves[0]);
            return;
        }

        self.select_square(sq);
    }

    fn game_state_title(state: GameState) -> &'static str {
        match state {
            GameState::Ongoing => "Game in progress",
            GameState::Checkmate { .. } => "Checkmate",
            GameState::Stalemate => "Stalemate",
            GameState::DrawByRepetition => "Draw by repetition",
            GameState::DrawByFiftyMoveRule => "Draw by fifty-move rule",
            GameState::DrawByInsufficientMaterial => "Draw by insufficient material",
        }
    }

    fn game_state_description(state: GameState) -> String {
        match state {
            GameState::Ongoing => "The game is still ongoing.".to_string(),

            GameState::Checkmate { winner } => {
                format!("{:?} wins by checkmate.", winner)
            }

            GameState::Stalemate => "The game is drawn by stalemate.".to_string(),

            GameState::DrawByRepetition => "The game is drawn by threefold repetition.".to_string(),

            GameState::DrawByFiftyMoveRule => {
                "The game is drawn by the fifty-move rule.".to_string()
            }

            GameState::DrawByInsufficientMaterial => {
                "The game is drawn by insufficient material.".to_string()
            }
        }
    }

    fn selected_moves_to(&self, sq: Square) -> Vec<Move> {
        self.selected_moves
            .iter()
            .copied()
            .filter(|mv| mv.to() == sq)
            .collect()
    }

    fn is_bot_thinking(&self) -> bool {
        self.bot.as_ref().is_some_and(|bot| bot.thinking)
    }

    fn human_can_interact(&self) -> bool {
        let Some(game) = &self.game else {
            return false;
        };

        if game.state() != GameState::Ongoing {
            return false;
        }

        match self.mode {
            Some(AppMode::PlayerVsPlayer) => true,

            Some(AppMode::PlayerVsBot { human }) => {
                game.board().side_to_move() == human && !self.is_bot_thinking()
            }

            None => false,
        }
    }

    fn handle_board_action(&mut self, action: BoardAction) {
        match action {
            BoardAction::Square(sq) => {
                self.handle_board_click(sq);
            }

            BoardAction::Promotion(piece) => {
                self.finish_promotion(piece);
            }

            BoardAction::PromotionCancel => {
                self.cancel_promotion();
            }
        }
    }
}

impl eframe::App for ChessApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.poll_bot_result();

        match self.screen {
            Screen::MainMenu => self.show_main_menu(ui),
            Screen::BotSetup => self.show_bot_setup(ui),
            Screen::Game => self.show_game(ui),
        }

        self.start_bot_search_if_needed();

        if self.bot_rx.is_some() || self.is_bot_thinking() {
            ui.ctx().request_repaint();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::MAX_PV;

    fn result(main_nodes: u64, elapsed_ms: u64, termination: SearchTermination) -> SearchResult {
        let mut stats = SearchStats::default();
        stats.node_stats.main = main_nodes;

        SearchResult {
            best_move: Some(Move::NULL),
            eval: 0,
            depth_reached: 1,
            stats,
            pv: [None; MAX_PV],
            elapsed: Duration::from_millis(elapsed_ms),
            termination,
        }
    }

    #[test]
    fn bot_game_search_stats_aggregate_total_and_non_regressing_phase_buckets() {
        let mut totals = BotGameSearchStats::default();
        totals.record(0, 24, &result(0, 5, SearchTermination::BookMove));
        totals.record(10, 24, &result(20, 20, SearchTermination::TimeLimit));
        totals.record(20, 24, &result(30, 30, SearchTermination::TimeLimit));
        totals.record(30, 12, &result(40, 40, SearchTermination::TimeLimit));

        // A promotion can increase the material phase, but the game must not
        // move backward from endgame to middlegame.
        totals.record(32, 16, &result(50, 50, SearchTermination::TimeLimit));

        assert_eq!(totals.current_phase, GamePhase::Endgame);
        assert_eq!(totals.total.move_count, 5);
        assert_eq!(totals.opening.move_count, 2);
        assert_eq!(totals.middlegame.move_count, 1);
        assert_eq!(totals.endgame.move_count, 2);

        assert_eq!(totals.total.stats.node_stats.main, 140);
        assert_eq!(totals.opening.stats.node_stats.main, 20);
        assert_eq!(totals.middlegame.stats.node_stats.main, 30);
        assert_eq!(totals.endgame.stats.node_stats.main, 90);
        assert_eq!(totals.total.elapsed, Duration::from_millis(145));

        assert_eq!(
            totals.opening.stats.node_stats.main
                + totals.middlegame.stats.node_stats.main
                + totals.endgame.stats.node_stats.main,
            totals.total.stats.node_stats.main,
        );
    }
}
