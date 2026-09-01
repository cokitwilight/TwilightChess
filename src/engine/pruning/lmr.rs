use crate::engine::MAX_PLY;
use crate::engine::configs::LMRConfig;

const MAX_MOVES: usize = 128;

pub const LMR_SCALE: f64 = 256.0;

pub const LMR_SCALE_I32: i32 = 256;

#[derive(Clone, Debug)]
pub struct LmrTable {
    reductions: [[i16; MAX_MOVES]; MAX_PLY],
}

impl LmrTable {
    pub fn new(config: &LMRConfig) -> Self {
        let mut reductions = [[0i16; MAX_MOVES]; MAX_PLY];

        for depth in 1..MAX_PLY {
            for mv in 1..MAX_MOVES {
                let d = depth as f64;
                let m = mv as f64;

                let reduction = config.base + d.ln() * m.ln() / config.divisor;

                reductions[depth][mv] = (reduction.max(0.0) * LMR_SCALE) as i16;
            }
        }
        Self { reductions }
    }

    #[inline]
    pub fn get(&self, depth: usize, mv: usize) -> i32 {
        self.reductions[depth.min(MAX_PLY - 1)][mv.min(MAX_MOVES - 1)] as i32
    }
}
