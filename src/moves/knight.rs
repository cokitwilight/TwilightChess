use crate::bitboard::{knight_attacks, pop_lsb};
use crate::board::{Board, Move, MoveList, MoveType};
use crate::moves::MoveGenInfo;
use crate::types::{Color, PieceType};

pub fn pseudo_knight_moves(board: &Board, color: Color, moves: &mut MoveList) {
    let mut knights = board.pieces(color, PieceType::Knight);
    let enemies = board.occupancy_of(color.opposite());
    let friends = board.occupancy_of(color);
    let empty = !(enemies | friends);

    while let Some(from) = pop_lsb(&mut knights) {
        let targets = knight_attacks(from) & !friends;

        let mut captures = targets & enemies;
        let mut quiets = targets & empty;

        while let Some(to) = pop_lsb(&mut captures) {
            moves.push(Move::new(from, to, MoveType::Capture, None));
        }
        while let Some(to) = pop_lsb(&mut quiets) {
            moves.push(Move::new(from, to, MoveType::Normal, None));
        }
    }
}

pub fn legal_knight_moves(board: &Board, color: Color, info: &MoveGenInfo, moves: &mut MoveList) {
    let mut knights = board.pieces(color, PieceType::Knight);
    let enemies = board.occupancy_of(color.opposite());
    let friends = board.occupancy_of(color);
    let empty = !(enemies | friends);

    while let Some(from) = pop_lsb(&mut knights) {
        let targets =
            knight_attacks(from) & !friends & info.pin_masks[from as usize] & info.check_mask;

        let mut captures = targets & enemies;
        let mut quiets = targets & empty;

        while let Some(to) = pop_lsb(&mut captures) {
            moves.push(Move::new(from, to, MoveType::Capture, None));
        }
        while let Some(to) = pop_lsb(&mut quiets) {
            moves.push(Move::new(from, to, MoveType::Normal, None));
        }
    }
}

pub fn pseudo_knight_capture_moves(board: &Board, color: Color, moves: &mut MoveList) {
    let mut knights = board.pieces(color, PieceType::Knight);
    let enemies = board.occupancy_of(color.opposite());
    let friends = board.occupancy_of(color);

    while let Some(from) = pop_lsb(&mut knights) {
        let targets = knight_attacks(from) & !friends;

        let mut captures = targets & enemies;

        while let Some(to) = pop_lsb(&mut captures) {
            moves.push(Move::new(from, to, MoveType::Capture, None));
        }
    }
}

pub fn legal_knight_capture_moves(
    board: &Board,
    color: Color,
    info: &MoveGenInfo,
    moves: &mut MoveList,
) {
    let mut knights = board.pieces(color, PieceType::Knight);
    let enemies = board.occupancy_of(color.opposite());
    let friends = board.occupancy_of(color);

    while let Some(from) = pop_lsb(&mut knights) {
        let targets =
            knight_attacks(from) & !friends & info.pin_masks[from as usize] & info.check_mask;

        let mut captures = targets & enemies;

        while let Some(to) = pop_lsb(&mut captures) {
            moves.push(Move::new(from, to, MoveType::Capture, None));
        }
    }
}

pub fn legal_knight_quiet_moves(
    board: &Board,
    color: Color,
    info: &MoveGenInfo,
    moves: &mut MoveList,
) {
    let mut knights = board.pieces(color, PieceType::Knight);
    let enemies = board.occupancy_of(color.opposite());
    let friends = board.occupancy_of(color);

    while let Some(from) = pop_lsb(&mut knights) {
        let targets =
            knight_attacks(from) & !friends & info.pin_masks[from as usize] & info.check_mask;

        let mut quiets = targets & !enemies;

        while let Some(to) = pop_lsb(&mut quiets) {
            moves.push(Move::new(from, to, MoveType::Normal, None));
        }
    }
}
