use crate::bitboard::{pop_lsb, rank_of};
use crate::board::{Board, Move, MoveType};
use crate::engine::history::HistoryKey;
use crate::engine::ordering::promotion_score;
use crate::engine::staged::{MovePicker, ScoredMove};
use crate::moves::MoveGenInfo;
use crate::moves::pawn::is_legal_pawn_move;
use crate::types::{Color, PieceType};

const UNDERPROMOTION: [PieceType; 3] = [PieceType::Knight, PieceType::Bishop, PieceType::Rook];

impl MovePicker {
    pub fn compute_promotions(
        &mut self,
        board: &Board,
        info: &MoveGenInfo,
        move_buffer: &mut [ScoredMove],
    ) {
        let mut front = 0;

        let mut tail = self.bad_capture_start - 1;

        self.staged_legal_pawn_promotion_moves(
            board,
            info,
            self.side,
            &mut front,
            &mut tail,
            move_buffer,
        );

        self.current_len = front;
        self.underpromo_start = tail + 1; // the next stage will use this index as the starting tail index
    }

    pub fn staged_legal_pawn_promotion_moves(
        &self,
        board: &Board,
        info: &MoveGenInfo,
        color: Color,
        front: &mut usize,
        tail: &mut usize,
        move_buffer: &mut [ScoredMove],
    ) {
        let pawns = board.pieces(color, PieceType::Pawn);
        let occupancy = board.all_occupancy();
        let empty = !occupancy;
        match color {
            Color::White => {
                let promotion_rank = 7;

                let mut single_pushes = (pawns << 8) & empty;

                while let Some(to) = pop_lsb(&mut single_pushes) {
                    let from = to - 8;

                    if rank_of(to) != promotion_rank || !is_legal_pawn_move(to, from, info) {
                        continue;
                    }

                    let mv = Move::new(from, to, MoveType::Normal, Some(PieceType::Queen));
                    if self.is_special(mv) {
                        continue;
                    }
                    let history_key = HistoryKey::new(color, PieceType::Pawn, to);
                    move_buffer[*front] = ScoredMove {
                        mv,
                        score: 800_000,
                        history_key: Some(history_key),
                        see: None,
                        captured: None,
                    };
                    *front += 1;

                    for &promo_piece in UNDERPROMOTION.iter() {
                        let mv = Move::new(from, to, MoveType::Normal, Some(promo_piece));
                        if self.is_special(mv) {
                            continue;
                        }
                        move_buffer[*tail] = ScoredMove {
                            mv,
                            score: -500_000 + promotion_score(promo_piece),
                            history_key: Some(history_key),
                            see: None,
                            captured: None,
                        };
                        *tail -= 1;
                    }
                }
            }

            Color::Black => {
                let promotion_rank = 0;

                let mut single_pushes = (pawns >> 8) & empty;

                while let Some(to) = pop_lsb(&mut single_pushes) {
                    let from = to + 8;

                    if rank_of(to) != promotion_rank || !is_legal_pawn_move(to, from, info) {
                        continue; // promotion moves are handled separately
                    }

                    let mv = Move::new(from, to, MoveType::Normal, Some(PieceType::Queen));
                    if self.is_special(mv) {
                        continue;
                    }
                    let history_key = HistoryKey::new(color, PieceType::Pawn, to);
                    move_buffer[*front] = ScoredMove {
                        mv,
                        score: 800_000,
                        history_key: Some(history_key),
                        see: None,
                        captured: None,
                    };
                    *front += 1;

                    for &promo_piece in UNDERPROMOTION.iter() {
                        let mv = Move::new(from, to, MoveType::Normal, Some(promo_piece));
                        if self.is_special(mv) {
                            continue;
                        }
                        move_buffer[*tail] = ScoredMove {
                            mv,
                            score: -500_000 + promotion_score(promo_piece),
                            history_key: Some(history_key),
                            see: None,
                            captured: None,
                        };
                        *tail -= 1;
                    }
                }
            }
        }
    }
}
