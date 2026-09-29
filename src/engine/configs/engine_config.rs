use crate::engine::SearchLimits;
use crate::engine::configs::SearchConfig;

#[derive(Clone, Copy, Debug)]
pub struct EngineConfig {
    pub search: SearchConfig,
    pub limits: SearchLimits,
    // pub eval: EvaluationConfig,
    pub tt_size: usize, // In MB

                        // pub opening_line: OpeningBook  // This might be better somewhere else but it should overall force certain lines between the engine
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            search: SearchConfig::default(),
            limits: SearchLimits::depth_and_time(50, 6, 250),
            tt_size: 128,
        }
    }
}

impl EngineConfig {
    pub fn standard() -> Self {
        let limits = SearchLimits::depth_and_time(50, 24, 250);
        let tt_size = 128;

        Self {
            search: SearchConfig::standard(),
            limits,
            tt_size,
        }
    }
}
