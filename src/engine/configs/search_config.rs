use crate::engine::configs::{AspirationConfig, NullMoveConfig};

#[derive(Clone, Debug)]
pub struct SearchConfig {
    pub aspiration: AspirationConfig,
    pub null_move: NullMoveConfig,
    // pub lmr: LmrConfig,
    // reverse_futility: ReverseFutilityConfig
    // etc
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self {
            aspiration: AspirationConfig::default(),
            null_move: NullMoveConfig::default(),
        }
    }
}
