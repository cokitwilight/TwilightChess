#[derive(Clone, Copy, Debug)]
pub struct SingularConfig {
    pub enabled: bool,

    pub minimum_depth: u16,
    pub base_margin: i32,
    pub depth_margin: i32,
}

impl Default for SingularConfig {
    fn default() -> Self {
        Self {
            enabled: true,

            minimum_depth: 7,
            base_margin: 50, // used as see value < -margin
            depth_margin: 10,
        }
    }
}
