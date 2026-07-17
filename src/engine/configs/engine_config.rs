use crate::engine::configs::SearchConfig;

#[derive(Clone, Debug)]
pub struct EngineConfig {
    pub search: SearchConfig,
    // pub eval: EvaluationConfig,
    pub tt_size: usize, // In MB
    pub qtt_size: usize, // In MB

                        // pub opening_line: OpeningBook  // This might be better somewhere else but it should overall force certain lines between the engine
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            search: SearchConfig::default(),
            tt_size: 64,
            qtt_size: 64,
        }
    }
}
