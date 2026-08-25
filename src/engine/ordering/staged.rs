use crate::board::{Board, Move, MoveList, MoveType};
use crate::engine::history::{HistoryKey, HistoryTables};
use crate::engine::ordering::{
    ordering::{move_order_score, promotion_score},
    see,
};
use crate::engine::{Engine, SearchContext};
use crate::types::{Color, PieceType};

const MAIN_STAGES: &[Stage; 8] = &[
    Stage::PvTt,
    Stage::GoodCaptures,
    Stage::QueenPromotions,
    Stage::Killers,
    Stage::GoodQuiets,
    Stage::BadQuiets,
    Stage::Underpromotions,
    Stage::BadCaptures,
];

const QSEARCH_STAGES: &[Stage] = &[
    Stage::PvTt,
    Stage::GoodCaptures,
    Stage::QueenPromotions,
    Stage::BadCaptures,
];

// stages go:
// 1: +1,500,000 = pv move and tt move
// 2: +900,000 = winning/equal captures
// 3: +800,000 = quiet queen promotions
// 4: +700_000 = killer moves
// 5: +0 = good/neutral history quiet moves
// 6: 0 <-> -49,999 = bad history quiet moves  // capped at -HISTORY MAX(currently ~48 000)
// 7: + -500,000 = quiet under promotions(maybe change this)
// 8: + -600,000 = losing captures

#[repr(usize)]
#[derive(Clone, Copy, Debug, PartialEq)]
enum Stage {
    PvTt = 0,
    GoodCaptures,
    QueenPromotions,
    Killers,
    GoodQuiets,
    BadQuiets,
    Underpromotions,
    BadCaptures,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Mode {
    Main,
    Quiescence,
}

const STAGE_COUNT: usize = 8;

#[derive(Clone, Copy, Debug)]
pub struct ScoredMove {
    pub mv: Move,
    pub score: i32,
    // reuse any computed data
    pub history_key: Option<HistoryKey>,
    pub see: Option<i32>,
}

#[derive(Clone)]
pub struct StagedMoveSelector {
    // since these are Vec try to sort with highest at the end so removing is just .pop()
    staged: [Vec<ScoredMove>; STAGE_COUNT],
    sorted: [bool; STAGE_COUNT],
    remaining: MoveList,
    current_stage: Stage, // the stage number(i.e 1 -> tt moves, 2-> winning captures)

    mode: Mode,

    // static parent info
    side: Color,
    ply: usize,
    pv_move: Option<Move>,
    tt_move: Option<Move>,
}

impl Engine {
    pub fn new_staged_move_selecter(
        &self,
        mode: u8,
        board: &Board,
        moves: &mut MoveList,
        side: Color,
        ply: usize,
        context: &SearchContext,
        pv_move: Option<Move>,
        tt_move: Option<Move>,
    ) -> StagedMoveSelector {
        let staged = [
            Vec::with_capacity(2),
            Vec::with_capacity(10),
            Vec::with_capacity(2),
            Vec::with_capacity(2),
            Vec::with_capacity(15),
            Vec::with_capacity(15),
            Vec::with_capacity(6),
            Vec::with_capacity(15),
        ];
        let mut current_stage: Stage = Stage::PvTt;

        let mode = match mode {
            1 => Mode::Quiescence,
            _ => Mode::Main,
        };

        if pv_move.is_none() && tt_move.is_none() {
            current_stage = Stage::GoodCaptures;
        }

        let mut move_selector = StagedMoveSelector {
            staged,
            sorted: [false; 8],
            current_stage,
            mode,
            remaining: *moves,
            side,
            ply,
            pv_move,
            tt_move,
        };

        if move_selector.remaining.is_empty() {
            move_selector.current_stage = Stage::BadCaptures;
            return move_selector;
        }

        loop {
            // only build stage 1(or earliest possible one)

            move_selector.recompute_staged(board, context, &self.history);

            let index = move_selector.current_stage as usize;

            if !move_selector.staged[index].is_empty()
                || move_selector.current_stage == Stage::BadCaptures
            {
                break;
            }

            move_selector.next_stage();
        }

        move_selector
    }
}

impl StagedMoveSelector {
    pub fn next_stage(&mut self) {
        if self.current_stage == Stage::BadCaptures {
            panic!("Invalid call of next stage at last stage!");
        }
        match self.mode {
            Mode::Main => self.current_stage = MAIN_STAGES[self.current_stage as usize + 1],
            Mode::Quiescence => {
                self.current_stage = QSEARCH_STAGES[self.current_stage as usize + 1]
            }
        }
    }
    pub fn get_next(
        &mut self,
        board: &Board,
        context: &SearchContext,
        history: &HistoryTables,
    ) -> Option<ScoredMove> {
        loop {
            if self.staged[self.current_stage as usize].is_empty() {
                if self.current_stage == Stage::BadCaptures {
                    // MAX STAGES
                    return None;
                }

                self.next_stage();
                self.recompute_staged(board, context, history);
                continue;
            }

            if self.sorted[self.current_stage as usize] {
                return self.staged[self.current_stage as usize].pop();
            }

            let best_index = self.staged[self.current_stage as usize]
                .iter()
                .enumerate()
                .max_by_key(|(_, scored)| scored.score)
                .map(|(index, _)| index)
                .unwrap();

            return Some(self.staged[self.current_stage as usize].swap_remove(best_index));
        }
    }

    pub fn recompute_staged(
        &mut self,
        board: &Board,
        context: &SearchContext,
        history: &HistoryTables,
    ) {
        if !self.staged[self.current_stage as usize].is_empty() || self.remaining.is_empty() {
            return;
        }
        match self.current_stage {
            Stage::PvTt => {
                let mut i = 0;
                while i < self.remaining.len() {
                    let mv = self.remaining.get(i);
                    if Some(mv) == self.pv_move {
                        self.staged[self.current_stage as usize].push(ScoredMove {
                            mv: mv,
                            score: 2_000_000,
                            history_key: None,
                            see: None,
                        });
                        self.remaining.swap_remove(i);
                        continue;
                    }
                    if Some(mv) == self.tt_move {
                        self.staged[self.current_stage as usize].push(ScoredMove {
                            mv: mv,
                            score: 1_500_000,
                            history_key: None,
                            see: None,
                        });
                        self.remaining.swap_remove(i);
                        continue;
                    }
                    i += 1;
                }
            }
            Stage::GoodCaptures => {
                let mut i = 0;

                while i < self.remaining.len() {
                    let mv = self.remaining.get(i);
                    if mv.kind() != MoveType::Capture && mv.kind() != MoveType::EnPassant {
                        i += 1;
                        continue;
                    }

                    let see = see(board, mv);
                    let promo_bonus = mv.promotion().map_or(0, promotion_score);
                    if see >= 0 {
                        self.staged[Stage::GoodCaptures as usize].push(ScoredMove {
                            mv,
                            score: 900_000 + see + promo_bonus,
                            history_key: None,
                            see: Some(see),
                        });
                        self.remaining.swap_remove(i);
                        continue;
                    } else {
                        self.staged[Stage::BadCaptures as usize].push(ScoredMove {
                            mv,
                            score: -600_000 + see + promo_bonus,
                            history_key: None,
                            see: Some(see),
                        });
                        self.remaining.swap_remove(i);
                        continue;
                    }
                }

                for stage in [Stage::GoodCaptures, Stage::BadCaptures] {
                    let index = stage as usize;

                    if self.staged[index].len() >= 6 {
                        self.staged[index].sort_unstable_by_key(|sm| sm.score);
                        self.sorted[index] = true;
                    }
                }
            }
            Stage::QueenPromotions | Stage::Killers => {
                let mut i = 0;

                while i < self.remaining.len() {
                    let mv = self.remaining.get(i);

                    if context.killer_moves.contains(self.ply, mv) {
                        self.staged[Stage::Killers as usize].push(ScoredMove {
                            mv,
                            score: 700_000,
                            history_key: None,
                            see: None,
                        });
                        self.remaining.swap_remove(i);
                        continue;
                    }

                    if mv.promotion() == Some(PieceType::Queen) {
                        self.staged[Stage::QueenPromotions as usize].push(ScoredMove {
                            mv,
                            score: 900_000,
                            history_key: None,
                            see: None,
                        });
                        self.remaining.swap_remove(i);
                        continue;
                    }
                    i += 1;
                }
            }
            _ => {
                if self.remaining.len() == 0 {
                    panic!(
                        "Recompute Staged called with self.remaining.len() == 0! Current stage: {:?}",
                        self.current_stage
                    );
                }
                while let Some(mv) = self.remaining.pop() {
                    let is_capture =
                        mv.kind() == MoveType::Capture || mv.kind() == MoveType::EnPassant;

                    let is_quiet = !is_capture && mv.promotion().is_none();

                    let is_underpromotion =
                        mv.promotion().is_some() && Some(PieceType::Queen) != mv.promotion();

                    let history_key = if is_quiet {
                        let piece = board
                            .piecetype_at(mv.from())
                            .expect("No piece in board in move ordering!");
                        Some(HistoryKey::new(self.side, piece, mv.to()))
                    } else {
                        None
                    };

                    let see = if is_capture {
                        Some(see(board, mv))
                    } else {
                        None
                    };

                    let score = move_order_score(
                        board,
                        mv,
                        self.side,
                        self.ply,
                        context,
                        history,
                        self.pv_move,
                        self.tt_move,
                        history_key,
                        see,
                    );

                    let scored_move = ScoredMove {
                        mv,
                        score,
                        history_key,
                        see,
                    };

                    let stage = if is_underpromotion {
                        Stage::Underpromotions
                    } else if score > 0 {
                        Stage::GoodQuiets
                    } else {
                        Stage::BadQuiets
                    };

                    self.staged[stage as usize].push(scored_move);
                }
                for stage in [
                    Stage::GoodQuiets,
                    Stage::BadQuiets,
                    Stage::Underpromotions,
                    Stage::BadCaptures,
                ] {
                    let index = stage as usize;
                    if self.staged[index].len() >= 6 {
                        self.staged[index].sort_unstable_by_key(|sm| sm.score);
                        self.sorted[index] = true;
                    }
                }
            }
        }
    }
}
