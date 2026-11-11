//! Search metrics exposed by debug UCI.
use crate::engine::{CHECKMATE_SCORE, MATE_THRESHOLD, SearchResult};
use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Score {
    Centipawns { value: i32 },
    Mate { plies: u32, winning: bool },
}

impl Score {
    fn from_raw(raw: i32) -> Self {
        if raw.abs() >= MATE_THRESHOLD {
            Self::Mate {
                plies: (CHECKMATE_SCORE - raw.abs()).max(0) as u32,
                winning: raw > 0,
            }
        } else {
            Self::Centipawns { value: raw }
        }
    }
}

/// All counters and elapsed times are cumulative from the start of this search,
/// including earlier iterative-deepening iterations and aspiration retries.
#[derive(Debug, Serialize)]
pub struct SearchMetrics {
    pub completed_depth: u16,
    pub seldepth: usize,
    pub main_nodes: u64,
    pub quiescence_nodes: u64,
    pub nodes: u64,
    pub elapsed_ns: u64,
    pub elapsed_ms: f64,
    pub nps: f64,
    /// None if no iteration completed; a fallback move has no searched score.
    pub raw_eval: Option<i32>,
    pub score: Option<Score>,
    pub best_move: Option<String>,
    pub pv: Vec<String>,
}

impl SearchMetrics {
    pub(crate) fn unsearched() -> Self {
        Self {
            completed_depth: 0,
            seldepth: 0,
            main_nodes: 0,
            quiescence_nodes: 0,
            nodes: 0,
            elapsed_ns: 0,
            elapsed_ms: 0.0,
            nps: 0.0,
            raw_eval: None,
            score: None,
            best_move: None,
            pv: Vec::new(),
        }
    }
    pub(crate) fn from_result(result: &SearchResult) -> Self {
        let nodes = result.stats.total_nodes();
        let raw_eval = (result.depth_reached > 0).then_some(result.eval);
        Self {
            completed_depth: result.depth_reached,
            seldepth: result.seldepth,
            main_nodes: result.stats.node_stats.main,
            quiescence_nodes: result.stats.node_stats.quiescence,
            nodes,
            elapsed_ns: result.elapsed.as_nanos().min(u64::MAX as u128) as u64,
            elapsed_ms: result.elapsed.as_secs_f64() * 1000.0,
            nps: if result.elapsed.is_zero() {
                0.0
            } else {
                nodes as f64 / result.elapsed.as_secs_f64()
            },
            raw_eval,
            score: raw_eval.map(Score::from_raw),
            best_move: result.best_move.map(|mv| mv.to_uci()),
            pv: result
                .pv
                .iter()
                .take_while(|mv| mv.is_some())
                .flatten()
                .map(|mv| mv.to_uci())
                .collect(),
        }
    }
}
