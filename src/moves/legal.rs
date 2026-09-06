use crate::bitboard::pins::{generate_checkers_and_check_mask, generate_pin_masks};
use crate::bitboard::{Bitboard, Square, pop_lsb};
use crate::board::{Board, MoveList, MoveType};
use crate::moves::king::{legal_king_capture_moves, legal_king_moves, legal_king_quiet_moves};
use crate::moves::knight::{
    legal_knight_capture_moves, legal_knight_moves, legal_knight_quiet_moves,
};
use crate::moves::pawn::{
    legal_en_passant_moves, legal_pawn_capture_moves, legal_pawn_moves, legal_pawn_promotion_moves,
    legal_pawn_quiet_moves,
};
use crate::moves::sliders::{
    legal_bishop_capture_moves, legal_bishop_moves, legal_bishop_quiet_moves,
    legal_queen_capture_moves, legal_queen_moves, legal_queen_quiet_moves,
    legal_rook_capture_moves, legal_rook_moves, legal_rook_quiet_moves,
};
use crate::types::{Color, PieceType};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MoveGenInfo {
    pub king_sq: Square,
    pub pin_masks: [Bitboard; 64],
    pub checkers: Bitboard,
    pub check_mask: Bitboard,
    // pub enemy_attacks // maybe
}

impl MoveGenInfo {
    pub fn calculate(board: &Board, color: Color) -> Self {
        let king_sq = pop_lsb(&mut board.pieces(color, PieceType::King))
            .expect("No king in MoveGenInfo Constructor");

        let pin_masks = generate_pin_masks(board, color, king_sq);
        let (checkers, check_mask) = generate_checkers_and_check_mask(board, color, king_sq);

        // Important: usually computed with king removed from occupancy.
        // let enemy_attacks = generate_enemy_attacks_for_king_safety(board, color);

        Self {
            king_sq,
            pin_masks,
            checkers,
            check_mask,
            // enemy_attacks,
        }
    }
}

pub fn all_legal_moves(board: &mut Board, color: Color, moves: &mut MoveList) {
    debug_assert_eq!(
        board.side_to_move, color,
        "all_legal_moves called with color != board.side_to_move"
    );

    let info = MoveGenInfo::calculate(board, color);

    if info.checkers.count_ones() > 1 {
        legal_king_moves(board, color, moves);
        return;
    }

    // these are special cases since en passant and king double checks are special
    // Currently the legal_move functions check legality.
    legal_en_passant_moves(board, color, &info, moves);
    legal_king_moves(board, color, moves);

    legal_pawn_moves(board, color, &info, moves); // DOES NOT INCLUDE EN PASSANT
    legal_knight_moves(board, color, &info, moves);
    legal_bishop_moves(board, color, &info, moves);
    legal_rook_moves(board, color, &info, moves);
    legal_queen_moves(board, color, &info, moves);
}

pub fn all_legal_capture_moves(board: &mut Board, color: Color, moves: &mut MoveList) {
    debug_assert_eq!(
        board.side_to_move, color,
        "all_legal_capture_moves called with color != board.side_to_move"
    );

    let info = MoveGenInfo::calculate(board, color);

    legal_king_capture_moves(board, color, moves);

    if info.checkers.count_ones() > 1 {
        return;
    }

    legal_en_passant_moves(board, color, &info, moves);
    legal_pawn_capture_moves(board, color, &info, moves);

    let mut promotions = MoveList::new();
    legal_pawn_promotion_moves(board, color, &info, &mut promotions);
    for &mv in promotions.iter() {
        if mv.kind() == MoveType::Capture {
            moves.push(mv);
        }
    }

    legal_knight_capture_moves(board, color, &info, moves);
    legal_bishop_capture_moves(board, color, &info, moves);
    legal_rook_capture_moves(board, color, &info, moves);
    legal_queen_capture_moves(board, color, &info, moves);
}

pub fn all_legal_quiet_moves(board: &mut Board, color: Color, moves: &mut MoveList) {
    debug_assert_eq!(
        board.side_to_move, color,
        "all_legal_quiet_moves called with color != board.side_to_move"
    );

    let info = MoveGenInfo::calculate(board, color);

    legal_king_quiet_moves(board, color, moves);

    if info.checkers.count_ones() > 1 {
        return;
    }

    legal_pawn_quiet_moves(board, color, &info, moves);

    let mut promotions = MoveList::new();
    legal_pawn_promotion_moves(board, color, &info, &mut promotions);
    for &mv in promotions.iter() {
        if mv.kind() == MoveType::Normal {
            moves.push(mv);
        }
    }

    legal_knight_quiet_moves(board, color, &info, moves);
    legal_bishop_quiet_moves(board, color, &info, moves);
    legal_rook_quiet_moves(board, color, &info, moves);
    legal_queen_quiet_moves(board, color, &info, moves);
}
