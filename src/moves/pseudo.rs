use crate::board::{Board, MoveList};
use crate::moves::king::pseudo_king_capture_moves;
use crate::moves::king::pseudo_king_moves;
use crate::moves::knight::pseudo_knight_capture_moves;
use crate::moves::knight::pseudo_knight_moves;
use crate::moves::pawn::pseudo_pawn_capture_moves;
use crate::moves::pawn::pseudo_pawn_moves;
use crate::moves::sliders::{
    pseudo_bishop_capture_moves, pseudo_queen_capture_moves, pseudo_rook_capture_moves,
};
use crate::moves::sliders::{pseudo_bishop_moves, pseudo_queen_moves, pseudo_rook_moves};
use crate::types::Color;

pub fn all_pseudo_moves(board: &Board, color: Color, moves: &mut MoveList) {
    pseudo_pawn_moves(board, color, moves);
    pseudo_knight_moves(board, color, moves);
    pseudo_bishop_moves(board, color, moves);
    pseudo_rook_moves(board, color, moves);
    pseudo_queen_moves(board, color, moves);
    pseudo_king_moves(board, color, moves);
}

pub fn all_pseudo_capture_moves(board: &mut Board, color: Color, moves: &mut MoveList) {
    pseudo_pawn_capture_moves(board, color, moves);
    pseudo_knight_capture_moves(board, color, moves);
    pseudo_bishop_capture_moves(board, color, moves);
    pseudo_rook_capture_moves(board, color, moves);
    pseudo_queen_capture_moves(board, color, moves);
    pseudo_king_capture_moves(board, color, moves);
}
