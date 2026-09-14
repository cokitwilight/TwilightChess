#[derive(Clone, Copy, Debug)]
pub struct LMPConfig {
    pub enabled: bool,
    pub max_depth: u16,
}

impl Default for LMPConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            max_depth: 5,
        }
    }
}
