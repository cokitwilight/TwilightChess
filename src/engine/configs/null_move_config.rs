#[derive(Clone, Debug)]
pub struct NullMoveConfig {
    pub enabled: bool,

    pub minimum_depth: usize,
    pub minimum_phase: i32,
    // Add more if needed
}

impl Default for NullMoveConfig {
    fn default() -> Self {
        Self {
            enabled: true,

            minimum_depth: 4,
            minimum_phase: 8,
        }
    }
}
