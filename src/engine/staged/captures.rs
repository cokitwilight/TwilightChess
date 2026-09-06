use crate::bitboard::{
    FILE_A, FILE_H, Square, bishop_attacks, bit, king_attacks, knight_attacks, pop_lsb,
    queen_attacks, rank_of, rook_attacks,
};
use crate::board::mv::MAX_MOVES;
use crate::board::{Board, Move, MoveType};
use crate::engine::history::{HistoryKey, HistoryTables};
use crate::engine::ordering::{promotion_score, see};
use crate::engine::staged::staged::Stage;
use crate::engine::staged::{MovePicker, ScoredMove};
use crate::moves::MoveGenInfo;
use crate::moves::king::legal_king_move;
use crate::moves::pawn::{is_legal_pawn_move, legal_en_passant};
use crate::types::{Color, PieceType};

const PROMOTIONS: [PieceType; 4] = [
    PieceType::Queen,
    PieceType::Rook,
    PieceType::Bishop,
    PieceType::Knight,
];

impl MovePicker {
    fn add_staged_capture_promotions(
        &self,
        board: &Board,
        color: Color,
        from: Square,
        to: Square,
        front: &mut usize,
        tail: &mut usize,
        move_buffer: &mut [ScoredMove],
        history: &HistoryTables,
    ) {
        let captured = board
            .piecetype_at(to)
            .expect("No piece at target square for capture promotion");
        let history_key = HistoryKey::new(color, PieceType::Pawn, to);
        let history_score = history.get_capture_score(history_key, captured);

        for promotion in PROMOTIONS {
            let mv = Move::new(from, to, MoveType::Capture, Some(promotion));
            if self.is_special(mv) {
                continue;
            }

            let see_score = see(board, mv, Some(captured));
            let score = if see_score >= 0 {
                900_000 + see_score + promotion_score(promotion) + history_score
            } else {
                -600_000 + see_score + promotion_score(promotion) + history_score
            };

            debug_assert!(
                *front < *tail,
                "Front index {} is not less than tail index {}",
                *front,
                *tail
            );

            let scored_move = ScoredMove {
                mv,
                score,
                history_key: Some(history_key),
                see: Some(see_score),
                captured: Some(captured),
            };

            if see_score >= 0 {
                move_buffer[*front] = scored_move;
                *front += 1;
            } else {
                move_buffer[*tail] = scored_move;
                *tail -= 1;
            }
        }
    }

    pub fn compute_captures(
        &mut self,
        board: &Board,
        info: &MoveGenInfo,
        move_buffer: &mut [ScoredMove],
        history: &HistoryTables,
    ) {
        debug_assert_eq!(
            self.stage,
            Stage::GoodCaptures,
            "compute_captures called in wrong stage"
        );

        let mut front = 0; // the starting index will always be 0 since we are reusing the start of the array

        let mut tail = MAX_MOVES - 1; // MAX_MOVES - 1. Since captures are first the starting tail should be the end of the array. Later calls will use the bad_capture_start index as the starting tail index.

        let color = board.side_to_move();

        self.staged_legal_king_capture_moves(
            board,
            color,
            &mut front,
            &mut tail,
            move_buffer,
            history,
        );
        self.staged_legal_knight_capture_moves(
            board,
            info,
            color,
            &mut front,
            &mut tail,
            move_buffer,
            history,
        );
        self.staged_legal_pawn_capture_moves(
            board,
            info,
            color,
            &mut front,
            &mut tail,
            move_buffer,
            history,
        );
        self.staged_legal_en_passant_moves(
            board,
            info,
            color,
            &mut front,
            &mut tail,
            move_buffer,
            history,
        );
        self.staged_legal_bishop_capture_moves(
            board,
            info,
            color,
            &mut front,
            &mut tail,
            move_buffer,
            history,
        );
        self.staged_legal_rook_capture_moves(
            board,
            info,
            color,
            &mut front,
            &mut tail,
            move_buffer,
            history,
        );
        self.staged_legal_queen_capture_moves(
            board,
            info,
            color,
            &mut front,
            &mut tail,
            move_buffer,
            history,
        );
        self.current_len = front;
        self.bad_capture_start = tail + 1; // the next stage will use this index as the starting tail index for bad captures
    }

    pub fn staged_legal_king_capture_moves(
        &self,
        board: &Board,
        color: Color,
        front: &mut usize,
        tail: &mut usize,
        move_buffer: &mut [ScoredMove],
        history: &HistoryTables,
    ) {
        let mut king = board.pieces(color, PieceType::King);
        let enemies = board.occupancy_of(color.opposite());
        let friends = board.occupancy_of(color);

        let Some(from) = pop_lsb(&mut king) else {
            board.print_board();
            panic!("No king at board.pieces(king)");
        };

        let targets = king_attacks(from) & !friends;

        let mut captures = targets & enemies;

        while let Some(to) = pop_lsb(&mut captures) {
            let mv = Move::new(from, to, MoveType::Capture, None);
            if self.is_special(mv) {
                continue;
            }
            if !legal_king_move(board, mv) {
                continue;
            }
            let captured = board
                .piecetype_at(to)
                .expect("No piece at target square for capture move");
            let see = see(board, mv, Some(captured));
            let history_key = HistoryKey::new(board.side_to_move(), PieceType::King, to);
            let history_score = history.get_capture_score(history_key, captured);
            debug_assert!(
                front < tail,
                "Front index {} is not less than tail index {}",
                *front,
                *tail
            );
            if see >= 0 {
                move_buffer[*front] = ScoredMove {
                    mv,
                    score: 900_000 + see + history_score,
                    history_key: Some(history_key),
                    see: Some(see),
                    captured: Some(captured),
                };
                *front += 1;
            } else {
                move_buffer[*tail] = ScoredMove {
                    mv,
                    score: -600_000 + see + history_score,
                    history_key: Some(history_key),
                    see: Some(see),
                    captured: Some(captured),
                };
                *tail -= 1;
            }
        }
    }

    pub fn staged_legal_knight_capture_moves(
        &self,
        board: &Board,
        info: &MoveGenInfo,
        color: Color,
        front: &mut usize,
        tail: &mut usize,
        move_buffer: &mut [ScoredMove],
        history: &HistoryTables,
    ) {
        let mut knights = board.pieces(color, PieceType::Knight);
        let enemies = board.occupancy_of(color.opposite());
        let friends = board.occupancy_of(color);

        while let Some(from) = pop_lsb(&mut knights) {
            let targets =
                knight_attacks(from) & !friends & info.pin_masks[from as usize] & info.check_mask;

            let mut captures = targets & enemies;

            while let Some(to) = pop_lsb(&mut captures) {
                let mv = Move::new(from, to, MoveType::Capture, None);
                if self.is_special(mv) {
                    continue;
                }
                let captured = board
                    .piecetype_at(to)
                    .expect("No piece at target square for capture move");
                let see = see(board, mv, Some(captured));
                let history_key = HistoryKey::new(color, PieceType::Knight, to);
                let history_score = history.get_capture_score(history_key, captured);
                debug_assert!(
                    *front < *tail,
                    "Front index {} is not less than tail index {}",
                    *front,
                    *tail
                );
                if see >= 0 {
                    move_buffer[*front] = ScoredMove {
                        mv,
                        score: 900_000 + see + history_score,
                        history_key: Some(history_key),
                        see: Some(see),
                        captured: Some(captured),
                    };
                    *front += 1;
                } else {
                    move_buffer[*tail] = ScoredMove {
                        mv,
                        score: -600_000 + see + history_score,
                        history_key: Some(history_key),
                        see: Some(see),
                        captured: Some(captured),
                    };
                    *tail -= 1;
                }
            }
        }
    }

    pub fn staged_legal_pawn_capture_moves(
        &self,
        board: &Board,
        info: &MoveGenInfo,
        color: Color,
        front: &mut usize,
        tail: &mut usize,
        move_buffer: &mut [ScoredMove],
        history: &HistoryTables,
    ) {
        let pawns = board.pieces(color, PieceType::Pawn);
        let enemies = board.occupancy_of(color.opposite());

        match color {
            Color::White => {
                let promotion_rank = 7;

                let mut captures_left = ((pawns & !FILE_A) << 7) & enemies;

                let mut captures_right = ((pawns & !FILE_H) << 9) & enemies;

                while let Some(to) = pop_lsb(&mut captures_left) {
                    let from = to - 7;

                    if !is_legal_pawn_move(to, from, info) {
                        continue;
                    }

                    if rank_of(to) == promotion_rank {
                        self.add_staged_capture_promotions(
                            board,
                            color,
                            from,
                            to,
                            front,
                            tail,
                            move_buffer,
                            history,
                        );
                        continue;
                    }

                    let mv = Move::new(from, to, MoveType::Capture, None);
                    if self.is_special(mv) {
                        continue;
                    }
                    let captured = board
                        .piecetype_at(to)
                        .expect("No piece at target square for capture move");
                    let see = see(board, mv, Some(captured));
                    let history_key = HistoryKey::new(color, PieceType::Pawn, to);
                    let history_score = history.get_capture_score(history_key, captured);
                    debug_assert!(
                        *front < *tail,
                        "Front index {} is not less than tail index {}",
                        *front,
                        *tail
                    );
                    if see >= 0 {
                        move_buffer[*front] = ScoredMove {
                            mv,
                            score: 900_000 + see + history_score,
                            history_key: Some(history_key),
                            see: Some(see),
                            captured: Some(captured),
                        };
                        *front += 1;
                    } else {
                        move_buffer[*tail] = ScoredMove {
                            mv,
                            score: -600_000 + see + history_score,
                            history_key: Some(history_key),
                            see: Some(see),
                            captured: Some(captured),
                        };
                        *tail -= 1;
                    }
                }

                while let Some(to) = pop_lsb(&mut captures_right) {
                    let from = to - 9;

                    if !is_legal_pawn_move(to, from, info) {
                        continue;
                    }

                    if rank_of(to) == promotion_rank {
                        self.add_staged_capture_promotions(
                            board,
                            color,
                            from,
                            to,
                            front,
                            tail,
                            move_buffer,
                            history,
                        );
                        continue;
                    }

                    let mv = Move::new(from, to, MoveType::Capture, None);
                    if self.is_special(mv) {
                        continue;
                    }
                    let captured = board
                        .piecetype_at(to)
                        .expect("No piece at target square for capture move");
                    let see = see(board, mv, Some(captured));
                    let history_key = HistoryKey::new(color, PieceType::Pawn, to);
                    let history_score = history.get_capture_score(history_key, captured);
                    debug_assert!(
                        *front < *tail,
                        "Front index {} is not less than tail index {}",
                        *front,
                        *tail
                    );
                    if see >= 0 {
                        move_buffer[*front] = ScoredMove {
                            mv,
                            score: 900_000 + see + history_score,
                            history_key: Some(history_key),
                            see: Some(see),
                            captured: Some(captured),
                        };
                        *front += 1;
                    } else {
                        move_buffer[*tail] = ScoredMove {
                            mv,
                            score: -600_000 + see + history_score,
                            history_key: Some(history_key),
                            see: Some(see),
                            captured: Some(captured),
                        };
                        *tail -= 1;
                    }
                }
            }

            Color::Black => {
                let promotion_rank = 0;

                let mut captures_left = ((pawns & !FILE_A) >> 9) & enemies;

                let mut captures_right = ((pawns & !FILE_H) >> 7) & enemies;

                while let Some(to) = pop_lsb(&mut captures_left) {
                    let from = to + 9;

                    if !is_legal_pawn_move(to, from, info) {
                        continue;
                    }

                    if rank_of(to) == promotion_rank {
                        self.add_staged_capture_promotions(
                            board,
                            color,
                            from,
                            to,
                            front,
                            tail,
                            move_buffer,
                            history,
                        );
                        continue;
                    }

                    let mv = Move::new(from, to, MoveType::Capture, None);
                    if self.is_special(mv) {
                        continue;
                    }
                    let captured = board
                        .piecetype_at(to)
                        .expect("No piece at target square for capture move");
                    let see = see(board, mv, Some(captured));
                    let history_key = HistoryKey::new(color, PieceType::Pawn, to);
                    let history_score = history.get_capture_score(history_key, captured);
                    debug_assert!(
                        *front < *tail,
                        "Front index {} is not less than tail index {}",
                        *front,
                        *tail
                    );
                    if see >= 0 {
                        move_buffer[*front] = ScoredMove {
                            mv,
                            score: 900_000 + see + history_score,
                            history_key: Some(history_key),
                            see: Some(see),
                            captured: Some(captured),
                        };
                        *front += 1;
                    } else {
                        move_buffer[*tail] = ScoredMove {
                            mv,
                            score: -600_000 + see + history_score,
                            history_key: Some(history_key),
                            see: Some(see),
                            captured: Some(captured),
                        };
                        *tail -= 1;
                    }
                }

                while let Some(to) = pop_lsb(&mut captures_right) {
                    let from = to + 7;

                    if !is_legal_pawn_move(to, from, info) {
                        continue;
                    }

                    if rank_of(to) == promotion_rank {
                        self.add_staged_capture_promotions(
                            board,
                            color,
                            from,
                            to,
                            front,
                            tail,
                            move_buffer,
                            history,
                        );
                        continue;
                    }

                    let mv = Move::new(from, to, MoveType::Capture, None);
                    if self.is_special(mv) {
                        continue;
                    }
                    let captured = board
                        .piecetype_at(to)
                        .expect("No piece at target square for capture move");
                    let see = see(board, mv, Some(captured));
                    let history_key = HistoryKey::new(color, PieceType::Pawn, to);
                    let history_score = history.get_capture_score(history_key, captured);
                    debug_assert!(
                        *front < *tail,
                        "Front index {} is not less than tail index {}",
                        *front,
                        *tail
                    );
                    if see >= 0 {
                        move_buffer[*front] = ScoredMove {
                            mv,
                            score: 900_000 + see + history_score,
                            history_key: Some(history_key),
                            see: Some(see),
                            captured: Some(captured),
                        };
                        *front += 1;
                    } else {
                        move_buffer[*tail] = ScoredMove {
                            mv,
                            score: -600_000 + see + history_score,
                            history_key: Some(history_key),
                            see: Some(see),
                            captured: Some(captured),
                        };
                        *tail -= 1;
                    }
                }
            }
        }
    }

    pub fn staged_legal_en_passant_moves(
        &self,
        board: &Board,
        info: &MoveGenInfo,
        color: Color,
        front: &mut usize,
        tail: &mut usize,
        move_buffer: &mut [ScoredMove],
        history: &HistoryTables,
    ) {
        let pawns = board.pieces(color, PieceType::Pawn);
        let king_sq = info.king_sq;

        match color {
            Color::White => {
                if let Some(en_pass_to) = board.en_passant() {
                    // en_pass_to is the target square(where the pawn will end up at)
                    let en_passant_to_bb = bit(en_pass_to);
                    let en_left = ((pawns & !FILE_A) << 7) & en_passant_to_bb;
                    let en_right = ((pawns & !FILE_H) << 9) & en_passant_to_bb;

                    if en_left != 0 {
                        // there is a pawn to the left
                        let mv = Move::new(en_pass_to - 7, en_pass_to, MoveType::EnPassant, None);

                        if legal_en_passant(board, mv, info, king_sq) && !self.is_special(mv) {
                            let captured = PieceType::Pawn; // The captured piece is always a pawn in en passant
                            let see = see(board, mv, Some(captured));
                            let history_key = HistoryKey::new(color, PieceType::Pawn, mv.to());
                            let history_score = history.get_capture_score(history_key, captured);
                            debug_assert!(
                                *front < *tail,
                                "Front index {} is not less than tail index {}",
                                *front,
                                *tail
                            );
                            if see >= 0 {
                                move_buffer[*front] = ScoredMove {
                                    mv,
                                    score: 900_000 + see + history_score,
                                    history_key: Some(history_key),
                                    see: Some(see),
                                    captured: Some(captured),
                                };
                                *front += 1;
                            } else {
                                move_buffer[*tail] = ScoredMove {
                                    mv,
                                    score: -600_000 + see + history_score,
                                    history_key: Some(history_key),
                                    see: Some(see),
                                    captured: Some(captured),
                                };
                                *tail -= 1;
                            }
                        }
                    }

                    if en_right != 0 {
                        // there is a pawn to the right
                        let mv = Move::new(en_pass_to - 9, en_pass_to, MoveType::EnPassant, None);

                        if legal_en_passant(board, mv, info, king_sq) && !self.is_special(mv) {
                            let captured = PieceType::Pawn; // The captured piece is always a pawn in en passant
                            let see = see(board, mv, Some(captured));
                            let history_key = HistoryKey::new(color, PieceType::Pawn, mv.to());
                            let history_score = history.get_capture_score(history_key, captured);
                            debug_assert!(
                                *front < *tail,
                                "Front index {} is not less than tail index {}",
                                *front,
                                *tail
                            );
                            if see >= 0 {
                                move_buffer[*front] = ScoredMove {
                                    mv,
                                    score: 900_000 + see + history_score,
                                    history_key: Some(history_key),
                                    see: Some(see),
                                    captured: Some(captured),
                                };
                                *front += 1;
                            } else {
                                move_buffer[*tail] = ScoredMove {
                                    mv,
                                    score: -600_000 + see + history_score,
                                    history_key: Some(history_key),
                                    see: Some(see),
                                    captured: Some(captured),
                                };
                                *tail -= 1;
                            }
                        }
                    }
                }
            }

            Color::Black => {
                if let Some(en_pass_to) = board.en_passant() {
                    // en_pass_to is the target square(where the pawn will end up at)
                    let en_passant_to_bb = bit(en_pass_to);
                    let en_left = ((pawns & !FILE_A) >> 9) & en_passant_to_bb;
                    let en_right = ((pawns & !FILE_H) >> 7) & en_passant_to_bb;

                    if en_left != 0 {
                        // there is a pawn to the left
                        let mv = Move::new(en_pass_to + 9, en_pass_to, MoveType::EnPassant, None);

                        if legal_en_passant(board, mv, info, king_sq) && !self.is_special(mv) {
                            let captured = PieceType::Pawn; // The captured piece is always a pawn in en passant
                            let see = see(board, mv, Some(captured));
                            let history_key = HistoryKey::new(color, PieceType::Pawn, mv.to());
                            let history_score = history.get_capture_score(history_key, captured);
                            debug_assert!(
                                *front < *tail,
                                "Front index {} is not less than tail index {}",
                                *front,
                                *tail
                            );
                            if see >= 0 {
                                move_buffer[*front] = ScoredMove {
                                    mv,
                                    score: 900_000 + see + history_score,
                                    history_key: Some(history_key),
                                    see: Some(see),
                                    captured: Some(captured),
                                };
                                *front += 1;
                            } else {
                                move_buffer[*tail] = ScoredMove {
                                    mv,
                                    score: -600_000 + see + history_score,
                                    history_key: Some(history_key),
                                    see: Some(see),
                                    captured: Some(captured),
                                };
                                *tail -= 1;
                            }
                        }
                    }

                    if en_right != 0 {
                        // there is a pawn to the right
                        let mv = Move::new(en_pass_to + 7, en_pass_to, MoveType::EnPassant, None);

                        if legal_en_passant(board, mv, info, king_sq) && !self.is_special(mv) {
                            let captured = PieceType::Pawn; // The captured piece is always a pawn in en passant
                            let see = see(board, mv, Some(captured));
                            let history_key = HistoryKey::new(color, PieceType::Pawn, mv.to());
                            let history_score = history.get_capture_score(history_key, captured);
                            debug_assert!(
                                *front < *tail,
                                "Front index {} is not less than tail index {}",
                                *front,
                                *tail
                            );
                            if see >= 0 {
                                move_buffer[*front] = ScoredMove {
                                    mv,
                                    score: 900_000 + see + history_score,
                                    history_key: Some(history_key),
                                    see: Some(see),
                                    captured: Some(captured),
                                };
                                *front += 1;
                            } else {
                                move_buffer[*tail] = ScoredMove {
                                    mv,
                                    score: -600_000 + see + history_score,
                                    history_key: Some(history_key),
                                    see: Some(see),
                                    captured: Some(captured),
                                };
                                *tail -= 1;
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn staged_legal_bishop_capture_moves(
        &self,
        board: &Board,
        info: &MoveGenInfo,
        color: Color,
        front: &mut usize,
        tail: &mut usize,
        move_buffer: &mut [ScoredMove],
        history: &HistoryTables,
    ) {
        let mut bishops = board.pieces(color, PieceType::Bishop);
        let enemies = board.occupancy_of(color.opposite());
        let friends = board.occupancy_of(color);
        let occupancy = board.all_occupancy();

        while let Some(from) = pop_lsb(&mut bishops) {
            let targets = bishop_attacks(from, occupancy)
                & !friends
                & info.pin_masks[from as usize]
                & info.check_mask;

            let mut captures = targets & enemies;

            while let Some(to) = pop_lsb(&mut captures) {
                let mv = Move::new(from, to, MoveType::Capture, None);
                if self.is_special(mv) {
                    continue;
                }

                let captured = board
                    .piecetype_at(to)
                    .expect("No piece at target square for capture move");
                let see = see(board, mv, Some(captured));
                let history_key = HistoryKey::new(board.side_to_move(), PieceType::Bishop, to);
                let history_score = history.get_capture_score(history_key, captured);
                debug_assert!(
                    *front < *tail,
                    "Front index {} is not less than tail index {}",
                    *front,
                    *tail
                );
                if see >= 0 {
                    move_buffer[*front] = ScoredMove {
                        mv,
                        score: 900_000 + see + history_score,
                        history_key: Some(history_key),
                        see: Some(see),
                        captured: Some(captured),
                    };
                    *front += 1;
                } else {
                    move_buffer[*tail] = ScoredMove {
                        mv,
                        score: -600_000 + see + history_score,
                        history_key: Some(history_key),
                        see: Some(see),
                        captured: Some(captured),
                    };
                    *tail -= 1;
                }
            }
        }
    }

    pub fn staged_legal_rook_capture_moves(
        &self,
        board: &Board,
        info: &MoveGenInfo,
        color: Color,
        front: &mut usize,
        tail: &mut usize,
        move_buffer: &mut [ScoredMove],
        history: &HistoryTables,
    ) {
        let mut rooks = board.pieces(color, PieceType::Rook);
        let enemies = board.occupancy_of(color.opposite());
        let friends = board.occupancy_of(color);
        let occupancy = board.all_occupancy();

        while let Some(from) = pop_lsb(&mut rooks) {
            let targets = rook_attacks(from, occupancy)
                & !friends
                & info.pin_masks[from as usize]
                & info.check_mask;

            let mut captures = targets & enemies;

            while let Some(to) = pop_lsb(&mut captures) {
                let mv = Move::new(from, to, MoveType::Capture, None);
                if self.is_special(mv) {
                    continue;
                }

                let captured = board
                    .piecetype_at(to)
                    .expect("No piece at target square for capture move");
                let see = see(board, mv, Some(captured));
                let history_key = HistoryKey::new(board.side_to_move(), PieceType::Rook, to);
                let history_score = history.get_capture_score(history_key, captured);
                debug_assert!(
                    *front < *tail,
                    "Front index {} is not less than tail index {}",
                    *front,
                    *tail
                );
                if see >= 0 {
                    move_buffer[*front] = ScoredMove {
                        mv,
                        score: 900_000 + see + history_score,
                        history_key: Some(history_key),
                        see: Some(see),
                        captured: Some(captured),
                    };
                    *front += 1;
                } else {
                    move_buffer[*tail] = ScoredMove {
                        mv,
                        score: -600_000 + see + history_score,
                        history_key: Some(history_key),
                        see: Some(see),
                        captured: Some(captured),
                    };
                    *tail -= 1;
                }
            }
        }
    }

    pub fn staged_legal_queen_capture_moves(
        &self,
        board: &Board,
        info: &MoveGenInfo,
        color: Color,
        front: &mut usize,
        tail: &mut usize,
        move_buffer: &mut [ScoredMove],
        history: &HistoryTables,
    ) {
        let mut queens = board.pieces(color, PieceType::Queen);
        let enemies = board.occupancy_of(color.opposite());
        let friends = board.occupancy_of(color);
        let occupancy = board.all_occupancy();

        while let Some(from) = pop_lsb(&mut queens) {
            let targets = queen_attacks(from, occupancy)
                & !friends
                & info.pin_masks[from as usize]
                & info.check_mask;

            let mut captures = targets & enemies;

            while let Some(to) = pop_lsb(&mut captures) {
                let mv = Move::new(from, to, MoveType::Capture, None);
                if self.is_special(mv) {
                    continue;
                }

                let captured = board
                    .piecetype_at(to)
                    .expect("No piece at target square for capture move");
                let see = see(board, mv, Some(captured));
                let history_key = HistoryKey::new(board.side_to_move(), PieceType::Queen, to);
                let history_score = history.get_capture_score(history_key, captured);
                debug_assert!(
                    *front < *tail,
                    "Front index {} is not less than tail index {}",
                    *front,
                    *tail
                );
                if see >= 0 {
                    move_buffer[*front] = ScoredMove {
                        mv,
                        score: 900_000 + see + history_score,
                        history_key: Some(history_key),
                        see: Some(see),
                        captured: Some(captured),
                    };
                    *front += 1;
                } else {
                    move_buffer[*tail] = ScoredMove {
                        mv,
                        score: -600_000 + see + history_score,
                        history_key: Some(history_key),
                        see: Some(see),
                        captured: Some(captured),
                    };
                    *tail -= 1;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::Board;
    use crate::engine::staged::staged::Mode;

    fn picker(side: Color) -> MovePicker {
        MovePicker {
            stage: Stage::GoodCaptures,
            mode: Mode::Main,
            current_len: 0,
            current_idx: 0,
            bad_quiet_start: MAX_MOVES,
            underpromo_start: MAX_MOVES,
            bad_capture_start: MAX_MOVES,
            picker: 0,
            side,
            ply: 0,
            pv_move: None,
            tt_move: None,
            killer_1: None,
            killer_2: None,
        }
    }

    #[test]
    fn capture_promotions_are_generated_as_captures_for_both_colors() {
        let cases = [
            (
                "r3k2r/1P4P1/8/8/8/8/8/4K3 w - - 0 1",
                Color::White,
                [(49, 56), (54, 63)],
            ),
            (
                "4k3/8/8/8/8/8/1p4p1/R3K2R b - - 0 1",
                Color::Black,
                [(9, 0), (14, 7)],
            ),
        ];

        for (fen, color, captures) in cases {
            let board = Board::from_fen(fen).expect("valid capture-promotion position");
            let info = MoveGenInfo::calculate(&board, color);
            let history = HistoryTables::new();
            let mut picker = picker(color);
            let mut move_buffer = [ScoredMove::new(); MAX_MOVES];

            picker.compute_captures(&board, &info, &mut move_buffer, &history);

            let generated: Vec<_> = move_buffer[..picker.current_len]
                .iter()
                .chain(move_buffer[picker.bad_capture_start..].iter())
                .copied()
                .collect();

            assert_eq!(generated.len(), captures.len() * PROMOTIONS.len());

            for (from, to) in captures {
                for promotion in PROMOTIONS {
                    let expected = Move::new(from, to, MoveType::Capture, Some(promotion));
                    let matching: Vec<_> = generated
                        .iter()
                        .filter(|scored| scored.mv == expected)
                        .collect();

                    assert_eq!(matching.len(), 1, "missing or duplicate {expected:?}");
                    assert_eq!(matching[0].captured, Some(PieceType::Rook));
                    assert!(matching[0].see.is_some());
                    assert_eq!(
                        matching[0].history_key.map(HistoryKey::idx),
                        Some(HistoryKey::new(color, PieceType::Pawn, to).idx())
                    );
                }
            }

            let mut promotion_buffer = [ScoredMove::new(); MAX_MOVES];
            let mut promotion_front = 0;
            let mut promotion_tail = MAX_MOVES - 1;
            picker.staged_legal_pawn_promotion_moves(
                &board,
                &info,
                color,
                &mut promotion_front,
                &mut promotion_tail,
                &mut promotion_buffer,
            );

            assert!(
                promotion_buffer[..promotion_front]
                    .iter()
                    .chain(promotion_buffer[promotion_tail + 1..].iter())
                    .all(|scored| scored.mv.kind() == MoveType::Normal),
                "the promotion stage must contain only quiet promotions"
            );
        }
    }
}
