#[derive(Clone, Debug)]
pub struct AspirationConfig {
    pub enabled: bool,

    pub initial_window: i32,
    pub growth_factor: i32,

    pub max_window: i32,
    pub mate_margin: i32,
}

impl Default for AspirationConfig {
    fn default() -> Self {
        Self {
            enabled: true,

            initial_window: 25,
            growth_factor: 2,

            max_window: 800,
            mate_margin: 1000,
        }
    }
}
