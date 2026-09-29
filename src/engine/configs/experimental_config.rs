#[derive(Clone, Copy, Debug)]
pub struct ExperimentalConfig {
    pub enabled: bool,
}

impl Default for ExperimentalConfig {
    fn default() -> Self {
        Self { enabled: true }
    }
}
