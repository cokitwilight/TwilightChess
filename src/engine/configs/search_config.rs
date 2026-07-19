use crate::engine::configs::{
    AspirationConfig, DeltaPruneConfig, LMRConfig, NullMoveConfig, RFPConfig, SEEConfig,
};

#[derive(Clone, Debug)]
pub struct SearchConfig {
    pub aspiration: AspirationConfig,
    pub null_move: NullMoveConfig,
    pub delta: DeltaPruneConfig,
    pub lmr: LMRConfig,
    pub see: SEEConfig,
    pub rfp: RFPConfig,
    // etc
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
        }
    }
}
