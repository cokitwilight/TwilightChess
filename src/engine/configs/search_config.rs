use crate::engine::configs::{
    AspirationConfig, DeltaPruneConfig, FutilityConfig, LMRConfig, NullMoveConfig, RFPConfig,
    SEEConfig,
};

#[derive(Clone, Copy, Debug)]
pub struct SearchConfig {
    pub aspiration: AspirationConfig,
    pub null_move: NullMoveConfig,
    pub delta: DeltaPruneConfig,
    pub lmr: LMRConfig,
    pub see: SEEConfig,
    pub rfp: RFPConfig,
    pub fut: FutilityConfig, // etc
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self {
            aspiration: AspirationConfig::default(),
            null_move: NullMoveConfig::default(),
            delta: DeltaPruneConfig::default(),
            lmr: LMRConfig::default(),
            see: SEEConfig::default(),
            rfp: RFPConfig::default(),
            fut: FutilityConfig::default(),
        }
    }
}
