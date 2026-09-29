#[derive(Clone, Copy, Debug)]
pub struct CorrectionConfig {
    pub enabled: bool,
}

impl Default for CorrectionConfig {
    fn default() -> Self {
        Self { enabled: true }
    }
}
