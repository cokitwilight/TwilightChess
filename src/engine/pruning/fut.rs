use crate::engine::configs::FutilityConfig;


#[inline]
pub fn calculate_fut_margin(depth: u16, history_score: i32, config: &FutilityConfig) -> i32 {
    let mut margin = depth * config.margin;

    if config.history_enabled {
        if history_score > config.good_history {
            margin += depth * config.history_margin;
        } else if history_score < config.bad_history {
            margin -= depth * config.history_margin;
        }
    }

    margin as i32
}
