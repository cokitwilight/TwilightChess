use crate::engine::configs::{
    AspirationConfig, DeltaPruneConfig, FutilityConfig, LMRConfig, NullMoveConfig, RFPConfig,
    SEEConfig, SingularConfig,
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
    pub singular: SingularConfig,
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
            singular: SingularConfig::default(),
        }
    }
}

impl SearchConfig {
    pub fn standard() -> Self {
        let mut aspiration = AspirationConfig::default();
        let mut null_move = NullMoveConfig::default();
        let mut delta = DeltaPruneConfig::default();
        let mut lmr = LMRConfig::default();
        let mut see = SEEConfig::default();
        let mut rfp = RFPConfig::default();
        let mut fut = FutilityConfig::default();
        let mut singular = SingularConfig::default();

        aspiration.enabled = true;
        null_move.enabled = false;
        delta.enabled = false;
        lmr.enabled = false;
        see.enabled = false;
        rfp.enabled = false;
        fut.enabled = false;
        singular.enabled = false;

        Self {
            aspiration,
            null_move,
            delta,
            lmr,
            see,
            rfp,
            fut,
            singular,
        }
    }
}
