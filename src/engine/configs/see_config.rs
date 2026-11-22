#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "debug-uci", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "debug-uci", serde(deny_unknown_fields))]
pub struct SEEConfig {
    pub enabled: bool,

    pub margin: i32,
}

impl Default for SEEConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            margin: 330, // used as see value < -margin
                         // defualt is 330 since that is the value of a minor piece.
        }
    }
}
