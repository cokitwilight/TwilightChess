use crate::{
    board::{Board, Move, MoveList, MoveType},
    engine::{
        Engine, SearchContext,
        history::HistoryKey,
        ordering::{ordering::promotion_score, see},
    },
    types::{Color, PieceType},
};

const STAGES: [i32; 8] = [
    1_500_000, 900_000, 800_000, 700_000, 0, -490_000, -500_000, -600_000,
];

// stages go:
// 1: +1,500,000 = pv move and tt move
// 2: +900,000 = winning/equal captures
// 3: +800,000 = quiet queen promotions
// 4: +700_000 = killer moves
// 5: +0 = good/neutral history quiet moves
// 6: 0 <-> -499,999 = bad history quiet moves
// 7: + -500,000 = quiet under promotions(maybe change this)
// 8: + -600,000 = losing captures

#[derive(Clone, Copy, Debug)]
pub struct ScoredMove {
    pub mv: Move,
    pub score: i32,
    pub history_key: Option<HistoryKey>,
    pub see: Option<i32>,
}

#[derive(Clone)]
pub struct StagedMoveSelector {
    // since these are Vec try to sort with highest at the end so removing is just .pop()
    staged: Vec<Vec<ScoredMove>>,
    remaining: MoveList,
    current_stage: usize, // the stage number(i.e 1 -> tt moves, 2-> winning captures)
}

/*
self.order_moves(
            board,
            &mut moves,
            side_to_move,
            ply,
            context,
            None,
            ordering_tt_move,
        );
*/

impl Engine {
    pub fn new_staged_move_selecter(
        &self,
        board: &Board,
        moves: &mut MoveList,
        side: Color,
        ply: usize,
        context: &SearchContext,
        pv_move: Option<Move>,
        tt_move: Option<Move>,
    ) -> StagedMoveSelector {
        let mut staged: Vec<Vec<ScoredMove>> = vec![Vec::with_capacity(5); 8];
        let mut current_stage: usize = 1;

        if pv_move.is_none() && tt_move.is_none() {
            current_stage += 1;
        }

        if moves.is_empty() {
            panic!("No moves in staged move selector constructor!");
        }

        loop {
            // only build stage 1(or earliest possible one)

            if current_stage > 7 {
                for stage in 0..7 {
                    if !staged[stage].is_empty() {
                        panic!(
                            "Current Stage is {} however stage {} has moves in new_staged_move_selector!",
                            current_stage, stage
                        )
                    }
                }
                panic!("Stage 7 reached but staged is empty!");
            }

            match current_stage {
                1 => {
                    let mut i = 0;
                    while i < moves.len() {
                        let mv = moves.get(i);
                        if Some(mv) == pv_move {
                            staged[current_stage].push(ScoredMove {
                                mv: mv,
                                score: 2_000_000,
                                history_key: None,
                                see: None,
                            });
                            moves.swap_remove(i);
                            continue;
                        }
                        if Some(mv) == tt_move {
                            staged[current_stage].push(ScoredMove {
                                mv: mv,
                                score: 1_500_000,
                                history_key: None,
                                see: None,
                            });
                            moves.swap_remove(i);
                            continue;
                        }
                        i += 1;
                    }
                }
                2 => {
                    let mut i = 0;

                    while i < moves.len() {
                        let mv = moves.get(i);
                        if mv.kind() != MoveType::Capture {
                            i += 1;
                            continue;
                        }

                        let see = see(board, mv);
                        let promo_bonus = mv.promotion().map_or(0, promotion_score);
                        if see >= 0 {
                            staged[current_stage].push(ScoredMove {
                                mv,
                                score: 900_000 + see + promo_bonus,
                                history_key: None,
                                see: Some(see),
                            });
                            moves.swap_remove(i);
                            continue;
                        } else {
                            staged[7].push(ScoredMove {
                                mv,
                                score: -600_000 + see + promo_bonus,
                                history_key: None,
                                see: Some(see),
                            });
                            moves.swap_remove(i);
                            continue;
                        }
                    }
                }
                3 | 4 => {
                    let mut i = 0;

                    while i < moves.len() {
                        let mv = moves.get(i);

                        if context.killer_moves.contains(ply, mv) {
                            staged[4].push(ScoredMove {
                                mv,
                                score: 700_000,
                                history_key: None,
                                see: None,
                            });
                            moves.swap_remove(i);
                            continue;
                        }

                        if mv.promotion() == Some(PieceType::Queen) {
                            staged[3].push(ScoredMove {
                                mv,
                                score: 900_000,
                                history_key: None,
                                see: None,
                            });
                            moves.swap_remove(i);
                            continue;
                        }
                        i += 1;
                    }
                }
                _ => {
                    let mut i = 0;

                    while i < moves.len() {
                        let mv = moves.get(i);
                        let is_quiet = mv.kind() == MoveType::Normal && mv.promotion().is_none();

                        let history_key = if is_quiet {
                            let piece = board
                                .piecetype_at(mv.from())
                                .expect("No piece in board in move ordering!");

                            Some(HistoryKey::new(side, piece, mv.to()))
                        } else {
                            None
                        };

                        let see = if !is_quiet {
                            Some(see(board, mv))
                        } else {
                            None
                        };

                        let score = self.move_order_score(
                            board,
                            mv,
                            side,
                            ply,
                            context,
                            pv_move,
                            tt_move,
                            see,
                            history_key,
                        );
                        let scored_mv = ScoredMove {
                            mv,
                            score,
                            history_key,
                            see,
                        };
                        if score >= STAGES[5] {
                            staged[5].push(scored_mv);
                        } else if score >= STAGES[6] {
                            staged[6].push(scored_mv);
                        } else {
                            staged[7].push(scored_mv);
                        }
                        i += 1;
                    }
                    moves.clear();
                }
            }
            if staged[current_stage].is_empty() {
                current_stage += 1;
            } else {
                break;
            }
        }

        StagedMoveSelector {
            staged,
            remaining: *moves,
            current_stage,
        }
    }
}

impl StagedMoveSelector {
    pub fn get_next(&mut self) -> Option<ScoredMove> {
        loop {
            if self.staged[self.current_stage].is_empty() {
                if self.current_stage >= 7 {
                    // MAX STAGES
                    return None;
                }

                self.current_stage += 1;
                continue;
            }
            let best_index = self.staged[self.current_stage]
                .iter()
                .enumerate()
                .max_by_key(|(_, scored)| scored.score)
                .map(|(index, _)| index)
                .unwrap();

            return Some(self.staged[self.current_stage].swap_remove(best_index));
        }
    }
}
