#[derive(Clone, Debug)]
pub struct DeltaPruneConfig {
    pub enabled: bool,

    pub margin: i32,
}

impl Default for DeltaPruneConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            margin: 200,
        }
    }
}
