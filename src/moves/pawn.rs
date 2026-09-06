use crate::bitboard::{
    FILE_A, FILE_H, RANK_1, RANK_3, RANK_6, RANK_8, Square, bishop_attacks, bit, file_of, pop_lsb,
    rank_of, rook_attacks, square,
};
use crate::board::{Board, Move, MoveList, MoveType};
use crate::moves::MoveGenInfo;
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

                moves.push(Move::new(from, to, MoveType::Normal, None));
            }

            while let Some(to) = pop_lsb(&mut captures_left) {
                let from = to - 7;

                add_pawn_move(moves, from, to, MoveType::Capture, color);
            }

            while let Some(to) = pop_lsb(&mut captures_right) {
                let from = to - 9;

                add_pawn_move(moves, from, to, MoveType::Capture, color);
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

                moves.push(Move::new(from, to, MoveType::Normal, None));
            }

            while let Some(to) = pop_lsb(&mut captures_left) {
                let from = to + 9;

                add_pawn_move(moves, from, to, MoveType::Capture, color);
            }

            while let Some(to) = pop_lsb(&mut captures_right) {
                let from = to + 7;

                add_pawn_move(moves, from, to, MoveType::Capture, color);
            }
        }
    }
}

pub fn legal_pawn_moves(board: &Board, color: Color, info: &MoveGenInfo, moves: &mut MoveList) {
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

                if is_legal_pawn_move(to, from, info) {
                    add_pawn_move(moves, from, to, MoveType::Normal, color);
                }
            }

            while let Some(to) = pop_lsb(&mut double_pushes) {
                let from = to - 16;

                if is_legal_pawn_move(to, from, info) {
                    moves.push(Move::new(from, to, MoveType::Normal, None));
                }
            }

            while let Some(to) = pop_lsb(&mut captures_left) {
                let from = to - 7;

                if is_legal_pawn_move(to, from, info) {
                    add_pawn_move(moves, from, to, MoveType::Capture, color);
                }
            }

            while let Some(to) = pop_lsb(&mut captures_right) {
                let from = to - 9;

                if is_legal_pawn_move(to, from, info) {
                    add_pawn_move(moves, from, to, MoveType::Capture, color);
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

                if is_legal_pawn_move(to, from, info) {
                    add_pawn_move(moves, from, to, MoveType::Normal, color);
                }
            }

            while let Some(to) = pop_lsb(&mut double_pushes) {
                let from = to + 16;
                if is_legal_pawn_move(to, from, info) {
                    moves.push(Move::new(from, to, MoveType::Normal, None));
                }
            }

            while let Some(to) = pop_lsb(&mut captures_left) {
                let from = to + 9;

                if is_legal_pawn_move(to, from, info) {
                    add_pawn_move(moves, from, to, MoveType::Capture, color);
                }
            }

            while let Some(to) = pop_lsb(&mut captures_right) {
                let from = to + 7;

                if is_legal_pawn_move(to, from, info) {
                    add_pawn_move(moves, from, to, MoveType::Capture, color);
                }
            }
        }
    }
}

pub fn pseudo_en_passant_moves(board: &Board, color: Color, moves: &mut MoveList) {
    let pawns = board.pieces(color, PieceType::Pawn);

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
                    moves.push(mv);
                }

                if en_right != 0 {
                    // there is a pawn to the right
                    let mv = Move::new(en_pass_to - 9, en_pass_to, MoveType::EnPassant, None);
                    moves.push(mv);
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
                    moves.push(mv);
                }

                if en_right != 0 {
                    // there is a pawn to the right
                    let mv = Move::new(en_pass_to + 7, en_pass_to, MoveType::EnPassant, None);
                    moves.push(mv);
                }
            }
        }
    }
}

pub fn legal_en_passant_moves(
    board: &Board,
    color: Color,
    info: &MoveGenInfo,
    moves: &mut MoveList,
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

                    if legal_en_passant(board, mv, info, king_sq) {
                        moves.push(mv);
                    }
                }

                if en_right != 0 {
                    // there is a pawn to the right
                    let mv = Move::new(en_pass_to - 9, en_pass_to, MoveType::EnPassant, None);

                    if legal_en_passant(board, mv, info, king_sq) {
                        moves.push(mv);
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

                    if legal_en_passant(board, mv, info, king_sq) {
                        moves.push(mv);
                    }
                }

                if en_right != 0 {
                    // there is a pawn to the right
                    let mv = Move::new(en_pass_to + 7, en_pass_to, MoveType::EnPassant, None);

                    if legal_en_passant(board, mv, info, king_sq) {
                        moves.push(mv);
                    }
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
                        moves.push(Move::new(from, to, MoveType::Normal, Some(promotion)));
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
                        moves.push(Move::new(from, to, MoveType::Normal, Some(promotion)));
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
        }
    }
}

pub fn legal_pawn_capture_moves(
    board: &Board,
    color: Color,
    info: &MoveGenInfo,
    moves: &mut MoveList,
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

                if rank_of(to) == promotion_rank {
                    continue; // promotion moves are handled separately
                }

                if is_legal_pawn_move(to, from, info) {
                    add_pawn_move(moves, from, to, MoveType::Capture, color);
                }
            }

            while let Some(to) = pop_lsb(&mut captures_right) {
                let from = to - 9;

                if rank_of(to) == promotion_rank {
                    continue; // promotion moves are handled separately
                }

                if is_legal_pawn_move(to, from, info) {
                    add_pawn_move(moves, from, to, MoveType::Capture, color);
                }
            }
        }

        Color::Black => {
            let promotion_rank = 0;

            let mut captures_left = ((pawns & !FILE_A) >> 9) & enemies;

            let mut captures_right = ((pawns & !FILE_H) >> 7) & enemies;

            while let Some(to) = pop_lsb(&mut captures_left) {
                let from = to + 9;

                if rank_of(to) == promotion_rank {
                    continue; // promotion moves are handled separately
                }

                if is_legal_pawn_move(to, from, info) {
                    add_pawn_move(moves, from, to, MoveType::Capture, color);
                }
            }

            while let Some(to) = pop_lsb(&mut captures_right) {
                let from = to + 7;

                if rank_of(to) == promotion_rank {
                    continue; // promotion moves are handled separately
                }

                if is_legal_pawn_move(to, from, info) {
                    add_pawn_move(moves, from, to, MoveType::Capture, color);
                }
            }
        }
    }
}

pub fn legal_pawn_quiet_moves(
    board: &Board,
    color: Color,
    info: &MoveGenInfo,
    moves: &mut MoveList,
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

                if rank_of(to) == promotion_rank {
                    continue; // promotion moves are handled separately
                }

                if is_legal_pawn_move(to, from, info) {
                    add_pawn_move(moves, from, to, MoveType::Normal, color);
                }
            }

            while let Some(to) = pop_lsb(&mut double_pushes) {
                let from = to - 16;

                if is_legal_pawn_move(to, from, info) {
                    moves.push(Move::new(from, to, MoveType::Normal, None));
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

                if is_legal_pawn_move(to, from, info) {
                    add_pawn_move(moves, from, to, MoveType::Normal, color);
                }
            }

            while let Some(to) = pop_lsb(&mut double_pushes) {
                let from = to + 16;
                if is_legal_pawn_move(to, from, info) {
                    moves.push(Move::new(from, to, MoveType::Normal, None));
                }
            }
        }
    }
}

pub fn legal_pawn_promotion_moves(
    board: &Board,
    color: Color,
    info: &MoveGenInfo,
    moves: &mut MoveList,
) {
    let pawns = board.pieces(color, PieceType::Pawn);
    let occupancy = board.all_occupancy();
    let empty = !occupancy;
    let enemies = board.occupancy_of(color.opposite());

    match color {
        Color::White => {
            let promotion_rank = 7;

            let mut single_pushes = (pawns << 8) & empty;

            let mut captures_left = ((pawns & !FILE_A) << 7) & enemies;

            let mut captures_right = ((pawns & !FILE_H) << 9) & enemies;

            while let Some(to) = pop_lsb(&mut single_pushes) {
                let from = to - 8;

                if rank_of(to) != promotion_rank {
                    continue; // promotion moves are handled separately
                }

                if is_legal_pawn_move(to, from, info) {
                    add_pawn_move(moves, from, to, MoveType::Normal, color);
                }
            }

            while let Some(to) = pop_lsb(&mut captures_left) {
                let from = to - 7;

                if rank_of(to) != promotion_rank {
                    continue; // promotion moves are handled separately
                }

                if is_legal_pawn_move(to, from, info) {
                    add_pawn_move(moves, from, to, MoveType::Capture, color);
                }
            }

            while let Some(to) = pop_lsb(&mut captures_right) {
                let from = to - 9;

                if rank_of(to) != promotion_rank {
                    continue; // promotion moves are handled separately
                }

                if is_legal_pawn_move(to, from, info) {
                    add_pawn_move(moves, from, to, MoveType::Capture, color);
                }
            }
        }

        Color::Black => {
            let promotion_rank = 0;

            let mut single_pushes = (pawns >> 8) & empty;

            let mut captures_left = ((pawns & !FILE_A) >> 9) & enemies;

            let mut captures_right = ((pawns & !FILE_H) >> 7) & enemies;

            while let Some(to) = pop_lsb(&mut single_pushes) {
                let from = to + 8;

                if rank_of(to) != promotion_rank {
                    continue; // promotion moves are handled separately
                }

                if is_legal_pawn_move(to, from, info) {
                    add_pawn_move(moves, from, to, MoveType::Normal, color);
                }
            }

            while let Some(to) = pop_lsb(&mut captures_left) {
                let from = to + 9;

                if rank_of(to) != promotion_rank {
                    continue; // promotion moves are handled separately
                }

                if is_legal_pawn_move(to, from, info) {
                    add_pawn_move(moves, from, to, MoveType::Capture, color);
                }
            }

            while let Some(to) = pop_lsb(&mut captures_right) {
                let from = to + 7;

                if rank_of(to) != promotion_rank {
                    continue; // promotion moves are handled separately
                }

                if is_legal_pawn_move(to, from, info) {
                    add_pawn_move(moves, from, to, MoveType::Capture, color);
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
            moves.push(Move::new(from, to, kind, Some(promotion)));
        }
    } else {
        moves.push(Move::new(from, to, kind, None));
    }
}

pub fn is_legal_pawn_move(to: Square, from: Square, info: &MoveGenInfo) -> bool {
    let to_bb = bit(to);

    to_bb & info.pin_masks[from as usize] & info.check_mask != 0
}

pub fn legal_en_passant(board: &Board, mv: Move, info: &MoveGenInfo, king_sq: Square) -> bool {
    debug_assert_eq!(mv.kind(), MoveType::EnPassant);

    let from = mv.from();
    let to = mv.to();
    let captured_sq = square(file_of(to), rank_of(from));
    let side_to_move = board.side_to_move();

    // FOR DEBUG
    let piece = board.piece_at(from).expect("NO PIECE IN LEGAL_EN_PASSANT!");

    debug_assert_eq!(Some(PieceType::Pawn), board.piecetype_at(captured_sq));
    debug_assert_eq!(side_to_move, piece.color);
    debug_assert_eq!(PieceType::Pawn, piece.kind);

    // first check if already in check

    if info.checkers != 0 {
        if info.checkers & &bit(captured_sq) == 0 {
            // if the checking piece is the captued square then this can be valid. However still prove that it doesn't expose any checks later
            return false;
        }
    }

    let mut occ = board.all_occupancy();

    occ &= !bit(from);
    occ &= !bit(captured_sq);
    occ |= bit(to);

    let diagonals = bishop_attacks(king_sq, occ);
    let straights = rook_attacks(king_sq, occ);

    let bishops = board.pieces(side_to_move.opposite(), PieceType::Bishop);
    let queens = board.pieces(side_to_move.opposite(), PieceType::Queen);
    let rooks = board.pieces(side_to_move.opposite(), PieceType::Rook);

    if diagonals & (bishops | queens) != 0 {
        return false;
    }

    if straights & (rooks | queens) != 0 {
        return false;
    }

    true
}
