use std::time::Duration;

use crate::{board::Move, engine::SearchStats, game::Game};

#[derive(Debug)]
pub enum GameResult {
    Draw,
    White,
    Black,
}

pub struct GameRecord {
    pub id: String,
    pub result: Option<GameResult>,
    pub start_fen: String,
    pub move_history: Vec<Move>,
    pub game: Game,

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
    pub fn new(start_fen: String) -> Self {
        Self {
            id: String::new(),
            result: None,
            start_fen,
            move_history: Vec::with_capacity(50),
            game: Game::new(),

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
