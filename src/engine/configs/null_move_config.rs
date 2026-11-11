#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "debug-uci", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "debug-uci", serde(deny_unknown_fields))]
pub struct NullMoveConfig {
    pub enabled: bool,

    pub minimum_depth: u16,
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
