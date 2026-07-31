#[derive(Clone, Copy, Debug)]
pub struct FutilityConfig {
    pub enabled: bool,
    // add different values here
    pub margin: u16,
    pub max_depth: u16,
}

impl Default for FutilityConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            margin: 100,
            max_depth: 2,
        }
    }
}
