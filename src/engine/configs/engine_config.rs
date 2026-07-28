use crate::engine::{SearchLimits, configs::SearchConfig};

#[derive(Clone, Copy, Debug)]
pub struct EngineConfig {
    pub search: SearchConfig,
    pub limits: SearchLimits,
    // pub eval: EvaluationConfig,
    pub tt_size: usize, // In MB
    pub qtt_size: usize, // In MB

                        // pub opening_line: OpeningBook  // This might be better somewhere else but it should overall force certain lines between the engine
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            search: SearchConfig::default(),
            limits: SearchLimits::depth(8, 6),
            tt_size: 512,
            qtt_size: 64,
        }
    }
}
