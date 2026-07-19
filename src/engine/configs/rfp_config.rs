#[derive(Clone, Debug)]
pub struct RFPConfig {
    pub enabled: bool,

    pub max_depth: usize,

    pub min_phase: i32,

    pub margin_factor: i32,
}

impl Default for RFPConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_depth: 4,
            min_phase: 6,
            margin_factor: 80,
        }
    }
}
