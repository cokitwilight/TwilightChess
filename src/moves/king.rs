use crate::bitboard::{
    A1, A8, B1, B8, C1, C8, D1, D8, E1, E8, F1, F8, G1, G8, H1, H8, Square, bit, king_attacks,
    pop_lsb,
};
use crate::board::{
    BLACK_KINGSIDE, BLACK_QUEENSIDE, Board, Move, MoveList, MoveType, WHITE_KINGSIDE,
    WHITE_QUEENSIDE,
};
use crate::moves::MoveGenInfo;
use crate::types::{Color, PieceType};

pub fn pseudo_king_moves(board: &Board, color: Color, moves: &mut MoveList) {
    let mut king = board.pieces(color, PieceType::King);
    let enemies = board.occupancy_of(color.opposite());
    let friends = board.occupancy_of(color);
    let empty = !(enemies | friends);

    let Some(from) = pop_lsb(&mut king) else {
        board.print_board();
        panic!("No king at board.pieces(king)");
    };

    let targets = king_attacks(from) & !friends;

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
    pseudo_castling_moves(board, color, moves);
}

pub fn pseudo_king_moves_at(board: &Board, color: Color, sq: Square, moves: &mut MoveList) {
    if sq >= 64 {
        return;
    }

    if board.pieces(color, PieceType::King) & bit(sq) == 0 {
        return;
    }

    let enemies = board.occupancy_of(color.opposite());
    let friends = board.occupancy_of(color);
    let empty = !(enemies | friends);

    let targets = king_attacks(sq) & !friends;

    let mut captures = targets & enemies;
    let mut quiets = targets & empty;

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

    // Only produces castles if this king is actually on E1/E8.
    pseudo_castling_moves_at(board, color, sq, moves);
}

pub fn pseudo_king_capture_moves(board: &Board, color: Color, moves: &mut MoveList) {
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
        moves.push(Move {
            from,
            to,
            kind: MoveType::Capture,
            promotion: None,
        });
    }
}

pub fn pseudo_king_capture_moves_at(board: &Board, color: Color, sq: Square, moves: &mut MoveList) {
    if sq >= 64 {
        return;
    }

    if board.pieces(color, PieceType::King) & bit(sq) == 0 {
        return;
    }

    let enemies = board.occupancy_of(color.opposite());
    let friends = board.occupancy_of(color);

    let targets = king_attacks(sq) & !friends;

    let mut captures = targets & enemies;

    while let Some(to) = pop_lsb(&mut captures) {
        moves.push(Move {
            from: sq,
            to,
            kind: MoveType::Capture,
            promotion: None,
        });
    }
}

fn pseudo_castling_moves(board: &Board, color: Color, moves: &mut MoveList) {
    let enemy = color.opposite();
    let occupied = board.all_occupancy();

    match color {
        Color::White => {
            // White king must be on e1.
            if board.pieces(Color::White, PieceType::King) != bit(E1) {
                return;
            }

            // Cannot castle while in check.
            if board.square_attacked_by(E1, enemy) {
                return;
            }

            // -------------------------
            // White kingside: e1 -> g1, rook h1 -> f1
            // -------------------------
            if board.has_castling_right(WHITE_KINGSIDE) {
                let between = bit(F1) | bit(G1);

                let rook_on_h1 = board.pieces(Color::White, PieceType::Rook) & bit(H1) != 0;

                let path_empty = occupied & between == 0;

                let path_safe =
                    !board.square_attacked_by(F1, enemy) && !board.square_attacked_by(G1, enemy);

                if rook_on_h1 && path_empty && path_safe {
                    moves.push(Move {
                        from: E1,
                        to: G1,
                        kind: MoveType::Castle,
                        promotion: None,
                    });
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
                let path_safe =
                    !board.square_attacked_by(D1, enemy) && !board.square_attacked_by(C1, enemy);

                if rook_on_a1 && path_empty && path_safe {
                    moves.push(Move {
                        from: E1,
                        to: C1,
                        kind: MoveType::Castle,
                        promotion: None,
                    });
                }
            }
        }

        Color::Black => {
            // Black king must be on e8.
            if board.pieces(Color::Black, PieceType::King) != bit(E8) {
                return;
            }

            // Cannot castle while in check.
            if board.square_attacked_by(E8, enemy) {
                return;
            }

            // -------------------------
            // Black kingside: e8 -> g8, rook h8 -> f8
            // -------------------------
            if board.has_castling_right(BLACK_KINGSIDE) {
                let between = bit(F8) | bit(G8);

                let rook_on_h8 = board.pieces(Color::Black, PieceType::Rook) & bit(H8) != 0;

                let path_empty = occupied & between == 0;

                let path_safe =
                    !board.square_attacked_by(F8, enemy) && !board.square_attacked_by(G8, enemy);

                if rook_on_h8 && path_empty && path_safe {
                    moves.push(Move {
                        from: E8,
                        to: G8,
                        kind: MoveType::Castle,
                        promotion: None,
                    });
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
                let path_safe =
                    !board.square_attacked_by(D8, enemy) && !board.square_attacked_by(C8, enemy);

                if rook_on_a8 && path_empty && path_safe {
                    moves.push(Move {
                        from: E8,
                        to: C8,
                        kind: MoveType::Castle,
                        promotion: None,
                    });
                }
            }
        }
    }
}

fn pseudo_castling_moves_at(board: &Board, color: Color, sq: Square, moves: &mut MoveList) {
    match color {
        Color::White if sq == E1 => pseudo_castling_moves(board, color, moves),
        Color::Black if sq == E8 => pseudo_castling_moves(board, color, moves),
        _ => {}
    }
}
