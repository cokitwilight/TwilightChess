use crate::board::mv::MAX_MOVES;
use crate::board::{Board, Move};
use crate::engine::history::{HistoryKey, HistoryTables};
use crate::engine::staged::verify::verify_move;
use crate::engine::{Engine, PickerFrame, SearchContext};
use crate::moves::MoveGenInfo;
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
pub enum Stage {
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
pub enum Mode {
    Main,
    Quiescence,
}

// const STAGE_COUNT: usize = 8;
// const STAGE_CAPACITIES: [usize; STAGE_COUNT] = [2, 10, 2, 2, 15, 15, 6, 15];

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
pub struct MoveBuffer {
    pub entries: [ScoredMove; MAX_MOVES],
}

impl MoveBuffer {
    pub fn new() -> Self {
        Self {
            entries: [ScoredMove::new(); MAX_MOVES],
        }
    }
}

#[derive(Clone)]
pub struct MovePicker {
    pub stage: Stage,
    pub mode: Mode,

    pub current_len: usize,
    pub current_idx: usize,

    pub bad_quiet_start: usize,
    pub underpromo_start: usize,
    pub bad_capture_start: usize,

    // static parent info
    pub picker: usize,
    pub side: Color,
    pub ply: usize,
    pub pv_move: Option<Move>,
    pub tt_move: Option<Move>,
    pub killer_1: Option<Move>,
    pub killer_2: Option<Move>,
}

impl Engine {
    pub(crate) fn new_move_picker(
        &self,
        mode: u8,
        context: &SearchContext,
        side: Color,
        ply: usize,
        picker: PickerFrame,
        pv_move: Option<Move>,
        tt_move: Option<Move>,
    ) -> MovePicker {
        // context.reset_staged_move_buffer(picker_frame);  We probably dont need a function like this as the move picker controls the stages and state

        let stage = Stage::PvTt;

        let mode = match mode {
            1 => Mode::Quiescence,
            _ => Mode::Main,
        };

        let killer_1 = context.killer_moves.get(ply, 0);
        let killer_2 = context.killer_moves.get(ply, 1);

        let move_picker = MovePicker {
            stage,
            mode,
            current_len: 0,
            current_idx: 0,
            bad_quiet_start: MAX_MOVES,
            underpromo_start: MAX_MOVES,
            bad_capture_start: MAX_MOVES,
            picker: picker.index(),
            side,
            ply,
            pv_move,
            tt_move,
            killer_1,
            killer_2,
        };

        move_picker
    }
}

impl MovePicker {
    pub fn next_stage(
        &mut self,
        board: &Board,
        info: &MoveGenInfo,
        context: &mut SearchContext,
        history: &HistoryTables,
    ) {
        if self.stage == Stage::BadCaptures {
            panic!("Invalid call of next stage at last stage!");
        }
        // reset indexing
        self.current_idx = 0;
        self.current_len = 0; // this gets updated in recompute staged in the corresponding stages where moves are actually computed

        match self.mode {
            Mode::Main => self.stage = MAIN_STAGES[self.stage as usize + 1],
            Mode::Quiescence => self.stage = QSEARCH_STAGES[self.stage as usize + 1],
        }

        self.recompute_staged(board, info, context, history);
    }
    pub fn get_next(
        &mut self,
        board: &Board,
        info: &MoveGenInfo,
        context: &mut SearchContext,
        history: &HistoryTables,
    ) -> Option<ScoredMove> {
        // Array(Move Buffer) is stored as [Good Captures, Queen Promotions, Good Quiets, ... , Bad Quiets, Underpromotions, Bad Captures]
        // As moves are generated the "good" moves get added to the front while "bad" moves get added to the back
        // since each of the "folds" together in the array this works nicely when indexing

        loop {
            match self.stage {
                Stage::PvTt => {
                    // verify legality of pv move and tt move and return them if legal, otherwise go to next stage
                    // if there is both pv move and tt move use the good_len as the order with 0 = pv move and 1 = tt move

                    if self.current_len == 0 {
                        if let Some(pv) = self.pv_move {
                            let valid_move = verify_move(board, pv, info); // IMPLEMENT LATER

                            if valid_move {
                                self.current_len += 1;
                                return Some(ScoredMove {
                                    mv: pv,
                                    score: 1_500_000,
                                    history_key: None,
                                    see: None,
                                    captured: None,
                                });
                            } else {
                                self.current_len += 1;
                            }
                        } else {
                            self.current_len += 1;
                        }
                    } else if self.current_len == 1 {
                        if let Some(tt) = self.tt_move {
                            let valid_move = verify_move(board, tt, info); // IMPLEMENT LATER

                            if valid_move && Some(tt) != self.pv_move {
                                self.current_len += 1;
                                return Some(ScoredMove {
                                    mv: tt,
                                    score: 1_000_000,
                                    history_key: None,
                                    see: None,
                                    captured: None,
                                });
                            } else {
                                self.next_stage(board, info, context, history);
                                continue;
                            }
                        } else {
                            self.next_stage(board, info, context, history);
                            continue;
                        }
                    } else {
                        // for now panic to catch bugs
                        self.next_stage(board, info, context, history);
                        continue;
                    }
                }
                Stage::Killers => {
                    // verify legality of killer 1 and killer 2 and return them if legal, otherwise go to next stage
                    // if there is both pv move and tt move use the good_len as the order with 0 = killer 1 and 1 = killer 2

                    if self.current_len == 0 {
                        if let Some(k1) = self.killer_1 {
                            let valid_move = verify_move(board, k1, info); // IMPLEMENT LATER

                            if valid_move && Some(k1) != self.pv_move && Some(k1) != self.tt_move {
                                self.current_len += 1;
                                return Some(ScoredMove {
                                    mv: k1,
                                    score: 1_500_000,
                                    history_key: None,
                                    see: None,
                                    captured: None,
                                });
                            } else {
                                self.current_len += 1;
                            }
                        } else {
                            self.current_len += 1;
                        }
                    } else if self.current_len == 1 {
                        if let Some(k2) = self.killer_2 {
                            let valid_move = verify_move(board, k2, info); // IMPLEMENT LATER

                            if valid_move && Some(k2) != self.pv_move && Some(k2) != self.tt_move {
                                self.current_len += 1;
                                return Some(ScoredMove {
                                    mv: k2,
                                    score: 1_000_000,
                                    history_key: None,
                                    see: None,
                                    captured: None,
                                });
                            } else {
                                self.next_stage(board, info, context, history);
                                continue;
                            }
                        } else {
                            self.next_stage(board, info, context, history);
                            continue;
                        }
                    } else {
                        // for now panic to catch bugs
                        self.next_stage(board, info, context, history);
                        continue;
                    }
                }
                Stage::BadCaptures => {
                    let move_buffer = &mut context.move_buffers[self.picker].entries;

                    if let Some(entry) =
                        pick_best(move_buffer, &mut self.current_idx, self.current_len)
                    {
                        return Some(entry);
                    } else {
                        return None;
                    }
                }
                _ => {
                    let move_buffer = &mut context.move_buffers[self.picker].entries;

                    if let Some(entry) =
                        pick_best(move_buffer, &mut self.current_idx, self.current_len)
                    {
                        return Some(entry);
                    } else {
                        self.next_stage(board, info, context, history);
                    }
                }
            }
        }
    }

    pub fn recompute_staged(
        &mut self,
        board: &Board,
        info: &MoveGenInfo,
        context: &mut SearchContext,
        history: &HistoryTables,
    ) {
        let move_buffer = &mut context.move_buffers[self.picker].entries;
        match self.stage {
            Stage::PvTt => {
                // try pv move first and if pv move != tt move, then try tt move next
                // this doesn't need to be computed since we already have the Option<Move> for both of them
            }
            Stage::GoodCaptures => {
                // compute captures and put good captures in the front of the list and bad captures in the back of the list
                self.compute_captures(board, info, move_buffer, history);
            }
            Stage::QueenPromotions => {
                // compute only queen promotions
                self.compute_promotions(board, info, move_buffer);
            }
            Stage::Killers => {
                // try killer moves
                // this doesn't need to be computed since we already have the Option<Move> for both of them
            }
            Stage::GoodQuiets => {
                // compute good quiets and put them in the front of the list and bad quiets in the back of the list
                // reuse the original list but remember to reset the length after captures
                self.compute_quiets(board, info, context, history);
            }
            Stage::BadQuiets => {
                // This should already be computed in the previous stage.
                self.current_idx = self.bad_quiet_start;
                self.current_len = self.underpromo_start;
            }
            Stage::Underpromotions => {
                // compute only underpromotions
                self.current_idx = self.underpromo_start;
                self.current_len = self.bad_capture_start;
            }
            Stage::BadCaptures => {
                // This should already be computed in the GoodCaptures stage.
                self.current_idx = self.bad_capture_start;
                self.current_len = MAX_MOVES;
            }
        }
    }
}

fn pick_best(entries: &mut [ScoredMove], idx: &mut usize, end: usize) -> Option<ScoredMove> {
    if *idx >= end {
        return None;
    }

    let mut best = *idx;

    for i in (*idx + 1)..end {
        if entries[i].score > entries[best].score {
            best = i;
        }
    }

    entries.swap(*idx, best);

    let entry = entries[*idx];
    *idx += 1;

    Some(entry)
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;
    use crate::board::{MoveType, STARTPOS_FEN};
    use crate::engine::SearchLimits;
    use crate::engine::configs::EngineConfig;
    use crate::engine::history::HistoryKey;

    const KIWIPETE: &str = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1";

    const TEST_FENS: &[&str] = &[
        STARTPOS_FEN,
        KIWIPETE,
        "r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1",
        "4k3/8/8/3pP3/8/8/8/4K3 w - d6 0 1",
        "k7/8/8/4KPpr/8/8/8/8 w - g6 0 1",
        "k3r3/8/8/8/1b6/8/8/4K3 w - - 0 1",
        "4k2r/6P1/8/8/8/8/8/4K3 w - - 0 1",
        "4k3/8/8/8/8/8/1p6/R3K3 b - - 0 1",
        "7k/6Q1/6K1/8/8/8/8/8 b - - 0 1",
        "7k/5Q2/6K1/8/8/8/8/8 b - - 0 1",
    ];

    type MoveKey = (u8, u8, u8, u8);

    fn move_key(mv: Move) -> MoveKey {
        (
            mv.from(),
            mv.to(),
            mv.kind() as u8,
            mv.promotion().map_or(u8::MAX, |piece| piece as u8),
        )
    }

    fn test_engine() -> Engine {
        let mut config = EngineConfig::standard();
        config.tt_size = 1;
        Engine::new(config)
    }

    fn test_context() -> SearchContext {
        SearchContext::new(SearchLimits::depth(1, 1), Vec::new())
    }

    fn legal_moves(board: &Board) -> Vec<Move> {
        let mut board = board.clone();
        board.all_legal_moves().iter().copied().collect()
    }

    fn collect_staged_moves(
        engine: &Engine,
        board: &Board,
        context: &mut SearchContext,
        pv_move: Option<Move>,
        tt_move: Option<Move>,
    ) -> Vec<(Stage, ScoredMove)> {
        let side = board.side_to_move();
        let info = MoveGenInfo::calculate(board, side);
        let mut picker =
            engine.new_move_picker(0, context, side, 0, PickerFrame::ROOT, pv_move, tt_move);
        let mut moves = Vec::new();

        while let Some(scored_move) = picker.get_next(board, &info, context, &engine.history) {
            moves.push((picker.stage, scored_move));
        }

        moves
    }

    fn assert_matches_legal_moves(
        engine: &Engine,
        board: &Board,
        context: &mut SearchContext,
        pv_move: Option<Move>,
        tt_move: Option<Move>,
    ) {
        let expected = legal_moves(board);
        let actual = collect_staged_moves(engine, board, context, pv_move, tt_move);
        let expected_keys: HashSet<_> = expected.iter().copied().map(move_key).collect();
        let mut actual_keys = HashSet::with_capacity(actual.len());

        for (_, scored_move) in &actual {
            assert!(
                actual_keys.insert(move_key(scored_move.mv)),
                "staged picker returned {:?} more than once at hash {:#x}",
                scored_move.mv,
                board.hash()
            );
        }

        assert_eq!(
            actual.len(),
            expected.len(),
            "staged and legal generators returned different move counts at hash {:#x}",
            board.hash()
        );
        assert_eq!(
            actual_keys,
            expected_keys,
            "staged picker did not match all_legal_moves at hash {:#x}",
            board.hash()
        );
    }

    fn scan_picker_against_legal_moves(
        engine: &Engine,
        board: &mut Board,
        context: &mut SearchContext,
        depth: usize,
    ) {
        assert_matches_legal_moves(engine, board, context, None, None);

        if depth == 0 {
            return;
        }

        let moves = legal_moves(board);
        for mv in moves {
            let undo = board.make_move(mv);
            scan_picker_against_legal_moves(engine, board, context, depth - 1);
            board.undo_move(undo);
            board.assert_hash();
        }
    }

    fn for_each_generated_move(mut check: impl FnMut(&Board, Stage, ScoredMove)) {
        let engine = test_engine();
        let mut context = test_context();

        for fen in TEST_FENS {
            let board = Board::from_fen(fen).expect("valid staged-move test FEN");
            for (stage, scored_move) in
                collect_staged_moves(&engine, &board, &mut context, None, None)
            {
                check(&board, stage, scored_move);
            }
        }
    }

    #[test]
    fn staged_picker_matches_all_legal_moves_without_duplicates() {
        let engine = test_engine();
        let mut context = test_context();

        for fen in TEST_FENS {
            let board = Board::from_fen(fen).expect("valid staged-move test FEN");
            assert_matches_legal_moves(&engine, &board, &mut context, None, None);
        }

        let mut board = Board::from_fen(KIWIPETE).expect("valid Kiwipete FEN");
        scan_picker_against_legal_moves(&engine, &mut board, &mut context, 2);
    }

    #[test]
    fn pv_tt_and_killer_moves_are_still_returned_exactly_once() {
        let engine = test_engine();
        let board = Board::from_fen(KIWIPETE).expect("valid Kiwipete FEN");
        let expected = legal_moves(&board);
        let pv_move = expected
            .iter()
            .copied()
            .find(|mv| !mv.is_capture() && mv.promotion().is_none())
            .expect("Kiwipete has a quiet PV move");
        let tt_move = expected
            .iter()
            .copied()
            .find(|mv| mv.is_capture())
            .expect("Kiwipete has a capture TT move");
        let second_killer = expected
            .iter()
            .copied()
            .find(|mv| {
                !mv.is_capture() && mv.promotion().is_none() && *mv != pv_move && *mv != tt_move
            })
            .expect("Kiwipete has a second quiet move");
        let mut context = test_context();

        context.killer_moves.add(0, second_killer);
        context.killer_moves.add(0, pv_move);

        assert_matches_legal_moves(&engine, &board, &mut context, Some(pv_move), Some(tt_move));
    }

    #[test]
    fn promotion_stages_only_contain_promotions() {
        let mut queen_promotions = 0;
        let mut underpromotions = 0;

        for_each_generated_move(|board, stage, scored_move| match stage {
            Stage::QueenPromotions => {
                queen_promotions += 1;
                assert_eq!(scored_move.mv.promotion(), Some(PieceType::Queen));
                assert_eq!(scored_move.mv.kind(), MoveType::Normal);
                assert_eq!(scored_move.captured, None);
            }
            Stage::Underpromotions => {
                underpromotions += 1;
                let promotion = scored_move
                    .mv
                    .promotion()
                    .expect("underpromotion stage produced a non-promotion");
                assert_ne!(promotion, PieceType::Queen);
                assert_eq!(scored_move.mv.kind(), MoveType::Normal);
                assert_eq!(scored_move.captured, None);
            }
            _ => {
                let _ = board;
            }
        });

        assert!(queen_promotions > 0);
        assert!(underpromotions > 0);
    }

    #[test]
    fn capture_stages_have_capture_type_and_captured_piece_metadata() {
        let mut captures = 0;
        let mut en_passant = 0;

        for_each_generated_move(|board, stage, scored_move| {
            if !matches!(stage, Stage::GoodCaptures | Stage::BadCaptures) {
                assert!(!scored_move.mv.is_capture());
                return;
            }

            captures += 1;
            let expected_captured = match scored_move.mv.kind() {
                MoveType::Capture => board
                    .piecetype_at(scored_move.mv.to())
                    .expect("capture destination must contain an enemy piece"),
                MoveType::EnPassant => {
                    en_passant += 1;
                    PieceType::Pawn
                }
                kind => panic!("capture stage produced {kind:?}"),
            };

            assert_eq!(scored_move.captured, Some(expected_captured));
            assert!(scored_move.see.is_some());
        });

        assert!(captures > 0);
        assert!(en_passant > 0);
    }

    #[test]
    fn quiet_stages_only_contain_normal_or_castling_moves() {
        let mut normal_quiets = 0;
        let mut castles = 0;

        for_each_generated_move(|_, stage, scored_move| {
            if !matches!(stage, Stage::GoodQuiets | Stage::BadQuiets) {
                return;
            }

            assert!(scored_move.mv.promotion().is_none());
            assert_eq!(scored_move.captured, None);
            match scored_move.mv.kind() {
                MoveType::Normal => normal_quiets += 1,
                MoveType::Castle => castles += 1,
                kind => panic!("quiet stage produced {kind:?}"),
            }
        });

        assert!(normal_quiets > 0);
        assert!(castles > 0);
    }

    #[test]
    fn generated_history_keys_match_the_moving_piece_and_destination() {
        let mut checked = 0;

        for_each_generated_move(|board, stage, scored_move| {
            assert!(
                !matches!(stage, Stage::PvTt | Stage::Killers),
                "test unexpectedly received an externally supplied special move"
            );

            let moving_piece = board
                .piece_at(scored_move.mv.from())
                .expect("generated move must have a moving piece");
            let expected =
                HistoryKey::new(moving_piece.color, moving_piece.kind, scored_move.mv.to());
            let actual = scored_move
                .history_key
                .expect("every generator-produced move must cache its history key");

            assert_eq!(
                actual.idx(),
                expected.idx(),
                "wrong history key for {:?}",
                scored_move.mv
            );

            if scored_move.mv.kind() == MoveType::EnPassant {
                assert_eq!(scored_move.captured, Some(PieceType::Pawn));
            }

            checked += 1;
        });

        assert!(checked > 0);
    }
}
