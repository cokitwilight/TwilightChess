use crate::engine::{SearchLimits, configs::EngineConfig};

#[derive(Clone, Debug)]
pub struct EnginePlayer {
    // pub id: ID
    pub name: String,
    pub config: EngineConfig,
}

#[derive(Clone, Debug)]
pub struct MatchPlayers {
    pub white: EnginePlayer,
    pub black: EnginePlayer,
}

impl EnginePlayer {
    pub fn new(name: String) -> Self {
        Self {
            name,
            config: EngineConfig::default(),
        }
    }
}

impl MatchPlayers {
    pub fn new(white: String, black: String) -> Self {
        Self {
            white: EnginePlayer::new(white),
            black: EnginePlayer::new(black),
        }
    }

    pub fn from_depth(
        white: String,
        black: String,
        w_depth: u16,
        w_q_depth: u16,
        b_depth: u16,
        b_q_depth: u16,
    ) -> Self {
        let white_limit = SearchLimits::depth(w_depth, w_q_depth);
        let black_limit = SearchLimits::depth(b_depth, b_q_depth);

        let mut white_config = EngineConfig::default();
        let mut black_config = EngineConfig::default();

        white_config.limits = white_limit;
        black_config.limits = black_limit;

        let white = EnginePlayer {
            name: white,
            config: white_config,
        };

        let black = EnginePlayer {
            name: black,
            config: black_config,
        };

        Self { white, black }
    }
}
