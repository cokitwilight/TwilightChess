use std::time::Duration;

use crate::board::Move;
use crate::engine::SearchStats;
use crate::game::Game;
use crate::types::Color;

#[derive(Debug, Clone)]
pub enum GameResult {
    Draw,
    White,
    Black,
}

#[derive(Debug, Clone)]
pub struct MoveRecord {
    pub mv: Move,
    pub eval: i32,
}

#[derive(Clone)]
pub struct GameRecord {
    pub id: String,
    pub result: Option<GameResult>,
    pub start_fen: String,
    pub opening_name: Option<String>,
    pub move_history: Vec<MoveRecord>,
    pub game: Game,

    pub engine_1_color: Option<Color>,

    // game stats
    pub white_stats: SearchStats,
    pub white_time_elapsed: Duration,
    pub white_avg_time: Duration,

    pub black_stats: SearchStats,
    pub black_time_elapsed: Duration,
    pub black_avg_time: Duration,

    pub total_time: Duration,
}

impl GameRecord {
    pub fn new(start_fen: String, engine_1: Color) -> Self {
        Self {
            id: String::new(),
            result: None,
            start_fen,
            opening_name: None,
            move_history: Vec::with_capacity(50),
            game: Game::new(),

            engine_1_color: Some(engine_1),

            white_stats: SearchStats::default(),
            white_time_elapsed: Duration::ZERO,
            white_avg_time: Duration::ZERO,

            black_stats: SearchStats::default(),
            black_time_elapsed: Duration::ZERO,
            black_avg_time: Duration::ZERO,

            total_time: Duration::ZERO,
        }
    }
}
