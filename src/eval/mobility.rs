use crate::bitboard::{Bitboard, RANK_1, RANK_3, RANK_6, RANK_8, RANK_MASKS};

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
    score += move_aggression(board, color, info);
    score += space_bonus(board, color, info);

    score
}

fn development_penalty(board: &Board, color: Color, info: &EvalInfo) -> i32 {
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

fn available_moves(board: &Board, color: Color, info: &EvalInfo) -> i32 {
    let mut score = 0;

    let enemy_color = color.opposite();

    let friends = board.occupancy_of(color);

    let pawn_moves = pawn_moves_bitboard(board, color).count_ones() as i32;

    // encourage pawn mobility in endgame
    let bonus = if info.phase() < 12 { 4 } else { 1 };

    score += pawn_moves * bonus;

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
fn move_pressure(_board: &Board, color: Color, info: &EvalInfo) -> i32 {
    let multiple_attacked = info.attacked_by_two(color.opposite()).count_ones() as i32; // already includes occupancy check

    if multiple_attacked > 3 {
        return multiple_attacked * -15;
    } else if multiple_attacked >= 1 {
        return -10;
    } else {
        return 0;
    }
}

fn move_aggression(board: &Board, color: Color, info: &EvalInfo) -> i32 {
    let enemy_half = match color {
        Color::White => RANK_MASKS[4] | RANK_MASKS[5] | RANK_MASKS[6] | RANK_MASKS[7],
        Color::Black => RANK_MASKS[0] | RANK_MASKS[1] | RANK_MASKS[2] | RANK_MASKS[3],
    };

    let num_attacks = (enemy_half & info.all_attacks(color)).count_ones() as i32;

    let num_pieces = (enemy_half & board.occupancy_of(color)).count_ones() as i32;

    return num_attacks + (num_pieces * 4);
}

fn hanging_pieces(board: &Board, color: Color, info: &EvalInfo) -> i32 {
    let occupancy = board.occupancy_of(color) & !board.pieces(color, PieceType::King);

    let hanging = occupancy & info.all_attacks(color.opposite()) & !info.all_attacks(color);

    return -10 * hanging.count_ones() as i32;
}

fn space_bonus(board: &Board, color: Color, info: &EvalInfo) -> i32 {
    if info.phase() < 12 {
        return 0;
    }

    let pawns = board.pieces(color, PieceType::Pawn);

    let space_mask = pawn_space_bitboard(pawns, color);

    let available_space =
        space_mask & !info.all_attacks(color.opposite()) & !board.occupancy_of(color.opposite());

    return available_space.count_ones() as i32 * 3;
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

fn pawn_space_bitboard(pawns: Bitboard, color: Color) -> Bitboard {
    let mut pawn_space: u64;

    match color {
        Color::White => {
            pawn_space = pawns >> 8;
            pawn_space |= pawn_space >> 8;
            pawn_space |= pawn_space >> 16;
            pawn_space |= pawn_space >> 32;
            pawn_space &= !RANK_1;
        }
        Color::Black => {
            pawn_space = pawns << 8;
            pawn_space |= pawn_space << 8;
            pawn_space |= pawn_space << 16;
            pawn_space |= pawn_space << 32;
            pawn_space &= !RANK_8;
        }
    }

    pawn_space
}
