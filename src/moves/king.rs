use crate::bitboard::{
    A1, A8, B1, B8, C1, C8, D1, D8, E1, E8, F1, F8, G1, G8, H1, H8, bishop_attacks, bit,
    king_attacks, knight_attacks, pawn_attacks_from_square, pop_lsb, rook_attacks,
};
use crate::board::{
    BLACK_KINGSIDE, BLACK_QUEENSIDE, Board, Move, MoveList, MoveType, WHITE_KINGSIDE,
    WHITE_QUEENSIDE,
};
use crate::types::{Color, PieceType};

pub fn legal_king_moves(board: &Board, color: Color, moves: &mut MoveList) {
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
        let mv = Move::new(from, to, MoveType::Capture, None);
        if legal_king_move(board, mv) {
            moves.push(mv);
        }
    }
    while let Some(to) = pop_lsb(&mut quiets) {
        let mv = Move::new(from, to, MoveType::Normal, None);
        if legal_king_move(board, mv) {
            moves.push(mv);
        }
    }
    legal_castling_moves(board, color, moves);
}

pub fn legal_king_capture_moves(board: &Board, color: Color, moves: &mut MoveList) {
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
        if legal_king_move(board, mv) {
            moves.push(mv);
        }
    }
}

pub fn legal_king_quiet_moves(board: &Board, color: Color, moves: &mut MoveList) {
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
        if legal_king_move(board, mv) {
            moves.push(mv);
        }
    }
    legal_castling_moves(board, color, moves);
}

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
        moves.push(Move::new(from, to, MoveType::Capture, None));
    }
    while let Some(to) = pop_lsb(&mut quiets) {
        moves.push(Move::new(from, to, MoveType::Normal, None));
    }
    pseudo_castling_moves(board, color, moves);
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
        moves.push(Move::new(from, to, MoveType::Capture, None));
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

                let path_safe =
                    !board.square_attacked_by(enemy, F1) && !board.square_attacked_by(enemy, G1);

                if rook_on_h1 && path_empty && path_safe {
                    moves.push(Move::new(E1, G1, MoveType::Castle, None));
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
                    !board.square_attacked_by(enemy, D1) && !board.square_attacked_by(enemy, C1);

                if rook_on_a1 && path_empty && path_safe {
                    moves.push(Move::new(E1, C1, MoveType::Castle, None));
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

                let path_safe =
                    !board.square_attacked_by(enemy, F8) && !board.square_attacked_by(enemy, G8);

                if rook_on_h8 && path_empty && path_safe {
                    moves.push(Move::new(E8, G8, MoveType::Castle, None));
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
                    !board.square_attacked_by(enemy, D8) && !board.square_attacked_by(enemy, C8);

                if rook_on_a8 && path_empty && path_safe {
                    moves.push(Move::new(E8, C8, MoveType::Castle, None));
                }
            }
        }
    }
}

fn legal_castling_moves(board: &Board, color: Color, moves: &mut MoveList) {
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

                let path_safe =
                    !board.square_attacked_by(enemy, F1) && !board.square_attacked_by(enemy, G1);

                if rook_on_h1 && path_empty && path_safe {
                    let mv = Move::new(E1, G1, MoveType::Castle, None);
                    if legal_king_move(board, mv) {
                        moves.push(mv);
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
                let path_safe =
                    !board.square_attacked_by(enemy, D1) && !board.square_attacked_by(enemy, C1);

                if rook_on_a1 && path_empty && path_safe {
                    let mv = Move::new(E1, C1, MoveType::Castle, None);
                    if legal_king_move(board, mv) {
                        moves.push(mv);
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

                let path_safe =
                    !board.square_attacked_by(enemy, F8) && !board.square_attacked_by(enemy, G8);

                if rook_on_h8 && path_empty && path_safe {
                    let mv = Move::new(E8, G8, MoveType::Castle, None);
                    if legal_king_move(board, mv) {
                        moves.push(mv);
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
                let path_safe =
                    !board.square_attacked_by(enemy, D8) && !board.square_attacked_by(enemy, C8);

                if rook_on_a8 && path_empty && path_safe {
                    let mv = Move::new(E8, C8, MoveType::Castle, None);
                    if legal_king_move(board, mv) {
                        moves.push(mv);
                    }
                }
            }
        }
    }
}

fn legal_king_move(board: &Board, mv: Move) -> bool {
    // NOTE: Castling is already checked in the move generator for moving across checks
    let from = mv.from();
    let to = mv.to();

    let side_to_move = board.side_to_move();
    let enemy = side_to_move.opposite();

    // FOR DEBUG
    let piece = board.piece_at(from).expect("NO PIECE IN LEGAL_KING_MOVE!");

    debug_assert_eq!(side_to_move, piece.color);
    debug_assert_eq!(PieceType::King, piece.kind);

    let mut occ = board.all_occupancy();

    let target = bit(to);

    occ &= !bit(from);
    occ |= target;

    let pawn_attacks = pawn_attacks_from_square(side_to_move, to);

    if board.pieces(enemy, PieceType::Pawn) & pawn_attacks != 0 {
        return false;
    }

    let knight_attacks = knight_attacks(to);

    if board.pieces(enemy, PieceType::Knight) & knight_attacks != 0 {
        return false;
    }

    if king_attacks(to) & board.pieces(enemy, PieceType::King) != 0 {
        return false;
    }

    let diagonals = bishop_attacks(to, occ); // this does not include mv.to() so there is no need modifying per piece bitboards
    let straights = rook_attacks(to, occ);

    let bishops = board.pieces(enemy, PieceType::Bishop);
    let queens = board.pieces(enemy, PieceType::Queen);
    let rooks = board.pieces(enemy, PieceType::Rook);

    if diagonals & (bishops | queens) != 0 {
        return false;
    }

    if straights & (rooks | queens) != 0 {
        return false;
    }

    true
}
