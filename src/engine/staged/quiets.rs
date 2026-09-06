use crate::bitboard::{
    A1, A8, B1, B8, C1, C8, D1, D8, E1, E8, F1, F8, G1, G8, H1, H8, RANK_3, RANK_6, bishop_attacks,
    bit, king_attacks, knight_attacks, pop_lsb, queen_attacks, rank_of, rook_attacks,
};
use crate::board::{
    BLACK_KINGSIDE, BLACK_QUEENSIDE, Board, Move, MoveType, WHITE_KINGSIDE, WHITE_QUEENSIDE,
};
use crate::engine::SearchContext;
use crate::engine::history::{HistoryKey, HistoryTables};
use crate::engine::staged::staged::Stage;
use crate::engine::staged::{MovePicker, ScoredMove};
use crate::moves::MoveGenInfo;
use crate::moves::king::legal_king_move;
use crate::moves::pawn::is_legal_pawn_move;
use crate::types::{Color, PieceType};

impl MovePicker {
    pub fn compute_quiets(
        &mut self,
        board: &Board,
        info: &MoveGenInfo,
        context: &mut SearchContext,
        history: &HistoryTables,
    ) {
        debug_assert_eq!(
            self.stage,
            Stage::GoodQuiets,
            "compute_quiets called in wrong stage"
        );

        let mut front = 0;

        let mut tail = self.underpromo_start - 1;

        self.staged_legal_king_quiet_moves(
            board,
            self.side,
            &mut front,
            &mut tail,
            context,
            history,
            self.ply,
            self.picker,
        );
        self.staged_legal_pawn_quiet_moves(
            board,
            info,
            self.side,
            &mut front,
            &mut tail,
            context,
            history,
            self.ply,
            self.picker,
        );
        self.staged_legal_knight_quiet_moves(
            board,
            info,
            self.side,
            &mut front,
            &mut tail,
            context,
            history,
            self.ply,
            self.picker,
        );
        self.staged_legal_bishop_quiet_moves(
            board,
            info,
            self.side,
            &mut front,
            &mut tail,
            context,
            history,
            self.ply,
            self.picker,
        );
        self.staged_legal_rook_quiet_moves(
            board,
            info,
            self.side,
            &mut front,
            &mut tail,
            context,
            history,
            self.ply,
            self.picker,
        );
        self.staged_legal_queen_quiet_moves(
            board,
            info,
            self.side,
            &mut front,
            &mut tail,
            context,
            history,
            self.ply,
            self.picker,
        );

        self.current_len = front;
        self.bad_quiet_start = tail + 1; // the next stage will use this index as the starting tail index
    }

    pub fn staged_legal_king_quiet_moves(
        &self,
        board: &Board,
        color: Color,
        front: &mut usize,
        tail: &mut usize,
        context: &mut SearchContext,
        history: &HistoryTables,
        ply: usize,
        picker: usize,
    ) {
        let mut king = board.pieces(color, PieceType::King);
        let friends = board.occupancy_of(color);
        let empty = !board.all_occupancy();

        let Some(from) = pop_lsb(&mut king) else {
            board.print_board();
            panic!("No king at board.pieces(king)");
        };

        let targets = king_attacks(from) & !friends;

        let mut quiets = targets & empty;

        while let Some(to) = pop_lsb(&mut quiets) {
            let mv = Move::new(from, to, MoveType::Normal, None);
            if self.is_special(mv) {
                continue;
            }
            if !legal_king_move(board, mv) {
                continue;
            }
            let history_key = HistoryKey::new(board.side_to_move(), PieceType::King, mv.to());
            let score = history.get_quiet_score(&context.stack, ply, history_key);
            if score > 0 {
                context.move_buffers[picker].entries[*front] = ScoredMove {
                    mv,
                    score,
                    history_key: Some(history_key),
                    see: None,
                    captured: None,
                };
                *front += 1;
            } else {
                context.move_buffers[picker].entries[*tail] = ScoredMove {
                    mv,
                    score,
                    history_key: Some(history_key),
                    see: None,
                    captured: None,
                };
                *tail -= 1;
            }
        }
        self.staged_legal_castling_moves(board, color, front, tail, context, history, ply, picker);
    }

    pub fn staged_legal_castling_moves(
        &self,
        board: &Board,
        color: Color,
        front: &mut usize,
        tail: &mut usize,
        context: &mut SearchContext,
        history: &HistoryTables,
        ply: usize,
        picker: usize,
    ) {
        let enemy = color.opposite();
        let occupied = board.all_occupancy();

        match color {
            Color::White => {
                // White king must be on e1.
                if board.pieces(Color::White, PieceType::King) != bit(E1) {
                    return;
                }

                // Cannot castle while in check.
                if board.square_attacked_by(enemy, E1) {
                    return;
                }

                // -------------------------
                // White kingside: e1 -> g1, rook h1 -> f1
                // -------------------------
                if board.has_castling_right(WHITE_KINGSIDE) {
                    let between = bit(F1) | bit(G1);

                    let rook_on_h1 = board.pieces(Color::White, PieceType::Rook) & bit(H1) != 0;

                    let path_empty = occupied & between == 0;

                    let path_safe = !board.square_attacked_by(enemy, F1)
                        && !board.square_attacked_by(enemy, G1);

                    if rook_on_h1 && path_empty && path_safe {
                        let mv = Move::new(E1, G1, MoveType::Castle, None);
                        if legal_king_move(board, mv) && !self.is_special(mv) {
                            let history_key = HistoryKey::new(color, PieceType::King, mv.to());
                            let score = history.get_quiet_score(&context.stack, ply, history_key);
                            if score > 0 {
                                context.move_buffers[picker].entries[*front] = ScoredMove {
                                    mv,
                                    score,
                                    history_key: Some(history_key),
                                    see: None,
                                    captured: None,
                                };
                                *front += 1;
                            } else {
                                context.move_buffers[picker].entries[*tail] = ScoredMove {
                                    mv,
                                    score,
                                    history_key: Some(history_key),
                                    see: None,
                                    captured: None,
                                };
                                *tail -= 1;
                            }
                        }
                    }
                }

                // -------------------------
                // White queenside: e1 -> c1, rook a1 -> d1
                // -------------------------
                if board.has_castling_right(WHITE_QUEENSIDE) {
                    let between = bit(D1) | bit(C1) | bit(B1);

                    let rook_on_a1 = board.pieces(Color::White, PieceType::Rook) & bit(A1) != 0;

                    let path_empty = occupied & between == 0;

                    // King moves through d1 and lands on c1.
                    // b1 must be empty, but b1 does not need to be safe.
                    let path_safe = !board.square_attacked_by(enemy, D1)
                        && !board.square_attacked_by(enemy, C1);

                    if rook_on_a1 && path_empty && path_safe {
                        let mv = Move::new(E1, C1, MoveType::Castle, None);
                        if legal_king_move(board, mv) && !self.is_special(mv) {
                            let history_key = HistoryKey::new(color, PieceType::King, mv.to());
                            let score = history.get_quiet_score(&context.stack, ply, history_key);
                            if score > 0 {
                                context.move_buffers[picker].entries[*front] = ScoredMove {
                                    mv,
                                    score,
                                    history_key: Some(history_key),
                                    see: None,
                                    captured: None,
                                };
                                *front += 1;
                            } else {
                                context.move_buffers[picker].entries[*tail] = ScoredMove {
                                    mv,
                                    score,
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

            Color::Black => {
                // Black king must be on e8.
                if board.pieces(Color::Black, PieceType::King) != bit(E8) {
                    return;
                }

                // Cannot castle while in check.
                if board.square_attacked_by(enemy, E8) {
                    return;
                }

                // -------------------------
                // Black kingside: e8 -> g8, rook h8 -> f8
                // -------------------------
                if board.has_castling_right(BLACK_KINGSIDE) {
                    let between = bit(F8) | bit(G8);

                    let rook_on_h8 = board.pieces(Color::Black, PieceType::Rook) & bit(H8) != 0;

                    let path_empty = occupied & between == 0;

                    let path_safe = !board.square_attacked_by(enemy, F8)
                        && !board.square_attacked_by(enemy, G8);

                    if rook_on_h8 && path_empty && path_safe {
                        let mv = Move::new(E8, G8, MoveType::Castle, None);
                        if legal_king_move(board, mv) && !self.is_special(mv) {
                            let history_key = HistoryKey::new(color, PieceType::King, mv.to());
                            let score = history.get_quiet_score(&context.stack, ply, history_key);
                            if score > 0 {
                                context.move_buffers[picker].entries[*front] = ScoredMove {
                                    mv,
                                    score,
                                    history_key: Some(history_key),
                                    see: None,
                                    captured: None,
                                };
                                *front += 1;
                            } else {
                                context.move_buffers[picker].entries[*tail] = ScoredMove {
                                    mv,
                                    score,
                                    history_key: Some(history_key),
                                    see: None,
                                    captured: None,
                                };
                                *tail -= 1;
                            }
                        }
                    }
                }

                // -------------------------
                // Black queenside: e8 -> c8, rook a8 -> d8
                // -------------------------
                if board.has_castling_right(BLACK_QUEENSIDE) {
                    let between = bit(D8) | bit(C8) | bit(B8);

                    let rook_on_a8 = board.pieces(Color::Black, PieceType::Rook) & bit(A8) != 0;

                    let path_empty = occupied & between == 0;

                    // King moves through d8 and lands on c8.
                    // b8 must be empty, but b8 does not need to be safe.
                    let path_safe = !board.square_attacked_by(enemy, D8)
                        && !board.square_attacked_by(enemy, C8);

                    if rook_on_a8 && path_empty && path_safe {
                        let mv = Move::new(E8, C8, MoveType::Castle, None);

                        if legal_king_move(board, mv) && !self.is_special(mv) {
                            let history_key = HistoryKey::new(color, PieceType::King, mv.to());
                            let score = history.get_quiet_score(&context.stack, ply, history_key);
                            if score > 0 {
                                context.move_buffers[picker].entries[*front] = ScoredMove {
                                    mv,
                                    score,
                                    history_key: Some(history_key),
                                    see: None,
                                    captured: None,
                                };
                                *front += 1;
                            } else {
                                context.move_buffers[picker].entries[*tail] = ScoredMove {
                                    mv,
                                    score,
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
    }

    pub fn staged_legal_knight_quiet_moves(
        &self,
        board: &Board,
        info: &MoveGenInfo,
        color: Color,
        front: &mut usize,
        tail: &mut usize,
        context: &mut SearchContext,
        history: &HistoryTables,
        ply: usize,
        picker: usize,
    ) {
        let mut knights = board.pieces(color, PieceType::Knight);
        let enemies = board.occupancy_of(color.opposite());
        let friends = board.occupancy_of(color);

        while let Some(from) = pop_lsb(&mut knights) {
            let targets =
                knight_attacks(from) & !friends & info.pin_masks[from as usize] & info.check_mask;

            let mut quiets = targets & !enemies;

            while let Some(to) = pop_lsb(&mut quiets) {
                let mv = Move::new(from, to, MoveType::Normal, None);
                if self.is_special(mv) {
                    continue;
                }
                let history_key = HistoryKey::new(color, PieceType::Knight, to);
                let score = history.get_quiet_score(&context.stack, ply, history_key);
                if score > 0 {
                    context.move_buffers[picker].entries[*front] = ScoredMove {
                        mv,
                        score,
                        history_key: Some(history_key),
                        see: None,
                        captured: None,
                    };
                    *front += 1;
                } else {
                    context.move_buffers[picker].entries[*tail] = ScoredMove {
                        mv,
                        score,
                        history_key: Some(history_key),
                        see: None,
                        captured: None,
                    };
                    *tail -= 1;
                }
            }
        }
    }

    pub fn staged_legal_pawn_quiet_moves(
        &self,
        board: &Board,
        info: &MoveGenInfo,
        color: Color,
        front: &mut usize,
        tail: &mut usize,
        context: &mut SearchContext,
        history: &HistoryTables,
        ply: usize,
        picker: usize,
    ) {
        let pawns = board.pieces(color, PieceType::Pawn);
        let occupancy = board.all_occupancy();
        let empty = !occupancy;

        match color {
            Color::White => {
                let promotion_rank = 7;

                let mut single_pushes = (pawns << 8) & empty;

                let mut double_pushes = ((single_pushes & RANK_3) << 8) & empty;

                while let Some(to) = pop_lsb(&mut single_pushes) {
                    let from = to - 8;

                    if rank_of(to) == promotion_rank || !is_legal_pawn_move(to, from, info) {
                        continue; // promotion moves are handled separately
                    }

                    let mv = Move::new(from, to, MoveType::Normal, None);
                    if self.is_special(mv) {
                        continue;
                    }
                    let history_key = HistoryKey::new(color, PieceType::Pawn, to);
                    let score = history.get_quiet_score(&context.stack, ply, history_key);
                    if score > 0 {
                        context.move_buffers[picker].entries[*front] = ScoredMove {
                            mv,
                            score,
                            history_key: Some(history_key),
                            see: None,
                            captured: None,
                        };
                        *front += 1;
                    } else {
                        context.move_buffers[picker].entries[*tail] = ScoredMove {
                            mv,
                            score,
                            history_key: Some(history_key),
                            see: None,
                            captured: None,
                        };
                        *tail -= 1;
                    }
                }

                while let Some(to) = pop_lsb(&mut double_pushes) {
                    let from = to - 16;

                    if !is_legal_pawn_move(to, from, info) {
                        continue;
                    }

                    let mv = Move::new(from, to, MoveType::Normal, None);
                    if self.is_special(mv) {
                        continue;
                    }
                    let history_key = HistoryKey::new(color, PieceType::Pawn, to);
                    let score = history.get_quiet_score(&context.stack, ply, history_key);
                    if score > 0 {
                        context.move_buffers[picker].entries[*front] = ScoredMove {
                            mv,
                            score,
                            history_key: Some(history_key),
                            see: None,
                            captured: None,
                        };
                        *front += 1;
                    } else {
                        context.move_buffers[picker].entries[*tail] = ScoredMove {
                            mv,
                            score,
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

                let mut double_pushes = ((single_pushes & RANK_6) >> 8) & empty;

                while let Some(to) = pop_lsb(&mut single_pushes) {
                    let from = to + 8;

                    if rank_of(to) == promotion_rank {
                        continue; // promotion moves are handled separately
                    }

                    if !is_legal_pawn_move(to, from, info) {
                        continue;
                    }

                    let mv = Move::new(from, to, MoveType::Normal, None);
                    if self.is_special(mv) {
                        continue;
                    }
                    let history_key = HistoryKey::new(color, PieceType::Pawn, to);
                    let score = history.get_quiet_score(&context.stack, ply, history_key);
                    if score > 0 {
                        context.move_buffers[picker].entries[*front] = ScoredMove {
                            mv,
                            score,
                            history_key: Some(history_key),
                            see: None,
                            captured: None,
                        };
                        *front += 1;
                    } else {
                        context.move_buffers[picker].entries[*tail] = ScoredMove {
                            mv,
                            score,
                            history_key: Some(history_key),
                            see: None,
                            captured: None,
                        };
                        *tail -= 1;
                    }
                }

                while let Some(to) = pop_lsb(&mut double_pushes) {
                    let from = to + 16;
                    if !is_legal_pawn_move(to, from, info) {
                        continue;
                    }

                    let mv = Move::new(from, to, MoveType::Normal, None);
                    if self.is_special(mv) {
                        continue;
                    }
                    let history_key = HistoryKey::new(color, PieceType::Pawn, to);
                    let score = history.get_quiet_score(&context.stack, ply, history_key);
                    if score > 0 {
                        context.move_buffers[picker].entries[*front] = ScoredMove {
                            mv,
                            score,
                            history_key: Some(history_key),
                            see: None,
                            captured: None,
                        };
                        *front += 1;
                    } else {
                        context.move_buffers[picker].entries[*tail] = ScoredMove {
                            mv,
                            score,
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

    pub fn staged_legal_bishop_quiet_moves(
        &self,
        board: &Board,
        info: &MoveGenInfo,
        color: Color,
        front: &mut usize,
        tail: &mut usize,
        context: &mut SearchContext,
        history: &HistoryTables,
        ply: usize,
        picker: usize,
    ) {
        let mut bishops = board.pieces(color, PieceType::Bishop);
        let friends = board.occupancy_of(color);
        let occupancy = board.all_occupancy();
        let empty = !occupancy;

        while let Some(from) = pop_lsb(&mut bishops) {
            let targets = bishop_attacks(from, occupancy)
                & !friends
                & info.pin_masks[from as usize]
                & info.check_mask;

            let mut quiets = targets & empty;

            while let Some(to) = pop_lsb(&mut quiets) {
                let mv = Move::new(from, to, MoveType::Normal, None);
                if self.is_special(mv) {
                    continue;
                }
                let history_key = HistoryKey::new(color, PieceType::Bishop, to);
                let score = history.get_quiet_score(&context.stack, ply, history_key);
                if score > 0 {
                    context.move_buffers[picker].entries[*front] = ScoredMove {
                        mv,
                        score,
                        history_key: Some(history_key),
                        see: None,
                        captured: None,
                    };
                    *front += 1;
                } else {
                    context.move_buffers[picker].entries[*tail] = ScoredMove {
                        mv,
                        score,
                        history_key: Some(history_key),
                        see: None,
                        captured: None,
                    };
                    *tail -= 1;
                }
            }
        }
    }

    pub fn staged_legal_rook_quiet_moves(
        &self,
        board: &Board,
        info: &MoveGenInfo,
        color: Color,
        front: &mut usize,
        tail: &mut usize,
        context: &mut SearchContext,
        history: &HistoryTables,
        ply: usize,
        picker: usize,
    ) {
        let mut rooks = board.pieces(color, PieceType::Rook);
        let friends = board.occupancy_of(color);
        let occupancy = board.all_occupancy();
        let empty = !occupancy;

        while let Some(from) = pop_lsb(&mut rooks) {
            let targets = rook_attacks(from, occupancy)
                & !friends
                & info.pin_masks[from as usize]
                & info.check_mask;

            let mut quiets = targets & empty;

            while let Some(to) = pop_lsb(&mut quiets) {
                let mv = Move::new(from, to, MoveType::Normal, None);
                if self.is_special(mv) {
                    continue;
                }
                let history_key = HistoryKey::new(color, PieceType::Rook, to);
                let score = history.get_quiet_score(&context.stack, ply, history_key);
                if score > 0 {
                    context.move_buffers[picker].entries[*front] = ScoredMove {
                        mv,
                        score,
                        history_key: Some(history_key),
                        see: None,
                        captured: None,
                    };
                    *front += 1;
                } else {
                    context.move_buffers[picker].entries[*tail] = ScoredMove {
                        mv,
                        score,
                        history_key: Some(history_key),
                        see: None,
                        captured: None,
                    };
                    *tail -= 1;
                }
            }
        }
    }

    pub fn staged_legal_queen_quiet_moves(
        &self,
        board: &Board,
        info: &MoveGenInfo,
        color: Color,
        front: &mut usize,
        tail: &mut usize,
        context: &mut SearchContext,
        history: &HistoryTables,
        ply: usize,
        picker: usize,
    ) {
        let mut queens = board.pieces(color, PieceType::Queen);
        let friends = board.occupancy_of(color);
        let occupancy = board.all_occupancy();
        let empty = !occupancy;

        while let Some(from) = pop_lsb(&mut queens) {
            let targets = queen_attacks(from, occupancy)
                & !friends
                & info.pin_masks[from as usize]
                & info.check_mask;

            let mut quiets = targets & empty;

            while let Some(to) = pop_lsb(&mut quiets) {
                let mv = Move::new(from, to, MoveType::Normal, None);
                if self.is_special(mv) {
                    continue;
                }
                let history_key = HistoryKey::new(color, PieceType::Queen, to);
                let score = history.get_quiet_score(&context.stack, ply, history_key);
                if score > 0 {
                    context.move_buffers[picker].entries[*front] = ScoredMove {
                        mv,
                        score,
                        history_key: Some(history_key),
                        see: None,
                        captured: None,
                    };
                    *front += 1;
                } else {
                    context.move_buffers[picker].entries[*tail] = ScoredMove {
                        mv,
                        score,
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
