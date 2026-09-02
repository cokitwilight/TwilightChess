#[derive(Clone, Copy, Debug)]
pub struct FutilityConfig {
    pub enabled: bool,
    // add different values here
    pub margin: u16,
    pub max_depth: u16,

    pub history_enabled: bool,
    pub history_margin: u16,
    pub good_history: i32,
    pub bad_history: i32,
}

impl Default for FutilityConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            margin: 110,
            max_depth: 1,
            history_enabled: true,
            history_margin: 20,
            good_history: 16000,
            bad_history: -16000,
        }
    }
}
