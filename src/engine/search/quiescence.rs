use crate::board::{Board, Move, MoveType};
use crate::engine::SearchContext;
use crate::engine::ordering::see;
use crate::engine::pruning::{calculate_see_threshold, delta_pruning_bound};
use crate::engine::search::NodeType;
use crate::engine::search::is_insufficient_material;
use crate::engine::search_stats::track_stat;
use crate::engine::tt::{TTFlag, TTNodeType, score_from_tt, score_to_tt};
use crate::engine::{Engine, MAX_PLY, PickerFrame};
use crate::eval::evaluation_for_turn;
use crate::moves::MoveGenInfo;
use crate::types::{PieceType, Score, is_decisive, is_valid, mated_in};

// const LAZY_MARGIN: i32 = 300;

const MAX_CHECK_Q_PLIES: usize = 64;

impl Engine {
    pub(crate) fn quiescence<NODE: NodeType>(
        &mut self,
        board: &mut Board,
        context: &mut SearchContext,
        _depth: u16,
        mut alpha: i32,
        beta: i32,
        ply: usize,
        picker_frame: PickerFrame,
        check_plies: usize,
    ) -> i32 {
        if context.stopped || (context.stats.total_nodes() & 2047 == 0 && context.should_stop()) {
            track_stat!(context.stats.terminal_stats.stopped_returns += 1);
            return Score::ZERO;
        }

        context.stats.node_stats.quiescence += 1;

        if NODE::PV {
            context.seldepth = context.seldepth.max(ply);
        }

        // NOTE: These will never be true in the first call to quiescence since negamax does this check first before calling if depth == 0 -> quiescence
        // this is why quiescence qtt probes always equals qnodes.
        if Engine::repetition_in_search(
            context,
            board.board_hash(),
            board.halfmove_clock() as usize,
        ) {
            track_stat!(context.stats.draw_stats.repetition_returns += 1);
            return Score::ZERO;
        }
        if board.halfmove_clock() >= 100 {
            track_stat!(context.stats.draw_stats.fifty_move_returns += 1);
            return Score::ZERO;
        }

        if board.phase() < 8 && is_insufficient_material(board) {
            track_stat!(context.stats.draw_stats.insufficient_material_returns += 1);
            return Score::ZERO;
        }

        if ply >= MAX_PLY - 1 {
            track_stat! {
                context.stats.quiescence_stats.max_ply_returns += 1;
                context.stats.terminal_stats.max_ply_returns += 1;
            }
            return evaluation_for_turn(board);
        }

        let in_check = board.in_check(board.side_to_move());

        track_stat! {
            if in_check {
                context.stats.quiescence_stats.in_check_nodes += 1;
                context.stats.node_stats.in_check += 1;
            } else {
                context.stats.quiescence_stats.normal_nodes += 1;
            }
        }

        let original_alpha = alpha;
        let hash = board.board_hash();
        let side_to_move = board.side_to_move();

        let mut tt_best_move: Option<Move> = None;

        track_stat!(context.stats.tt_stats.quiescence.probes += 1);

        if let Some(entry) = self.tt.get(hash, TTNodeType::Quiescence) {
            track_stat! {
                context.stats.tt_stats.quiescence.hits += 1;

                match entry.flag.bound_type() {
                    TTFlag::Exact => context.stats.tt_stats.quiescence.exact_hits += 1,
                    TTFlag::LowerBound => context.stats.tt_stats.quiescence.lower_bound_hits += 1,
                    TTFlag::UpperBound => context.stats.tt_stats.quiescence.upper_bound_hits += 1,
                }
                if entry.best_move.is_some() {
                    context.stats.tt_stats.quiescence.move_hits += 1;
                }
            }

            // if entry.depth < depth as u8 {
            //     context.stats.tt_stats.quiescence.depth_rejected_hits += 1;
            // }

            tt_best_move = entry.best_move.filter(|mv| {
                in_check || mv.is_capture() || mv.promotion() == Some(PieceType::Queen)
            });

            let tt_score = score_from_tt(entry.eval as i32, ply);

            if is_valid(tt_score) {
                track_stat!(context.stats.tt_stats.quiescence.usable += 1);

                let cutoff = match entry.flag.bound_type() {
                    TTFlag::Exact => {
                        track_stat!(context.stats.tt_stats.quiescence.exact_returns += 1);
                        return tt_score;
                    }
                    TTFlag::LowerBound => tt_score >= beta,
                    TTFlag::UpperBound => tt_score <= alpha,
                };

                if cutoff {
                    track_stat!(context.stats.tt_stats.quiescence.bound_cutoffs += 1);
                    return tt_score;
                }
            }
        }

        let mut best_eval = -Score::INFINITE;
        let mut stand_pat = -Score::INFINITE;
        let mut best_move: Option<Move> = None;

        let mut move_picker = if in_check {
            self.new_move_picker(
                0,
                context,
                side_to_move,
                ply,
                picker_frame,
                None,
                tt_best_move,
            )
        } else {
            self.new_move_picker(
                1,
                context,
                side_to_move,
                ply,
                picker_frame,
                None,
                tt_best_move,
            )
        };

        let info = MoveGenInfo::calculate(board, side_to_move);
        let mut pending_move = move_picker.get_next(board, &info, context, &self.history);

        let mut raw_static_eval = Score::NONE;

        if in_check {
            if pending_move.is_none() {
                track_stat!(context.stats.terminal_stats.checkmates += 1);
                let score = mated_in(ply);
                track_stat!(context.stats.tt_stats.quiescence.stores += 1);
                let _insert_result = self.tt.insert(
                    hash,
                    false,
                    TTFlag::Exact,
                    0,
                    score_to_tt(score, ply),
                    Score::NONE as i16,
                    None,
                );
                track_stat!(
                    context
                        .stats
                        .tt_stats
                        .quiescence
                        .record_insert(_insert_result)
                );
                return score;
            }

            // Too many consecutive checks. Confirm that the position is not
            // checkmate before returning the fallback score.
            if check_plies > MAX_CHECK_Q_PLIES {
                track_stat!(context.stats.quiescence_stats.check_ply_limit_returns += 1);

                // Since this is an awkward node, do not store it in the TT.
                let fallback = Score::ZERO.clamp(alpha, beta - 1);
                return fallback;
            }
        } else {
            if pending_move.is_none() {
                // A capture-only picker cannot distinguish stalemate from a
                // position with only quiet moves. Probe one move from a main
                // picker on a separate frame while this picker remains alive.
                let mut legal_move_picker = self.new_move_picker(
                    0,
                    context,
                    side_to_move,
                    ply,
                    picker_frame.child(),
                    None,
                    None,
                );

                if legal_move_picker
                    .get_next(board, &info, context, &self.history)
                    .is_none()
                {
                    track_stat!(context.stats.terminal_stats.stalemates += 1);
                    let score = Score::ZERO;
                    track_stat!(context.stats.tt_stats.quiescence.stores += 1);
                    let _insert_result = self.tt.insert(
                        hash,
                        false,
                        TTFlag::Exact,
                        0,
                        score_to_tt(score, ply),
                        Score::NONE as i16,
                        None,
                    );
                    track_stat!(
                        context
                            .stats
                            .tt_stats
                            .quiescence
                            .record_insert(_insert_result)
                    );
                    return score;
                }
            }

            raw_static_eval = evaluation_for_turn(board);

            stand_pat = if self.config.search.correction.enabled {
                let correction = self.history.correction.get(board);
                track_stat!(
                    context
                        .stats
                        .correction_stats
                        .record_quiescence_lookup(correction)
                );
                raw_static_eval + correction
            } else {
                raw_static_eval
            };

            best_eval = stand_pat;

            if stand_pat >= beta {
                track_stat!(context.stats.cutoff_stats.stand_pat += 1);

                track_stat!(context.stats.tt_stats.quiescence.stores += 1);
                let _insert_result = self.tt.insert(
                    hash,
                    false,
                    TTFlag::LowerBound,
                    0,
                    score_to_tt(stand_pat, ply),
                    raw_static_eval as i16,
                    None,
                );
                track_stat!(
                    context
                        .stats
                        .tt_stats
                        .quiescence
                        .record_insert(_insert_result)
                );

                return stand_pat;
            }

            if alpha < stand_pat {
                alpha = stand_pat;
            }
        }

        #[cfg(feature = "search-stats")]
        let mut searched_moves = 0;

        loop {
            let scored_mv = if let Some(scored_mv) = pending_move.take() {
                scored_mv
            } else if let Some(scored_mv) =
                move_picker.get_next(board, &info, context, &self.history)
            {
                scored_mv
            } else {
                break;
            };

            track_stat! {
                if in_check {
                    context.stats.quiescence_stats.check_evasion_candidates += 1;
                } else {
                    context.stats.quiescence_stats.capture_candidates += 1;
                }
            }

            let mv = &scored_mv.mv;
            let captured_value = match mv.kind() {
                MoveType::EnPassant => PieceType::Pawn.value(),

                _ => board.piece_at(mv.to()).map(|p| p.kind.value()).unwrap_or(0),
            };

            let gives_check = board.move_gives_check(mv);

            let can_prune = !NODE::PV
                && !in_check
                && board.phase() > 8
                && mv.promotion().is_none()
                && !is_decisive(alpha)
                && Some(*mv) != tt_best_move
                && !gives_check;

            if can_prune {
                // delta pruning
                if self.config.search.delta.enabled {
                    track_stat!(context.stats.q_pruning_stats.delta_attempts += 1);
                    if delta_pruning_bound(stand_pat, captured_value, &self.config.search.delta)
                        < alpha
                    {
                        track_stat!(context.stats.q_pruning_stats.delta_prunes += 1);
                        continue;
                    }
                }

                if self.config.search.see.enabled {
                    track_stat!(context.stats.q_pruning_stats.see_attempts += 1);
                    let see_value = if let Some(value) = scored_mv.see {
                        value
                    } else {
                        see(board, *mv, scored_mv.captured)
                    };
                    if see_value < calculate_see_threshold(&self.config.search.see) {
                        track_stat!(context.stats.q_pruning_stats.see_prunes += 1);
                        continue;
                    }
                }
            }

            let undo = board.make_move(*mv);
            let child_hash = board.board_hash();

            context.repetition_history.push(child_hash);

            let check_plies = if gives_check { check_plies + 1 } else { 0 };

            let eval = -self.quiescence::<NODE>(
                board,
                context,
                _depth,
                -beta,
                -alpha,
                ply + 1,
                picker_frame.child(),
                check_plies,
            );

            if context.stopped {
                context.repetition_history.pop();
                board.undo_move(undo);
                return Score::ZERO;
            }

            track_stat! {
                context.stats.move_stats.quiescence_searched += 1;
                searched_moves += 1;
            }

            context.repetition_history.pop();

            board.undo_move(undo);

            if eval > best_eval {
                best_eval = eval;
                best_move = Some(*mv);
            }

            alpha = alpha.max(eval);

            if eval >= beta {
                track_stat! {
                    context.stats.cutoff_stats.quiescence_beta += 1;

                    if searched_moves == 1 {
                        context.stats.cutoff_stats.first_move_quiescence_beta += 1;
                    }
                }

                track_stat!(context.stats.tt_stats.quiescence.stores += 1);
                let _insert_result = self.tt.insert(
                    hash,
                    false,
                    TTFlag::LowerBound,
                    0,
                    score_to_tt(eval, ply),
                    raw_static_eval as i16,
                    best_move,
                );
                track_stat!(
                    context
                        .stats
                        .tt_stats
                        .quiescence
                        .record_insert(_insert_result)
                );
                return eval;
            }
        }

        let flag = if best_eval <= original_alpha {
            TTFlag::UpperBound
        } else if best_eval >= beta {
            TTFlag::LowerBound
        } else {
            TTFlag::Exact
        };

        track_stat!(context.stats.tt_stats.quiescence.stores += 1);
        let _insert_result = self.tt.insert(
            hash,
            false,
            flag,
            0,
            score_to_tt(best_eval, ply),
            raw_static_eval as i16,
            best_move,
        );
        track_stat!(
            context
                .stats
                .tt_stats
                .quiescence
                .record_insert(_insert_result)
        );

        best_eval
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::engine::configs::EngineConfig;
    use crate::engine::search::PV;
    use crate::engine::{SearchContext, SearchLimits};

    #[test]
    fn staged_quiescence_detects_checkmate_and_stalemate() {
        let cases = [
            ("7k/6Q1/6K1/8/8/8/8/8 b - - 0 1", -Score::MATE),
            ("7k/5Q2/6K1/8/8/8/8/8 b - - 0 1", 0),
        ];

        for (fen, expected) in cases {
            let mut config = EngineConfig::standard();
            config.tt_size = 1;
            let mut engine = Engine::new(config);
            let mut board = Board::from_fen(fen).expect("valid terminal FEN");
            let mut context =
                SearchContext::new(SearchLimits::depth(1, 1), vec![board.board_hash()]);

            let score = engine.quiescence::<PV>(
                &mut board,
                &mut context,
                1,
                -Score::INFINITE,
                Score::INFINITE,
                0,
                PickerFrame::ROOT,
                0,
            );

            assert_eq!(score, expected, "wrong terminal score for {fen}");
        }
    }

    #[test]
    fn noncutting_tt_bound_and_quiet_hash_move_preserve_stand_pat() {
        let mut config = EngineConfig::standard();
        config.tt_size = 1;
        let mut engine = Engine::new(config);
        let mut board = Board::from_fen(crate::board::STARTPOS_FEN).unwrap();
        let hash = board.board_hash();
        let stand_pat = evaluation_for_turn(&board);
        let quiet = crate::uci::parse_uci_move(&board, "e2e4").unwrap();
        engine.tt.insert(
            hash,
            false,
            TTFlag::UpperBound,
            1,
            score_to_tt(stand_pat, 0),
            stand_pat as i16,
            Some(quiet),
        );
        let mut context = SearchContext::new(SearchLimits::depth(1, 6), vec![hash]);

        let score = engine.quiescence::<PV>(
            &mut board,
            &mut context,
            6,
            stand_pat - 10,
            stand_pat + 10,
            0,
            PickerFrame::ROOT,
            0,
        );

        assert_eq!(score, stand_pat);
        assert_eq!(context.stats.node_stats.quiescence, 1);
        assert_eq!(
            engine.tt.get_any(hash).unwrap().flag.bound_type(),
            TTFlag::Exact
        );
    }

    #[test]
    fn quiescence_reuses_its_depth_zero_exact_entry() {
        let mut config = EngineConfig::standard();
        config.tt_size = 1;
        let mut engine = Engine::new(config);
        let mut board = Board::from_fen("7k/6Q1/6K1/8/8/8/8/8 b - - 0 1").unwrap();
        let mut context = SearchContext::new(SearchLimits::depth(1, 6), vec![board.board_hash()]);

        for _ in 0..2 {
            assert_eq!(
                engine.quiescence::<PV>(
                    &mut board,
                    &mut context,
                    6,
                    -Score::INFINITE,
                    Score::INFINITE,
                    0,
                    PickerFrame::ROOT,
                    0,
                ),
                -Score::MATE,
            );
        }

        assert_eq!(
            engine
                .tt
                .get(board.board_hash(), TTNodeType::Quiescence)
                .unwrap()
                .depth,
            0
        );
        assert_eq!(context.stats.node_stats.quiescence, 2);
        #[cfg(feature = "search-stats")]
        {
            assert_eq!(context.stats.tt_stats.quiescence.hits, 1);
            // Regression: qsearch stores zero but still compares it with `depth`.
            assert_eq!(context.stats.tt_stats.quiescence.exact_returns, 1);
            assert_eq!(context.stats.tt_stats.quiescence.depth_rejected_hits, 0);
        }
        #[cfg(not(feature = "search-stats"))]
        {
            assert_eq!(context.stats.tt_stats.quiescence.hits, 0);
            assert_eq!(context.stats.tt_stats.quiescence.exact_returns, 0);
            assert_eq!(context.stats.tt_stats.quiescence.stores, 0);
            assert_eq!(context.stats.terminal_stats.checkmates, 0);
        }
    }

    #[test]
    fn quiescence_can_reuse_a_main_search_entry() {
        let mut config = EngineConfig::standard();
        config.tt_size = 1;
        let mut engine = Engine::new(config);
        let mut board = Board::from_fen("7k/6Q1/6K1/8/8/8/8/8 b - - 0 1").unwrap();
        engine.tt.insert(
            board.board_hash(),
            true,
            TTFlag::Exact,
            10,
            123,
            Score::NONE as i16,
            None,
        );
        let mut context = SearchContext::new(SearchLimits::depth(1, 6), vec![board.board_hash()]);

        let score = engine.quiescence::<PV>(
            &mut board,
            &mut context,
            6,
            -Score::INFINITE,
            Score::INFINITE,
            0,
            PickerFrame::ROOT,
            0,
        );

        assert_eq!(score, 123);
        #[cfg(feature = "search-stats")]
        assert_eq!(context.stats.tt_stats.quiescence.exact_returns, 1);
        #[cfg(not(feature = "search-stats"))]
        assert_eq!(context.stats.tt_stats.quiescence.exact_returns, 0);
    }
}
