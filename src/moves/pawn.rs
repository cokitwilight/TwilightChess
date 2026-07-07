use crate::bitboard::{
    Bitboard, FILE_A, FILE_H, RANK_1, RANK_3, RANK_6, RANK_8, Square, bit, pawn_attacks, pop_lsb,
    rank_of,
};
use crate::board::{Board, Move, MoveList, MoveType};
use crate::types::{Color, PieceType};

pub fn pseudo_pawn_moves(board: &Board, color: Color, moves: &mut MoveList) {
    let pawns = board.pieces(color, PieceType::Pawn);
    let occupancy = board.all_occupancy();
    let enemies = board.occupancy_of(color.opposite());
    let empty = !occupancy;

    match color {
        Color::White => {
            let mut single_pushes = (pawns << 8) & empty;

            let mut double_pushes = ((single_pushes & RANK_3) << 8) & empty;

            let mut captures_left = ((pawns & !FILE_A) << 7) & enemies;

            let mut captures_right = ((pawns & !FILE_H) << 9) & enemies;

            while let Some(to) = pop_lsb(&mut single_pushes) {
                let from = to - 8;

                add_pawn_move(moves, from, to, MoveType::Normal, color);
            }

            while let Some(to) = pop_lsb(&mut double_pushes) {
                let from = to - 16;

                moves.push(Move {
                    from,
                    to,
                    kind: MoveType::Normal,
                    promotion: None,
                });
            }

            while let Some(to) = pop_lsb(&mut captures_left) {
                let from = to - 7;

                add_pawn_move(moves, from, to, MoveType::Capture, color);
            }

            while let Some(to) = pop_lsb(&mut captures_right) {
                let from = to - 9;

                add_pawn_move(moves, from, to, MoveType::Capture, color);
            }

            if let Some(en_pass_to) = board.en_passant() {
                // en_pass_to is the target square(where the pawn will end up at)
                let en_passant_to_bb = bit(en_pass_to);
                let en_left = ((pawns & !FILE_A) << 7) & en_passant_to_bb;
                let en_right = ((pawns & !FILE_H) << 9) & en_passant_to_bb;

                if en_left != 0 {
                    // there is a pawn to the left
                    let mv = Move {
                        from: en_pass_to - 7,
                        to: en_pass_to,
                        kind: MoveType::EnPassant,
                        promotion: None,
                    };
                    moves.push(mv);
                }

                if en_right != 0 {
                    // there is a pawn to the right
                    let mv = Move {
                        from: en_pass_to - 9,
                        to: en_pass_to,
                        kind: MoveType::EnPassant,
                        promotion: None,
                    };
                    moves.push(mv);
                }
            }
        }

        Color::Black => {
            let mut single_pushes = (pawns >> 8) & empty;

            let mut double_pushes = ((single_pushes & RANK_6) >> 8) & empty;

            let mut captures_left = ((pawns & !FILE_A) >> 9) & enemies;

            let mut captures_right = ((pawns & !FILE_H) >> 7) & enemies;

            while let Some(to) = pop_lsb(&mut single_pushes) {
                let from = to + 8;

                add_pawn_move(moves, from, to, MoveType::Normal, color);
            }

            while let Some(to) = pop_lsb(&mut double_pushes) {
                let from = to + 16;

                moves.push(Move {
                    from,
                    to,
                    kind: MoveType::Normal,
                    promotion: None,
                });
            }

            while let Some(to) = pop_lsb(&mut captures_left) {
                let from = to + 9;

                add_pawn_move(moves, from, to, MoveType::Capture, color);
            }

            while let Some(to) = pop_lsb(&mut captures_right) {
                let from = to + 7;

                add_pawn_move(moves, from, to, MoveType::Capture, color);
            }

            if let Some(en_pass_to) = board.en_passant() {
                // en_pass_to is the target square(where the pawn will end up at)
                let en_passant_to_bb = bit(en_pass_to);
                let en_left = ((pawns & !FILE_A) >> 9) & en_passant_to_bb;
                let en_right = ((pawns & !FILE_H) >> 7) & en_passant_to_bb;

                if en_left != 0 {
                    // there is a pawn to the left
                    let mv = Move {
                        from: en_pass_to + 9,
                        to: en_pass_to,
                        kind: MoveType::EnPassant,
                        promotion: None,
                    };
                    moves.push(mv);
                }

                if en_right != 0 {
                    // there is a pawn to the right
                    let mv = Move {
                        from: en_pass_to + 7,
                        to: en_pass_to,
                        kind: MoveType::EnPassant,
                        promotion: None,
                    };
                    moves.push(mv);
                }
            }
        }
    }
}

// pub fn pseudo_pawn_moves_bb(board: &Board, color: Color) -> Bitboard {
//     let pawns = board.pieces(color, PieceType::Pawn);
//     let occupancy = board.all_occupancy();
//     let enemies = board.occupancy_of(color.opposite());
//     let empty = !occupancy;

//     let mut moves = 0u64;

//     match color {
//         Color::White => {
//             let single_pushes = (pawns << 8) & empty; // single pushes

//             moves |= single_pushes;

//             moves |= ((single_pushes & RANK_3) << 8) & empty; // double pushes

//             moves |= ((pawns & !FILE_A) << 7) & enemies; // captures left

//             moves |= ((pawns & !FILE_H) << 9) & enemies; // captures right

//             if let Some(en_pass_to) = board.en_passant() {
//                 // en_pass_to is the target square(where the pawn will end up at)
//                 let en_passant_to_bb = bit(en_pass_to);
//                 moves |= ((pawns & !FILE_A) << 7) & en_passant_to_bb;
//                 moves |= ((pawns & !FILE_H) << 9) & en_passant_to_bb;
//             }
//         }

//         Color::Black => {
//             let single_pushes = (pawns >> 8) & empty; // single pushes

//             moves |= single_pushes;

//             moves |= ((single_pushes & RANK_6) >> 8) & empty; // double pushes

//             moves |= ((pawns & !FILE_A) >> 9) & enemies; // captures left

//             moves |= ((pawns & !FILE_H) >> 7) & enemies; // captures right

//             if let Some(en_pass_to) = board.en_passant() {
//                 // en_pass_to is the target square(where the pawn will end up at)
//                 let en_passant_to_bb = bit(en_pass_to);
//                 moves |= ((pawns & !FILE_A) >> 9) & en_passant_to_bb;
//                 moves |= ((pawns & !FILE_H) >> 7) & en_passant_to_bb;
//             }
//         }
//     }
//     moves
// }

pub fn pseudo_pawn_moves_at(board: &Board, color: Color, sq: Square, moves: &mut MoveList) {
    if sq >= 64 {
        return;
    }

    let pawn = bit(sq);

    if board.pieces(color, PieceType::Pawn) & pawn == 0 {
        return;
    }

    let occupancy = board.all_occupancy();
    let enemies = board.occupancy_of(color.opposite());
    let empty = !occupancy;

    match color {
        Color::White => {
            let mut single_push = (pawn << 8) & empty;
            let mut double_push = ((single_push & RANK_3) << 8) & empty;

            let mut captures_left = ((pawn & !FILE_A) << 7) & enemies;
            let mut captures_right = ((pawn & !FILE_H) << 9) & enemies;

            while let Some(to) = pop_lsb(&mut single_push) {
                add_pawn_move(moves, sq, to, MoveType::Normal, color);
            }

            while let Some(to) = pop_lsb(&mut double_push) {
                moves.push(Move {
                    from: sq,
                    to,
                    kind: MoveType::Normal,
                    promotion: None,
                });
            }

            while let Some(to) = pop_lsb(&mut captures_left) {
                add_pawn_move(moves, sq, to, MoveType::Capture, color);
            }

            while let Some(to) = pop_lsb(&mut captures_right) {
                add_pawn_move(moves, sq, to, MoveType::Capture, color);
            }

            if let Some(en_pass_to) = board.en_passant() {
                let en_passant_to_bb = bit(en_pass_to);
                let en_left = ((pawn & !FILE_A) << 7) & en_passant_to_bb;
                let en_right = ((pawn & !FILE_H) << 9) & en_passant_to_bb;

                if en_left != 0 || en_right != 0 {
                    moves.push(Move {
                        from: sq,
                        to: en_pass_to,
                        kind: MoveType::EnPassant,
                        promotion: None,
                    });
                }
            }
        }

        Color::Black => {
            let mut single_push = (pawn >> 8) & empty;
            let mut double_push = ((single_push & RANK_6) >> 8) & empty;

            let mut captures_left = ((pawn & !FILE_A) >> 9) & enemies;
            let mut captures_right = ((pawn & !FILE_H) >> 7) & enemies;

            while let Some(to) = pop_lsb(&mut single_push) {
                add_pawn_move(moves, sq, to, MoveType::Normal, color);
            }

            while let Some(to) = pop_lsb(&mut double_push) {
                moves.push(Move {
                    from: sq,
                    to,
                    kind: MoveType::Normal,
                    promotion: None,
                });
            }

            while let Some(to) = pop_lsb(&mut captures_left) {
                add_pawn_move(moves, sq, to, MoveType::Capture, color);
            }

            while let Some(to) = pop_lsb(&mut captures_right) {
                add_pawn_move(moves, sq, to, MoveType::Capture, color);
            }

            if let Some(en_pass_to) = board.en_passant() {
                let en_passant_to_bb = bit(en_pass_to);
                let en_left = ((pawn & !FILE_A) >> 9) & en_passant_to_bb;
                let en_right = ((pawn & !FILE_H) >> 7) & en_passant_to_bb;

                if en_left != 0 || en_right != 0 {
                    moves.push(Move {
                        from: sq,
                        to: en_pass_to,
                        kind: MoveType::EnPassant,
                        promotion: None,
                    });
                }
            }
        }
    }
}

pub fn pseudo_pawn_capture_moves(board: &Board, color: Color, moves: &mut MoveList) {
    let pawns = board.pieces(color, PieceType::Pawn);
    let enemies = board.occupancy_of(color.opposite());
    let empty = !board.all_occupancy();

    let promotion_rank = match color {
        Color::White => 7,
        Color::Black => 0,
    };

    match color {
        Color::White => {
            let mut single_pushes = (pawns << 8) & empty;

            let mut captures_left = ((pawns & !FILE_A) << 7) & enemies;

            let mut captures_right = ((pawns & !FILE_H) << 9) & enemies;

            while let Some(to) = pop_lsb(&mut single_pushes) {
                let from = to - 8;
                if rank_of(to) == promotion_rank {
                    for promotion in [
                        PieceType::Queen,
                        PieceType::Rook,
                        PieceType::Bishop,
                        PieceType::Knight,
                    ] {
                        moves.push(Move {
                            from,
                            to,
                            kind: MoveType::Normal,
                            promotion: Some(promotion),
                        });
                    }
                }
            }

            while let Some(to) = pop_lsb(&mut captures_left) {
                let from = to - 7;

                add_pawn_move(moves, from, to, MoveType::Capture, color);
            }

            while let Some(to) = pop_lsb(&mut captures_right) {
                let from = to - 9;

                add_pawn_move(moves, from, to, MoveType::Capture, color);
            }

            if let Some(en_pass_to) = board.en_passant() {
                // en_pass_to is the target square(where the pawn will end up at)
                let en_passant_to_bb = bit(en_pass_to);
                let en_left = ((pawns & !FILE_A) << 7) & en_passant_to_bb;
                let en_right = ((pawns & !FILE_H) << 9) & en_passant_to_bb;

                if en_left != 0 {
                    // there is a pawn to the left
                    let mv = Move {
                        from: en_pass_to - 7,
                        to: en_pass_to,
                        kind: MoveType::EnPassant,
                        promotion: None,
                    };
                    moves.push(mv);
                }

                if en_right != 0 {
                    // there is a pawn to the right
                    let mv = Move {
                        from: en_pass_to - 9,
                        to: en_pass_to,
                        kind: MoveType::EnPassant,
                        promotion: None,
                    };
                    moves.push(mv);
                }
            }
        }

        Color::Black => {
            let mut single_pushes = (pawns >> 8) & empty;

            let mut captures_left = ((pawns & !FILE_A) >> 9) & enemies;

            let mut captures_right = ((pawns & !FILE_H) >> 7) & enemies;

            while let Some(to) = pop_lsb(&mut single_pushes) {
                let from = to + 8;
                if rank_of(to) == promotion_rank {
                    for promotion in [
                        PieceType::Queen,
                        PieceType::Rook,
                        PieceType::Bishop,
                        PieceType::Knight,
                    ] {
                        moves.push(Move {
                            from,
                            to,
                            kind: MoveType::Normal,
                            promotion: Some(promotion),
                        });
                    }
                }
            }

            while let Some(to) = pop_lsb(&mut captures_left) {
                let from = to + 9;

                add_pawn_move(moves, from, to, MoveType::Capture, color);
            }

            while let Some(to) = pop_lsb(&mut captures_right) {
                let from = to + 7;

                add_pawn_move(moves, from, to, MoveType::Capture, color);
            }

            if let Some(en_pass_to) = board.en_passant() {
                // en_pass_to is the target square(where the pawn will end up at)
                let en_passant_to_bb = bit(en_pass_to);
                let en_left = ((pawns & !FILE_A) >> 9) & en_passant_to_bb;
                let en_right = ((pawns & !FILE_H) >> 7) & en_passant_to_bb;

                if en_left != 0 {
                    // there is a pawn to the left
                    let mv = Move {
                        from: en_pass_to + 9,
                        to: en_pass_to,
                        kind: MoveType::EnPassant,
                        promotion: None,
                    };
                    moves.push(mv);
                }

                if en_right != 0 {
                    // there is a pawn to the right
                    let mv = Move {
                        from: en_pass_to + 7,
                        to: en_pass_to,
                        kind: MoveType::EnPassant,
                        promotion: None,
                    };
                    moves.push(mv);
                }
            }
        }
    }
}

pub fn pseudo_pawn_capture_moves_at(board: &Board, color: Color, sq: Square, moves: &mut MoveList) {
    if sq >= 64 {
        return;
    }

    let pawn = bit(sq);

    if board.pieces(color, PieceType::Pawn) & pawn == 0 {
        return;
    }

    let enemies = board.occupancy_of(color.opposite());

    match color {
        Color::White => {
            let mut captures_left = ((pawn & !FILE_A) << 7) & enemies;
            let mut captures_right = ((pawn & !FILE_H) << 9) & enemies;

            while let Some(to) = pop_lsb(&mut captures_left) {
                add_pawn_move(moves, sq, to, MoveType::Capture, color);
            }

            while let Some(to) = pop_lsb(&mut captures_right) {
                add_pawn_move(moves, sq, to, MoveType::Capture, color);
            }

            if let Some(en_pass_to) = board.en_passant() {
                let en_passant_to_bb = bit(en_pass_to);
                let en_left = ((pawn & !FILE_A) << 7) & en_passant_to_bb;
                let en_right = ((pawn & !FILE_H) << 9) & en_passant_to_bb;

                if en_left != 0 || en_right != 0 {
                    moves.push(Move {
                        from: sq,
                        to: en_pass_to,
                        kind: MoveType::EnPassant,
                        promotion: None,
                    });
                }
            }
        }

        Color::Black => {
            let mut captures_left = ((pawn & !FILE_A) >> 9) & enemies;
            let mut captures_right = ((pawn & !FILE_H) >> 7) & enemies;

            while let Some(to) = pop_lsb(&mut captures_left) {
                add_pawn_move(moves, sq, to, MoveType::Capture, color);
            }

            while let Some(to) = pop_lsb(&mut captures_right) {
                add_pawn_move(moves, sq, to, MoveType::Capture, color);
            }

            if let Some(en_pass_to) = board.en_passant() {
                let en_passant_to_bb = bit(en_pass_to);
                let en_left = ((pawn & !FILE_A) >> 9) & en_passant_to_bb;
                let en_right = ((pawn & !FILE_H) >> 7) & en_passant_to_bb;

                if en_left != 0 || en_right != 0 {
                    moves.push(Move {
                        from: sq,
                        to: en_pass_to,
                        kind: MoveType::EnPassant,
                        promotion: None,
                    });
                }
            }
        }
    }
}
fn add_pawn_move(moves: &mut MoveList, from: Square, to: Square, kind: MoveType, color: Color) {
    let to_bb = bit(to);

    let promotes = match color {
        Color::White => to_bb & RANK_8 != 0,
        Color::Black => to_bb & RANK_1 != 0,
    };

    if promotes {
        for promotion in [
            PieceType::Queen,
            PieceType::Rook,
            PieceType::Bishop,
            PieceType::Knight,
        ] {
            moves.push(Move {
                from,
                to,
                kind,
                promotion: Some(promotion),
            });
        }
    } else {
        moves.push(Move {
            from,
            to,
            kind,
            promotion: None,
        });
    }
}
