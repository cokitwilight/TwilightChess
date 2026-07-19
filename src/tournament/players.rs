use crate::engine::configs::EngineConfig;

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
