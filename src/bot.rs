use crate::engine::{Engine, SearchLimits};
use crate::game::{Game, GameState};
use crate::types::Color;

pub struct Bot {
    pub color: Color,
    pub limits: SearchLimits,
    pub engine: Option<Engine>,
    pub thinking: bool,
}

impl Bot {
    pub fn new(color: Color, limits: SearchLimits) -> Self {
        Self {
            color,
            limits,
            engine: Some(Engine::new()),
            thinking: false,
        }
    }

    pub fn is_turn(&self, game: &Game) -> bool {
        game.state() == GameState::Ongoing && game.board().side_to_move() == self.color
    }

    pub fn can_start_search(&self, game: &Game) -> bool {
        self.is_turn(game) && !self.thinking && self.engine.is_some()
    }

    pub fn take_engine(&mut self) -> Option<Engine> {
        self.engine.take()
    }

    pub fn restore_engine(&mut self, engine: Engine) {
        self.engine = Some(engine);
        self.thinking = false;
    }
}
