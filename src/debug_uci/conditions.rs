//! Optional completed-iteration stopping conditions. This module accepts legal
//! UCI moves and numeric weights; it does not load EPD files or manage suites.
use crate::board::Board;
use crate::types::{Color, PieceType};
use crate::uci::parse_uci_move;
use serde::Deserialize;
use std::collections::BTreeMap;

#[derive(Clone, Deserialize)]
pub(super) struct MoveWeight {
    #[serde(rename = "move")]
    mv: String,
    points: i32,
}

#[derive(Clone, Deserialize)]
pub(super) struct ExpectedAnswer {
    pub fen: String,
    #[serde(default)]
    acceptable_moves: Vec<String>,
    #[serde(default)]
    avoided_moves: Vec<String>,
    #[serde(default)]
    move_scores: Vec<MoveWeight>,
}

impl ExpectedAnswer {
    pub fn validate(&self) -> Result<(), String> {
        let board = Board::from_fen(&self.fen)?;
        for side in [Color::White, Color::Black] {
            if board.pieces(side, PieceType::King).count_ones() != 1 {
                return Err("search conditions require one king of each color".into());
            }
        }
        for mv in self
            .acceptable_moves
            .iter()
            .chain(&self.avoided_moves)
            .chain(self.move_scores.iter().map(|w| &w.mv))
        {
            parse_uci_move(&board, mv)?;
        }
        if self
            .acceptable_moves
            .iter()
            .any(|m| self.avoided_moves.contains(m))
        {
            return Err("the same move cannot be accepted and avoided".into());
        }
        Ok(())
    }

    fn weights(&self) -> Option<BTreeMap<String, i32>> {
        let mut weights = BTreeMap::new();
        for entry in &self.move_scores {
            if entry.points < 0 {
                return None;
            }
            if let Some(previous) = weights.insert(entry.mv.to_ascii_lowercase(), entry.points) {
                if previous != entry.points {
                    return None;
                }
            }
        }
        if !weights.is_empty() && weights.values().all(|&p| p <= 0) {
            return None;
        }
        Some(weights)
    }

    pub fn scorable(&self) -> bool {
        (!self.acceptable_moves.is_empty()
            || !self.avoided_moves.is_empty()
            || !self.move_scores.is_empty())
            && self.weights().is_some()
    }

    pub fn is_passing(&self, best_move: Option<&str>, completed_depth: u16) -> bool {
        if completed_depth == 0 || !self.scorable() {
            return false;
        }
        let Some(best) = best_move else {
            return false;
        };
        if self
            .avoided_moves
            .iter()
            .any(|m| m.eq_ignore_ascii_case(best))
        {
            return false;
        }
        if !self.acceptable_moves.is_empty()
            && !self
                .acceptable_moves
                .iter()
                .any(|m| m.eq_ignore_ascii_case(best))
        {
            return false;
        }
        let Some(weights) = self.weights() else {
            return false;
        };
        weights.is_empty()
            || weights.get(&best.to_ascii_lowercase()).copied() == weights.values().max().copied()
    }
}
