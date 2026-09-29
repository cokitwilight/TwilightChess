use crate::engine::configs::{
    AspirationConfig, CorrectionConfig, DeltaPruneConfig, ExperimentalConfig, FutilityConfig,
    LMPConfig, LMRConfig, NullMoveConfig, RFPConfig, SEEConfig, SingularConfig,
};

#[derive(Clone, Copy, Debug)]
pub struct SearchConfig {
    pub aspiration: AspirationConfig,
    pub null_move: NullMoveConfig,
    pub delta: DeltaPruneConfig,
    pub lmr: LMRConfig,
    pub lmp: LMPConfig,
    pub see: SEEConfig,
    pub rfp: RFPConfig,
    pub fut: FutilityConfig, // etc
    pub singular: SingularConfig,
    pub correction: CorrectionConfig,
    pub experimental: ExperimentalConfig,
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self {
            aspiration: AspirationConfig::default(),
            null_move: NullMoveConfig::default(),
            delta: DeltaPruneConfig::default(),
            lmr: LMRConfig::default(),
            lmp: LMPConfig::default(),
            see: SEEConfig::default(),
            rfp: RFPConfig::default(),
            fut: FutilityConfig::default(),
            singular: SingularConfig::default(),
            correction: CorrectionConfig::default(),
            experimental: ExperimentalConfig::default(),
        }
    }
}

impl SearchConfig {
    pub fn standard() -> Self {
        let mut aspiration = AspirationConfig::default();
        let mut null_move = NullMoveConfig::default();
        let mut delta = DeltaPruneConfig::default();
        let mut lmr = LMRConfig::default();
        let mut lmp = LMPConfig::default();
        let mut see = SEEConfig::default();
        let mut rfp = RFPConfig::default();
        let mut fut = FutilityConfig::default();
        let mut singular = SingularConfig::default();
        let mut correction = CorrectionConfig::default();
        let mut experimental = ExperimentalConfig::default();

        aspiration.enabled = true;
        null_move.enabled = false;
        delta.enabled = false;
        lmr.enabled = false;
        lmp.enabled = false;
        see.enabled = false;
        rfp.enabled = false;
        fut.enabled = false;
        singular.enabled = false;
        correction.enabled = false;
        experimental.enabled = false;

        Self {
            aspiration,
            null_move,
            delta,
            lmr,
            lmp,
            see,
            rfp,
            fut,
            singular,
            correction,
            experimental,
        }
    }
}
