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
            enabled: true,
            margin: 110,
            max_depth: 1,
        }
    }
}
