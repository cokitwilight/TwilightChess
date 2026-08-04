#[derive(Clone, Copy, Debug)]
pub struct SEEConfig {
    pub enabled: bool,

    pub margin: i32,
}

impl Default for SEEConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            margin: -200,
        }
    }
}
