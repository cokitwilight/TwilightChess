use crate::bitboard::pins::{generate_checkers_and_check_mask, generate_pin_masks};
use crate::bitboard::{Bitboard, Square, pop_lsb};
use crate::board::{Board, MoveList};
use crate::moves::king::{pseudo_king_capture_moves, pseudo_king_moves};
use crate::moves::knight::{legal_knight_capture_moves, legal_knight_moves};
use crate::moves::pawn::{legal_pawn_capture_moves, legal_pawn_moves};
use crate::moves::pseudo::{all_pseudo_capture_moves, all_pseudo_moves, all_pseudo_moves_at};
use crate::moves::sliders::{
    legal_bishop_capture_moves, legal_bishop_moves, legal_queen_capture_moves, legal_queen_moves,
    legal_rook_capture_moves, legal_rook_moves,
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

    let mut all_moves = MoveList::new();

    if info.checkers.count_ones() > 1 {
        pseudo_king_moves(board, color, &mut all_moves);
        for &mv in all_moves.iter() {
            let undo = board.make_move(mv);

            let is_legal = !board.in_check(color);

            board.undo_move(undo);

            if is_legal {
                moves.push(mv);
            }
        }
        return;
    }

    // these are special cases since en passant and king double checks so keep make/undo for now
    legal_pawn_moves(board, color, &mut all_moves, &info);
    pseudo_king_moves(board, color, &mut all_moves);

    for &mv in all_moves.iter() {
        let undo = board.make_move(mv);

        let is_legal = !board.in_check(color);

        board.undo_move(undo);

        if is_legal {
            moves.push(mv);
        }
    }

    legal_knight_moves(board, color, moves, &info);
    legal_bishop_moves(board, color, moves, &info);
    legal_rook_moves(board, color, moves, &info);
    legal_queen_moves(board, color, moves, &info);
}

pub fn all_legal_moves_at(board: &mut Board, color: Color, sq: Square, moves: &mut MoveList) {
    debug_assert_eq!(
        board.side_to_move, color,
        "all_legal_moves called with color != board.side_to_move"
    );

    let mut pseudo_moves = MoveList::new();

    all_pseudo_moves_at(board, color, sq, &mut pseudo_moves);

    for &mv in pseudo_moves.iter() {
        let undo = board.make_move(mv);

        let is_legal = !board.in_check(color);

        board.undo_move(undo);

        if is_legal {
            moves.push(mv);
        }
    }
}

pub fn all_legal_capture_moves(board: &mut Board, color: Color, moves: &mut MoveList) {
    debug_assert_eq!(
        board.side_to_move, color,
        "all_legal_moves called with color != board.side_to_move"
    );

    let info = MoveGenInfo::calculate(board, color);

    let mut all_moves = MoveList::new();

    if info.checkers.count_ones() > 1 {
        pseudo_king_moves(board, color, &mut all_moves);
        for &mv in all_moves.iter() {
            let undo = board.make_move(mv);

            let is_legal = !board.in_check(color);

            board.undo_move(undo);

            if is_legal {
                moves.push(mv);
            }
        }
        return;
    }

    // these are special cases since en passant and king double checks so keep make/undo for now
    legal_pawn_capture_moves(board, color, &mut all_moves, &info);
    pseudo_king_capture_moves(board, color, &mut all_moves);

    for &mv in all_moves.iter() {
        let undo = board.make_move(mv);

        let is_legal = !board.in_check(color);

        board.undo_move(undo);

        if is_legal {
            moves.push(mv);
        }
    }

    legal_knight_capture_moves(board, color, moves, &info);
    legal_bishop_capture_moves(board, color, moves, &info);
    legal_rook_capture_moves(board, color, moves, &info);
    legal_queen_capture_moves(board, color, moves, &info);
}
