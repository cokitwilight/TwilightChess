use crate::engine::configs::LMRConfig;
use crate::engine::search_stats::track_stat;
use crate::engine::{MAX_PLY, SearchContext};
use crate::types::PieceType;

const MAX_MOVES: usize = 128;

pub const LMR_SCALE: f64 = 256.0;

pub const LMR_SCALE_I32: i32 = 256;

pub const LMR_CAPTURE_VALUE: [i32; 6] = [51, 153, 166, 230, 512, 0];

#[inline]
pub fn calculate_lmr_research_depth(
    depth: u16,
    full_child_depth: u16,
    reduced_child_depth: u16,
    eval: i32,
    best_eval: i32,
) -> u16 {
    let deeper_search = reduced_child_depth < depth && eval > best_eval + 50;
    let shallow_search = eval < best_eval + 10;

    if deeper_search {
        full_child_depth + 1
    } else if shallow_search {
        full_child_depth - 1
    } else {
        full_child_depth
    }
}

#[derive(Clone, Debug)]
pub struct LmrTable {
    reductions: [[i16; MAX_MOVES]; MAX_PLY],
}

impl LmrTable {
    pub fn new(config: &LMRConfig) -> Self {
        let mut reductions = [[0i16; MAX_MOVES]; MAX_PLY];

        for (depth, row) in reductions.iter_mut().enumerate().skip(1) {
            for (mv, entry) in row.iter_mut().enumerate().skip(1) {
                let d = depth as f64;
                let m = mv as f64;

                let reduction = config.base + (d.ln() * m.ln() / config.divisor);

                *entry = (reduction.max(0.0) * LMR_SCALE) as i16;
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
        _context: &mut SearchContext,
        depth: usize,
        searched_moves: usize,
        is_quiet: bool,
        is_capture: bool,
        is_improving: bool,
        is_pv: bool,
        cut_node: bool,
        history_score: i32,
        captured_piece: Option<PieceType>,
        gives_check: bool,
        config: &LMRConfig,
    ) -> i32 {
        let base_reduction = self.get(depth, searched_moves + 1);
        let mut current_reduction = base_reduction;

        let history_delta = if config.history_enabled {
            let history_adjustment = if is_quiet {
                history_score / config.history_scale
            } else if is_capture {
                // Capture history is less than max quiet history.
                history_score * 2 / config.history_scale
            } else {
                0
            };
            -history_adjustment
        } else {
            0
        };
        current_reduction += history_delta;

        track_stat! {
            if config.history_enabled {
                if history_delta != 0 {
                    _context.stats.lmr_stats.history_adjustments += 1;
                }

                let baseline_red = base_reduction / LMR_SCALE_I32;
                let history_red = current_reduction / LMR_SCALE_I32;
                if history_red < baseline_red {
                    _context.stats.lmr_stats.history_improvements += 1;
                } else if history_red > baseline_red {
                    _context.stats.lmr_stats.history_reductions += 1;
                }
            }
        }

        let non_improving_delta = if !is_improving { base_reduction / 3 } else { 0 };
        current_reduction += non_improving_delta;
        track_stat! {
            if non_improving_delta != 0 {
                _context.stats.lmr_stats.non_improving_adjustments += 1;
            }
        }

        let cut_node_delta = if cut_node { base_reduction / 2 } else { 0 };
        current_reduction += cut_node_delta;
        track_stat! {
            if cut_node_delta != 0 {
                _context.stats.lmr_stats.cut_node_adjustments += 1;
            }
        }

        let pv_delta = if is_pv { -(base_reduction / 3) } else { 0 };
        current_reduction += pv_delta;
        track_stat! {
            if pv_delta != 0 {
                _context.stats.lmr_stats.pv_adjustments += 1;
            }
        }

        let capture_delta = if is_capture {
            -(LMR_CAPTURE_VALUE[captured_piece.unwrap().idx()] + LMR_SCALE_I32 / 2)
        } else {
            0
        };
        current_reduction += capture_delta;
        track_stat! {
            if is_capture {
                _context.stats.lmr_stats.capture_adjustments += 1;
            }
        }

        let check_delta = if gives_check { -(LMR_SCALE_I32 / 2) } else { 0 };
        current_reduction += check_delta;
        track_stat! {
            if gives_check {
                _context.stats.lmr_stats.check_adjustments += 1;
            }
        }

        let reduction_in_plies =
            |scaled_reduction: i32| (scaled_reduction / LMR_SCALE_I32).clamp(0, depth as i32 - 2);
        let red = reduction_in_plies(current_reduction);

        track_stat! {
            if history_delta != 0 && reduction_in_plies(current_reduction - history_delta) != red {
                _context.stats.lmr_stats.history_effects += 1;
            }
            if non_improving_delta != 0
                && reduction_in_plies(current_reduction - non_improving_delta) != red
            {
                _context.stats.lmr_stats.non_improving_effects += 1;
            }
            if pv_delta != 0 && reduction_in_plies(current_reduction - pv_delta) != red {
                _context.stats.lmr_stats.pv_effects += 1;
            }
            if cut_node_delta != 0
                && reduction_in_plies(current_reduction - cut_node_delta) != red
            {
                _context.stats.lmr_stats.cut_node_effects += 1;
            }
            if capture_delta != 0 && reduction_in_plies(current_reduction - capture_delta) != red {
                _context.stats.lmr_stats.capture_effects += 1;
            }
            if check_delta != 0 && reduction_in_plies(current_reduction - check_delta) != red {
                _context.stats.lmr_stats.check_effects += 1;
            }
        }

        red
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::SearchLimits;

    #[test]
    fn cut_node_modifier_counts_only_changes_to_the_final_clamped_reduction() {
        let config = LMRConfig::default();
        let mut table = LmrTable::new(&config);
        let mut context = SearchContext::new(SearchLimits::depth(8, 4), vec![]);
        for (depth, cut_node, base, expected) in [
            (8, false, 512, 2),
            (8, true, 512, 3),
            (3, true, 512, 1), // The upper clamp absorbs the adjustment.
            (8, true, 0, 0),
        ] {
            table.reductions[depth][4] = base;
            assert_eq!(
                table.calculate_reduction(
                    &mut context,
                    depth,
                    3,
                    true,
                    false,
                    true,
                    false,
                    cut_node,
                    0,
                    None,
                    false,
                    &config,
                ),
                expected,
            );
        }
        let stats = context.stats.lmr_stats;
        let enabled = u64::from(cfg!(feature = "search-stats"));
        assert_eq!(stats.cut_node_adjustments, 2 * enabled);
        assert_eq!(stats.cut_node_effects, enabled);

        let mut combined = crate::engine::search_stats::LmrStats::default();
        combined += stats;
        combined += stats;
        let delta = combined - stats;
        assert_eq!(delta.cut_node_adjustments, stats.cut_node_adjustments);
        assert_eq!(delta.cut_node_effects, stats.cut_node_effects);
    }
}
