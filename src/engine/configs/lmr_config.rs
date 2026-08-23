#[derive(Clone, Copy, Debug)]
pub struct LMRConfig {
    pub enabled: bool,
    // add different values here
    pub base: f64,
    pub divisor: f64,

    pub history_enabled: bool,
    pub history_scale: i32,
}

impl Default for LMRConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            base: 0.75,
            divisor: 2.25,
            history_enabled: true,
            history_scale: 94,
        }
        // Base: is the initial value added to the reduction formula. Increasing/Decreasing affects ALL values
        // Divisor: how much the ln(depth) * ln(move #) affects the reduction. Increasing/Decreasing will weaken/strengthen how fast the values climb/stop climbing
        // history score ~capped from -48,000 - 48,000. Scaled values are ply * 256(lmr scaler) so range 0-3 = 0 - 768
        // so approximately 48_000 / scaler = 768 ~= 63, 94 for max 2 plies,
        // General formula -HISTORY_MAX - HISTORY_MAX for ply * LMR_SCALE for range 0-MAX_PLY =
        // HISTORY_MAX / (LMR_SCALE * MAX_PLY) = scaler
    }
}
