#[derive(Clone, Debug)]
pub struct LMRConfig {
    pub enabled: bool,
    // add different values here
}

impl Default for LMRConfig {
    fn default() -> Self {
        Self { enabled: true }
    }
}
