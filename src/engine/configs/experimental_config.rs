#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "debug-uci", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "debug-uci", serde(deny_unknown_fields))]
pub struct ExperimentalConfig {
    pub enabled: bool,
}

impl Default for ExperimentalConfig {
    fn default() -> Self {
        Self { enabled: true }
    }
}
