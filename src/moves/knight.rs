use crate::bitboard::{Square, bit, knight_attacks, pop_lsb};
use crate::board::{Board, Move, MoveList, MoveType};
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
            moves.push(Move {
                from,
                to,
                kind: MoveType::Capture,
                promotion: None,
            });
        }
        while let Some(to) = pop_lsb(&mut quiets) {
            moves.push(Move {
                from,
                to,
                kind: MoveType::Normal,
                promotion: None,
            });
        }
    }
}

pub fn pseudo_knight_moves_at(board: &Board, color: Color, sq: Square, moves: &mut MoveList) {
    if sq >= 64 {
        return;
    }

    let knights = board.pieces(color, PieceType::Knight) & bit(sq);

    if knights == 0 {
        return;
    }

    let enemies = board.occupancy_of(color.opposite());
    let friends = board.occupancy_of(color);

    let targets = knight_attacks(sq) & !friends;

    let mut captures = targets & enemies;
    let mut quiets = targets & !enemies;

    while let Some(to) = pop_lsb(&mut captures) {
        moves.push(Move {
            from: sq,
            to,
            kind: MoveType::Capture,
            promotion: None,
        });
    }
    while let Some(to) = pop_lsb(&mut quiets) {
        moves.push(Move {
            from: sq,
            to,
            kind: MoveType::Normal,
            promotion: None,
        });
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
            moves.push(Move {
                from,
                to,
                kind: MoveType::Capture,
                promotion: None,
            });
        }
    }
}
