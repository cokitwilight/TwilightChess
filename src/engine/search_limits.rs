#[derive(Clone, Copy, Debug)]
pub struct SearchLimits {
    pub max_depth: usize,
    pub max_q_depth: usize,
    pub max_nodes: Option<u64>,
    pub soft_time_limit_ms: Option<u64>,
    pub hard_time_limit_ms: Option<u64>,
}

impl SearchLimits {
    pub fn depth(max_depth: usize, max_q_depth: usize) -> Self {
        Self {
            max_depth,
            max_q_depth,
            max_nodes: None,
            soft_time_limit_ms: None,
            hard_time_limit_ms: None,
        }
    }

    pub fn depth_and_time(max_depth: usize, max_q_depth: usize, time_limit_ms: u64) -> Self {
        Self {
            max_depth,
            max_q_depth,
            max_nodes: None,
            soft_time_limit_ms: Some(time_limit_ms),
            hard_time_limit_ms: Some(time_limit_ms + 100),
        }
    }
}
