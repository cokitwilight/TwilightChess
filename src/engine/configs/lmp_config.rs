#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "debug-uci", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "debug-uci", serde(deny_unknown_fields))]
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
