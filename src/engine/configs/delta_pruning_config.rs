#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "debug-uci", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "debug-uci", serde(deny_unknown_fields))]
pub struct DeltaPruneConfig {
    pub enabled: bool,

    pub margin: i32,
}

impl Default for DeltaPruneConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            margin: 300,
        }
    }
}
