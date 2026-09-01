use crate::board::{Board, Move, MoveList, MoveType};
use crate::engine::history::{HistoryKey, HistoryTables};
use crate::engine::ordering::{move_order_score, promotion_score, see};
use crate::engine::{Engine, PickerFrame, SearchContext};
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
const STAGE_CAPACITIES: [usize; STAGE_COUNT] = [2, 10, 2, 2, 15, 15, 6, 15];

#[derive(Clone, Copy, Debug)]
pub struct ScoredMove {
    pub mv: Move,
    pub score: i32,
    // reuse any computed data
    pub history_key: Option<HistoryKey>,
    pub see: Option<i32>,
    pub captured: Option<PieceType>,
}

impl ScoredMove {
    pub fn new() -> Self {
        Self {
            mv: Move::NULL,
            score: 0,
            history_key: None,
            see: None,
            captured: None,
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct StagedMoveBuffer {
    // Highest scores are kept at the end when a stage is sorted, so retrieval is a pop.
    staged: [Vec<ScoredMove>; STAGE_COUNT],
}

impl StagedMoveBuffer {
    pub(crate) fn new() -> Self {
        Self {
            staged: std::array::from_fn(|stage| Vec::with_capacity(STAGE_CAPACITIES[stage])),
        }
    }

    pub(crate) fn clear(&mut self) {
        for stage in &mut self.staged {
            stage.clear();
        }
    }

    fn stage(&self, stage: Stage) -> &Vec<ScoredMove> {
        &self.staged[stage as usize]
    }

    fn stage_mut(&mut self, stage: Stage) -> &mut Vec<ScoredMove> {
        &mut self.staged[stage as usize]
    }
}

#[derive(Clone)]
pub struct StagedMoveSelector {
    buffer_frame: usize,
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
    pub(crate) fn new_staged_move_selector(
        &self,
        mode: u8,
        board: &Board,
        moves: &mut MoveList,
        side: Color,
        ply: usize,
        picker_frame: PickerFrame,
        context: &mut SearchContext,
        pv_move: Option<Move>,
        tt_move: Option<Move>,
    ) -> StagedMoveSelector {
        context.reset_staged_move_buffer(picker_frame);

        let mut current_stage: Stage = Stage::PvTt;

        let mode = match mode {
            1 => Mode::Quiescence,
            _ => Mode::Main,
        };

        if pv_move.is_none() && tt_move.is_none() {
            current_stage = Stage::GoodCaptures;
        }

        let mut move_selector = StagedMoveSelector {
            buffer_frame: picker_frame.index(),
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

            if !context.staged_move_buffers[picker_frame.index()].staged[index].is_empty()
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
        context: &mut SearchContext,
        history: &HistoryTables,
    ) -> Option<ScoredMove> {
        loop {
            if context.staged_move_buffers[self.buffer_frame]
                .stage(self.current_stage)
                .is_empty()
            {
                if self.current_stage == Stage::BadCaptures {
                    // MAX STAGES
                    return None;
                }

                self.next_stage();
                self.recompute_staged(board, context, history);
                continue;
            }

            if self.sorted[self.current_stage as usize] {
                return context.staged_move_buffers[self.buffer_frame]
                    .stage_mut(self.current_stage)
                    .pop();
            }

            let best_index = context.staged_move_buffers[self.buffer_frame]
                .stage(self.current_stage)
                .iter()
                .enumerate()
                .max_by_key(|(_, scored)| scored.score)
                .map(|(index, _)| index)
                .unwrap();

            return Some(
                context.staged_move_buffers[self.buffer_frame]
                    .stage_mut(self.current_stage)
                    .swap_remove(best_index),
            );
        }
    }

    pub fn recompute_staged(
        &mut self,
        board: &Board,
        context: &mut SearchContext,
        history: &HistoryTables,
    ) {
        if !context.staged_move_buffers[self.buffer_frame]
            .stage(self.current_stage)
            .is_empty()
            || self.remaining.is_empty()
        {
            return;
        }
        match self.current_stage {
            Stage::PvTt => {
                let mut i = 0;
                while i < self.remaining.len() {
                    let mv = self.remaining.get(i);
                    if Some(mv) == self.pv_move {
                        context.staged_move_buffers[self.buffer_frame]
                            .stage_mut(self.current_stage)
                            .push(ScoredMove {
                                mv,
                                score: 2_000_000,
                                history_key: None,
                                see: None,
                                captured: None,
                            });
                        self.remaining.swap_remove(i);
                        continue;
                    }
                    if Some(mv) == self.tt_move {
                        context.staged_move_buffers[self.buffer_frame]
                            .stage_mut(self.current_stage)
                            .push(ScoredMove {
                                mv,
                                score: 1_500_000,
                                history_key: None,
                                see: None,
                                captured: None,
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

                    let captured = match mv.kind() {
                        MoveType::EnPassant => PieceType::Pawn,
                        MoveType::Capture => {
                            board.piecetype_at(mv.to()).expect("No piece in board!")
                        }
                        _ => unreachable!("Unreachable!"),
                    };

                    let history_key = {
                        let piece = board
                            .piecetype_at(mv.from())
                            .expect("No piece in board in move ordering!");
                        HistoryKey::new(self.side, piece, mv.to())
                    };

                    let see = see(board, mv, Some(captured));
                    let promo_bonus = mv.promotion().map_or(0, promotion_score);
                    let history_score = history.get_capture_score(history_key, captured);
                    if see >= 0 {
                        context.staged_move_buffers[self.buffer_frame]
                            .stage_mut(Stage::GoodCaptures)
                            .push(ScoredMove {
                                mv,
                                score: 900_000 + see + promo_bonus + history_score,
                                history_key: Some(history_key),
                                see: Some(see),
                                captured: Some(captured),
                            });
                        self.remaining.swap_remove(i);
                        continue;
                    } else {
                        context.staged_move_buffers[self.buffer_frame]
                            .stage_mut(Stage::BadCaptures)
                            .push(ScoredMove {
                                mv,
                                score: -600_000 + see + promo_bonus,
                                history_key: Some(history_key),
                                see: Some(see),
                                captured: Some(captured),
                            });
                        self.remaining.swap_remove(i);
                        continue;
                    }
                }

                for stage in [Stage::GoodCaptures, Stage::BadCaptures] {
                    let index = stage as usize;
                    let moves = context.staged_move_buffers[self.buffer_frame].stage_mut(stage);

                    if moves.len() >= 6 {
                        moves.sort_unstable_by_key(|sm| sm.score);
                        self.sorted[index] = true;
                    }
                }
            }
            Stage::QueenPromotions | Stage::Killers => {
                let mut i = 0;

                while i < self.remaining.len() {
                    let mv = self.remaining.get(i);

                    if context.killer_moves.contains(self.ply, mv) {
                        context.staged_move_buffers[self.buffer_frame]
                            .stage_mut(Stage::Killers)
                            .push(ScoredMove {
                                mv,
                                score: 700_000,
                                history_key: None,
                                see: None,
                                captured: None,
                            });
                        self.remaining.swap_remove(i);
                        continue;
                    }

                    if mv.promotion() == Some(PieceType::Queen) {
                        context.staged_move_buffers[self.buffer_frame]
                            .stage_mut(Stage::QueenPromotions)
                            .push(ScoredMove {
                                mv,
                                score: 900_000,
                                history_key: None,
                                see: None,
                                captured: None,
                            });
                        self.remaining.swap_remove(i);
                        continue;
                    }
                    i += 1;
                }
            }
            _ => {
                if self.remaining.is_empty() {
                    panic!(
                        "Recompute Staged called with self.remaining.len() == 0! Current stage: {:?}",
                        self.current_stage
                    );
                }
                while let Some(mv) = self.remaining.pop() {
                    let is_capture =
                        mv.kind() == MoveType::Capture || mv.kind() == MoveType::EnPassant;

                    let is_underpromotion =
                        mv.promotion().is_some() && Some(PieceType::Queen) != mv.promotion();

                    let history_key = {
                        let piece = board
                            .piecetype_at(mv.from())
                            .expect("No piece in board in move ordering!");
                        Some(HistoryKey::new(self.side, piece, mv.to()))
                    };

                    let captured = if is_capture {
                        match mv.kind() {
                            MoveType::Capture => board.piecetype_at(mv.to()),
                            MoveType::EnPassant => Some(PieceType::Pawn),
                            _ => unreachable!("Unreachable Code in staged move selector!"),
                        }
                    } else {
                        None
                    };
                    let see = if is_capture {
                        Some(see(board, mv, captured))
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
                        captured,
                    );

                    let scored_move = ScoredMove {
                        mv,
                        score,
                        history_key,
                        see,
                        captured,
                    };

                    let stage = if is_underpromotion {
                        Stage::Underpromotions
                    } else if score > 0 {
                        Stage::GoodQuiets
                    } else {
                        Stage::BadQuiets
                    };

                    context.staged_move_buffers[self.buffer_frame]
                        .stage_mut(stage)
                        .push(scored_move);
                }
                for stage in [
                    Stage::GoodQuiets,
                    Stage::BadQuiets,
                    Stage::Underpromotions,
                    Stage::BadCaptures,
                ] {
                    let index = stage as usize;
                    let moves = context.staged_move_buffers[self.buffer_frame].stage_mut(stage);
                    if moves.len() >= 6 {
                        moves.sort_unstable_by_key(|sm| sm.score);
                        self.sorted[index] = true;
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::{Board, STARTPOS_FEN};
    use crate::engine::configs::EngineConfig;
    use crate::engine::{MAX_PLY, SearchLimits};

    fn test_engine() -> Engine {
        let mut config = EngineConfig::default();
        config.tt_size = 1;
        Engine::new(config)
    }

    fn assert_contains_each_move_once(expected: &MoveList, actual: &[Move]) {
        assert_eq!(actual.len(), expected.len());
        for expected_move in expected.iter() {
            assert_eq!(
                actual
                    .iter()
                    .filter(|actual_move| *actual_move == expected_move)
                    .count(),
                1,
                "move picker must return {expected_move:?} exactly once"
            );
        }
    }

    #[test]
    fn search_context_preallocates_and_reuses_stage_buffers() {
        let mut context = SearchContext::new(SearchLimits::depth(1, 1), Vec::new());

        assert_eq!(context.staged_move_buffers.len(), MAX_PLY + 1);

        for (stage, expected_capacity) in STAGE_CAPACITIES.into_iter().enumerate() {
            assert!(context.staged_move_buffers[0].staged[stage].capacity() >= expected_capacity);
            context.staged_move_buffers[0].staged[stage].push(ScoredMove::new());
        }

        let allocations: Vec<_> = context.staged_move_buffers[0]
            .staged
            .iter()
            .map(|stage| (stage.as_ptr(), stage.capacity()))
            .collect();

        context.reset_staged_move_buffer(PickerFrame::ROOT);

        for (stage, (allocation, capacity)) in context.staged_move_buffers[0]
            .staged
            .iter()
            .zip(allocations)
        {
            assert!(stage.is_empty());
            assert_eq!(stage.as_ptr(), allocation);
            assert_eq!(stage.capacity(), capacity);
        }
    }

    #[test]
    fn nested_same_ply_picker_uses_a_different_frame() {
        let engine = test_engine();
        let mut board = Board::from_fen(STARTPOS_FEN).expect("valid starting position");
        let expected = board.all_legal_moves();
        let mut context = SearchContext::new(SearchLimits::depth(1, 1), vec![board.hash()]);

        let mut parent_moves = expected;
        let mut parent = engine.new_staged_move_selector(
            0,
            &board,
            &mut parent_moves,
            board.side_to_move(),
            0,
            PickerFrame::ROOT,
            &mut context,
            None,
            None,
        );

        let mut parent_result = vec![
            parent
                .get_next(&board, &mut context, &engine.history)
                .expect("starting position has a move")
                .mv,
        ];

        // This models singular verification: the chess ply is unchanged, but the
        // nested search receives the next picker frame and must not reset its parent.
        let mut nested_moves = expected;
        let mut nested = engine.new_staged_move_selector(
            0,
            &board,
            &mut nested_moves,
            board.side_to_move(),
            0,
            PickerFrame::ROOT.child(),
            &mut context,
            None,
            None,
        );
        let mut nested_result = Vec::new();
        while let Some(scored_move) = nested.get_next(&board, &mut context, &engine.history) {
            nested_result.push(scored_move.mv);
        }

        while let Some(scored_move) = parent.get_next(&board, &mut context, &engine.history) {
            parent_result.push(scored_move.mv);
        }

        assert_contains_each_move_once(&expected, &nested_result);
        assert_contains_each_move_once(&expected, &parent_result);
    }
}
