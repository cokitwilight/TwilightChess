use crate::bitboard::{
    A1, A8, B1, B8, Bitboard, C1, C8, E1, E8, F1, F8, FILE_A, FILE_H, G1, G8, H1, H8, RANK_3,
    RANK_6, Square, bishop_attacks, bit, file_of, king_attacks, knight_attacks, pawn_attacks,
    pop_lsb, queen_attacks, rank_of, rook_attacks, square,
};
use crate::board::{
    BLACK_KINGSIDE, BLACK_QUEENSIDE, Board, Move, MoveType, WHITE_KINGSIDE, WHITE_QUEENSIDE,
};
use crate::engine::staged::MovePicker;
use crate::moves::MoveGenInfo;
use crate::moves::king::legal_king_move;
use crate::moves::pawn::{is_legal_pawn_move, legal_en_passant};
use crate::types::{Color, Piece, PieceType};

impl MovePicker {
    pub fn is_special(&self, mv: Move) -> bool {
        let m = Some(mv);

        m == self.pv_move || m == self.tt_move || m == self.killer_1 || m == self.killer_2
    }
}

pub fn verify_move(board: &Board, mv: Move, info: &MoveGenInfo) -> bool {
    // match
    let Some(moving_piece) = board.piece_at(mv.from()) else {
        return false;
    };

    let color = moving_piece.color;

    if color != board.side_to_move() {
        return false;
    }

    let piece_type = moving_piece.kind;

    let pieces = board.pieces(color, piece_type);

    if pieces & bit(mv.from()) == 0 {
        return false;
    }

    let occ = board.all_occupancy();

    let piece_attacks = match piece_type {
        PieceType::Pawn => {
            if mv.kind() == MoveType::EnPassant {
                !0u64
            } else {
                pawn_moves_at(board, mv.from(), info)
            }
        }
        PieceType::Knight => knight_attacks(mv.from()),
        PieceType::Bishop => bishop_attacks(mv.from(), occ),
        PieceType::Rook => rook_attacks(mv.from(), occ),
        PieceType::Queen => queen_attacks(mv.from(), occ),
        PieceType::King => {
            if mv.kind() == MoveType::Castle {
                !0u64
            } else {
                king_attacks(mv.from())
            }
        }
    };

    let friends = board.occupancy_of(color);
    let enemies = board.occupancy_of(color.opposite());

    let valid_moves = piece_attacks & !friends;
    let valid_captures = valid_moves & enemies;

    if valid_moves & bit(mv.to()) == 0 {
        return false;
    }

    let king_sq = info.king_sq;

    match mv.kind() {
        MoveType::Normal => {
            if valid_captures & bit(mv.to()) != 0 {
                return false;
            }
        }
        MoveType::Capture => {
            if valid_captures & bit(mv.to()) == 0 {
                return false;
            }
        }
        MoveType::Castle => {
            if piece_type != PieceType::King {
                return false;
            }

            let rook_sq = match (color, mv.from(), mv.to()) {
                (Color::White, E1, G1) => H1,
                (Color::White, E1, C1) => A1,
                (Color::Black, E8, G8) => H8,
                (Color::Black, E8, C8) => A8,
                _ => return false,
            };

            if board.piece_at(rook_sq)
                != Some(Piece {
                    kind: PieceType::Rook,
                    color,
                })
            {
                return false;
            }
        }
        MoveType::EnPassant => {
            if piece_type != PieceType::Pawn
                || board.en_passant() != Some(mv.to())
                || board.piece_at(mv.to()).is_some()
            {
                return false;
            }
            let pawn_attacks = pawn_attacks(color, bit(mv.from()));

            if pawn_attacks & bit(mv.to()) == 0 {
                return false;
            }

            let captured_sq = square(file_of(mv.to()), rank_of(mv.from()));

            if board.piece_at(captured_sq)
                != Some(Piece {
                    kind: PieceType::Pawn,
                    color: color.opposite(),
                })
            {
                return false;
            }
        }
    }
    if let Some(promo) = mv.promotion() {
        if promo == PieceType::Pawn || promo == PieceType::King {
            return false;
        }
        if piece_type != PieceType::Pawn {
            return false;
        }
        match color {
            Color::White => {
                if rank_of(mv.to()) != 7 {
                    return false;
                }
            }
            Color::Black => {
                if rank_of(mv.to()) != 0 {
                    return false;
                }
            }
        }
    } else {
        let promo_rank = match color {
            Color::White => 7,
            Color::Black => 0,
        };
        if piece_type == PieceType::Pawn && rank_of(mv.to()) == promo_rank {
            return false;
        }
    }

    match mv.kind() {
        MoveType::Castle => {
            // special case. Has to check valid moving squares
            if info.checkers != 0 {
                return false;
            }

            let destination_file = file_of(mv.to());

            if destination_file == 6 {
                // File G == King side castle
                let right = match color {
                    Color::White => WHITE_KINGSIDE,
                    Color::Black => BLACK_KINGSIDE,
                };

                if !board.has_castling_right(right) {
                    return false;
                }

                let mut starting_square = match color {
                    Color::White => F1,
                    Color::Black => F8,
                };
                if board.square_attacked_by(color.opposite(), starting_square)
                    || occ & bit(starting_square) != 0
                {
                    return false;
                }

                starting_square += 1;

                if board.square_attacked_by(color.opposite(), starting_square)
                    || occ & bit(starting_square) != 0
                {
                    return false;
                }
            } else if destination_file == 2 {
                // FILE C == Queen Side Castle
                let right = match color {
                    Color::White => WHITE_QUEENSIDE,
                    Color::Black => BLACK_QUEENSIDE,
                };

                if !board.has_castling_right(right) {
                    return false;
                }

                let mut starting_square = match color {
                    Color::White => C1,
                    Color::Black => C8,
                };
                if board.square_attacked_by(color.opposite(), starting_square)
                    || occ & bit(starting_square) != 0
                {
                    return false;
                }

                starting_square += 1;

                if board.square_attacked_by(color.opposite(), starting_square)
                    || occ & bit(starting_square) != 0
                {
                    return false;
                }

                let end_square = match color {
                    Color::White => B1,
                    Color::Black => B8,
                };

                if occ & bit(end_square) != 0 {
                    return false;
                }
            } else {
                return false; // maybe panic as this is a very abnormal case
            }
        }
        MoveType::EnPassant => {
            if !legal_en_passant(board, mv, info, king_sq) {
                return false;
            }
        }
        _ => {
            if piece_type == PieceType::King {
                if !legal_king_move(board, mv) {
                    return false;
                }
            } else {
                if bit(mv.to()) & info.check_mask == 0 {
                    return false;
                }

                if bit(mv.to()) & info.pin_masks[mv.from() as usize] == 0 {
                    return false;
                }
            }
        }
    }

    true
}

fn pawn_moves_at(board: &Board, sq: Square, info: &MoveGenInfo) -> Bitboard {
    let mut moves = 0u64;

    let color = board.side_to_move;

    let pawns = board.pieces(color, PieceType::Pawn) & bit(sq);
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
                    moves |= bit(to);
                }
            }

            while let Some(to) = pop_lsb(&mut double_pushes) {
                let from = to - 16;

                if is_legal_pawn_move(to, from, info) {
                    moves |= bit(to);
                }
            }

            while let Some(to) = pop_lsb(&mut captures_left) {
                let from = to - 7;

                if is_legal_pawn_move(to, from, info) {
                    moves |= bit(to);
                }
            }

            while let Some(to) = pop_lsb(&mut captures_right) {
                let from = to - 9;

                if is_legal_pawn_move(to, from, info) {
                    moves |= bit(to);
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
                    moves |= bit(to);
                }
            }

            while let Some(to) = pop_lsb(&mut double_pushes) {
                let from = to + 16;
                if is_legal_pawn_move(to, from, info) {
                    moves |= bit(to);
                }
            }

            while let Some(to) = pop_lsb(&mut captures_left) {
                let from = to + 9;

                if is_legal_pawn_move(to, from, info) {
                    moves |= bit(to);
                }
            }

            while let Some(to) = pop_lsb(&mut captures_right) {
                let from = to + 7;

                if is_legal_pawn_move(to, from, info) {
                    moves |= bit(to);
                }
            }
        }
    }
    moves
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn castling_requires_exact_squares_and_the_matching_rook() {
        let board = Board::from_fen("r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1")
            .expect("valid castling position");
        let info = MoveGenInfo::calculate(&board, Color::White);

        assert!(verify_move(
            &board,
            Move::new(E1, G1, MoveType::Castle, None),
            &info
        ));
        assert!(verify_move(
            &board,
            Move::new(E1, C1, MoveType::Castle, None),
            &info
        ));
        assert!(!verify_move(
            &board,
            Move::new(E1, G8, MoveType::Castle, None),
            &info
        ));

        let black_board = Board::from_fen("r3k2r/8/8/8/8/8/8/R3K2R b KQkq - 0 1")
            .expect("valid black castling position");
        let black_info = MoveGenInfo::calculate(&black_board, Color::Black);

        assert!(verify_move(
            &black_board,
            Move::new(E8, G8, MoveType::Castle, None),
            &black_info
        ));
        assert!(verify_move(
            &black_board,
            Move::new(E8, C8, MoveType::Castle, None),
            &black_info
        ));

        let missing_rook = Board::from_fen("4k3/8/8/8/8/8/8/4K3 w K - 0 1")
            .expect("valid position with stale castling rights");
        let missing_rook_info = MoveGenInfo::calculate(&missing_rook, Color::White);

        assert!(!verify_move(
            &missing_rook,
            Move::new(E1, G1, MoveType::Castle, None),
            &missing_rook_info
        ));
    }
}
