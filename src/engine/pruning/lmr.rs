use crate::engine::configs::LMRConfig;
use crate::engine::{MAX_PLY, SearchContext};
use crate::types::PieceType;

const MAX_MOVES: usize = 128;

pub const LMR_SCALE: f64 = 256.0;

pub const LMR_SCALE_I32: i32 = 256;

pub const LMR_CAPTURE_VALUE: [i32; 6] = [51, 153, 166, 230, 512, 0];

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

                let reduction = config.base + (d.ln() * m.ln() / config.divisor);

                reductions[depth][mv] = (reduction.max(0.0) * LMR_SCALE) as i16;
            }
        }
        Self { reductions }
    }

    #[inline]
    pub fn get(&self, depth: usize, mv: usize) -> i32 {
        self.reductions[depth.min(MAX_PLY - 1)][mv.min(MAX_MOVES - 1)] as i32
    }

    #[inline(always)]
    pub fn calculate_reduction(
        &self,
        depth: usize,
        searched_moves: usize,
        is_quiet: bool,
        is_capture: bool,
        is_improving: bool,
        is_pv: bool,
        history_score: i32,
        captured_piece: Option<PieceType>,
        gives_check: bool,
        config: &LMRConfig,
    ) -> i32 {
        let base_reduction = self.get(depth, searched_moves + 1);
        let mut current_reduction = base_reduction;

        if config.history_enabled {
            let history_adjustment = if is_quiet {
                history_score / config.history_scale
            } else if is_capture {
                history_score * 2 / config.history_scale
            } else {
                0
            };
            current_reduction += -history_adjustment;
        }

        if !is_improving {
            current_reduction += base_reduction / 3;
        }

        if is_pv {
            current_reduction += -(base_reduction / 3);
        }

        if is_capture {
            current_reduction +=
                -(LMR_CAPTURE_VALUE[captured_piece.unwrap().idx()] + LMR_SCALE_I32 / 2);
        }

        if gives_check {
            current_reduction += -(LMR_SCALE_I32 / 2);
        }

        let reduction_in_plies =
            |scaled_reduction: i32| (scaled_reduction / LMR_SCALE_I32).clamp(0, depth as i32 - 2);
        reduction_in_plies(current_reduction)
    }


    #[inline(always)]
    pub fn calculate_reduction_with_stats(
        &self,
        context: &mut SearchContext,
        depth: usize,
        searched_moves: usize,
        is_quiet: bool,
        is_capture: bool,
        is_improving: bool,
        is_pv: bool,
        history_score: i32,
        captured_piece: Option<PieceType>,
        gives_check: bool,
        config: &LMRConfig,
    ) -> i32 {
        let base_reduction = self.get(depth as usize, searched_moves as usize + 1);
        let mut current_reduction = base_reduction;
        let mut history_delta = 0;
        let mut non_improving_delta = 0;
        let mut pv_delta = 0;
        let mut capture_delta = 0;
        let mut check_delta = 0;

        if config.history_enabled {
            let baseline_red = current_reduction / LMR_SCALE_I32;

            let history_adjustement = if is_quiet {
                history_score / config.history_scale
            } else if is_capture {
                // Capture history is less than max quiet history.
                history_score * 2 / config.history_scale
            } else {
                0
            };

            history_delta = -history_adjustement;
            current_reduction += history_delta;

            if history_delta != 0 {
                context.stats.lmr_stats.history_adjustments += 1;
            }

            let history_red = current_reduction / LMR_SCALE_I32;

            if history_red < baseline_red {
                context.stats.lmr_stats.history_improvements += 1;
            } else if history_red > baseline_red {
                context.stats.lmr_stats.history_reductions += 1;
            }
        }

        if !is_improving {
            non_improving_delta = base_reduction / 3;
            current_reduction += non_improving_delta;
            if non_improving_delta != 0 {
                context.stats.lmr_stats.non_improving_adjustments += 1;
            }
        }

        if is_pv {
            pv_delta = -(base_reduction / 3);
            current_reduction += pv_delta;
            if pv_delta != 0 {
                context.stats.lmr_stats.pv_adjustments += 1;
            }
        }

        if is_capture {
            capture_delta = -(LMR_CAPTURE_VALUE[captured_piece.unwrap().idx()] + LMR_SCALE_I32 / 2);
            current_reduction += capture_delta;
            context.stats.lmr_stats.capture_adjustments += 1;
        }

        if gives_check {
            check_delta = -(LMR_SCALE_I32 / 2);
            current_reduction += check_delta;
            context.stats.lmr_stats.check_adjustments += 1;
        }

        let reduction_in_plies =
            |scaled_reduction: i32| (scaled_reduction / LMR_SCALE_I32).clamp(0, depth as i32 - 2);
        let red = reduction_in_plies(current_reduction);

        if history_delta != 0 && reduction_in_plies(current_reduction - history_delta) != red {
            context.stats.lmr_stats.history_effects += 1;
        }
        if non_improving_delta != 0
            && reduction_in_plies(current_reduction - non_improving_delta) != red
        {
            context.stats.lmr_stats.non_improving_effects += 1;
        }
        if pv_delta != 0 && reduction_in_plies(current_reduction - pv_delta) != red {
            context.stats.lmr_stats.pv_effects += 1;
        }
        if capture_delta != 0 && reduction_in_plies(current_reduction - capture_delta) != red {
            context.stats.lmr_stats.capture_effects += 1;
        }
        if check_delta != 0 && reduction_in_plies(current_reduction - check_delta) != red {
            context.stats.lmr_stats.check_effects += 1;
        }

        red
    }
}
