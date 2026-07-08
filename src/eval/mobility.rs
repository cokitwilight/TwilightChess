use crate::bitboard::{
    Bitboard, FILE_A, FILE_H, RANK_1, RANK_3, RANK_6, RANK_8, knight_attacks, pop_lsb,
};

use crate::board::Board;
use crate::eval::eval::EvalInfo;
use crate::types::{Color, PieceType};

pub fn mobility_score(board: &Board, info: &EvalInfo) -> i32 {
    mobility_score_raw(board, Color::White, info) - mobility_score_raw(board, Color::Black, info)
}

pub fn mobility_score_raw(board: &Board, color: Color, info: &EvalInfo) -> i32 {
    // check every move and subtract how many valid moves there are
    let mut score = 0;

    score += development_penalty(board, color, info);
    score += available_moves(board, color, info);
    score += move_pressure(board, color, info);
    score += hanging_pieces(board, color, info);

    score
}

pub fn development_penalty(board: &Board, color: Color, info: &EvalInfo) -> i32 {
    let starting_rank = match color {
        Color::White => RANK_1,
        Color::Black => RANK_8,
    };

    let non_developed_pieces = ((board.occupancy_of(color) & starting_rank)
        & !(board.pieces(color, PieceType::King) | board.pieces(color, PieceType::Rook)))
    .count_ones() as i32;

    let bonus = if info.phase() < 16 { -6 } else { -4 };

    return non_developed_pieces * bonus;
}

pub fn available_moves(board: &Board, color: Color, info: &EvalInfo) -> i32 {
    let mut score = 0;

    let enemy_color = color.opposite();

    let friends = board.occupancy_of(color);

    score += pawn_moves_bitboard(board, color).count_ones() as i32; // only includes single/double pawn pushes

    let pawn_attacks = info.attacks(enemy_color, PieceType::Pawn);
    let knight_and_bishop_attacks =
        info.attacks(enemy_color, PieceType::Knight) | info.attacks(enemy_color, PieceType::Bishop);

    let rook_attacks = info.attacks(enemy_color, PieceType::Rook);

    let available_knight_moves = info.attacks(color, PieceType::Knight) & !pawn_attacks & !friends;

    let available_bishop_moves = info.attacks(color, PieceType::Bishop) & !pawn_attacks & !friends;

    let available_rook_moves = info.attacks(color, PieceType::Rook)
        & !(pawn_attacks | knight_and_bishop_attacks | friends);

    let available_queen_moves = info.attacks(color, PieceType::Queen)
        & !(pawn_attacks | knight_and_bishop_attacks | rook_attacks | friends);

    score += available_knight_moves.count_ones() as i32 * 4;
    score += available_bishop_moves.count_ones() as i32 * 3;
    score += available_rook_moves.count_ones() as i32 * 3;
    score += available_queen_moves.count_ones() as i32 * 2;

    score
}

// this might not be best file for this function
pub fn move_pressure(board: &Board, color: Color, info: &EvalInfo) -> i32 {
    let multiple_attacked = info.attacked_by_two(color.opposite()).count_ones() as i32; // already includes occupancy check

    if multiple_attacked > 3 {
        return multiple_attacked * -15;
    } else if multiple_attacked >= 1 {
        return -10;
    } else {
        return 0;
    }
}

pub fn hanging_pieces(board: &Board, color: Color, info: &EvalInfo) -> i32 {
    let occupancy = board.occupancy_of(color) & !board.pieces(color, PieceType::King);

    let hanging = occupancy & info.all_attacks(color.opposite()) & !info.all_attacks(color);

    return -10 * hanging.count_ones() as i32;
}

// since not all pawn moves are captures. Ignore en passant for now as it might be too expensive/complicated
fn pawn_moves_bitboard(board: &Board, color: Color) -> Bitboard {
    let pawns = board.pieces(color, PieceType::Pawn);

    let empty = !board.all_occupancy();

    let mut moves = 0u64;

    match color {
        Color::White => {
            let single_pushes = (pawns << 8) & empty;
            moves |= single_pushes;

            moves |= ((single_pushes & RANK_3) << 8) & empty;
        }
        Color::Black => {
            let single_pushes = (pawns >> 8) & empty;
            moves |= single_pushes;

            moves |= ((single_pushes & RANK_6) >> 8) & empty;
        }
    }

    moves
}
