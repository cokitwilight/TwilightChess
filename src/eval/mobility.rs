use crate::bitboard::{
    Bitboard, FILE_A, FILE_H, RANK_1, RANK_2, RANK_3, RANK_6, RANK_7, RANK_8, RANK_MASKS,
};

use crate::board::Board;
use crate::eval::eval::EvalInfo;
use crate::eval::scale_by_phase;
use crate::types::{Color, PieceType};

const KNIGHT_MOVE: [i32; 8] = [-100, -40, -10, 0, 10, 40, 60, 85]; // PREV MAX: 64
const BISHOP_MOVE: [i32; 13] = [-100, -90, -70, -40, -10, 0, 10, 20, 30, 45, 50, 55, 60]; // PREV MAX: 78
const ROOK_MOVE: [i32; 14] = [-40, -35, -20, -10, 0, 0, 0, 10, 20, 25, 30, 35, 40, 45]; // PREV MAX: 84
const QUEEN_MOVE: [i32; 27] = [
    -400, -150, -100, -50, -25, -15, 0, 0, 0, 0, 0, 0, 0, 20, 25, 30, 35, 40, 45, 50, 55, 60, 65,
    70, 75, 80, 80,
]; // PREV MAX: 54

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

    let mut non_developed_pieces = ((board.occupancy_of(color) & starting_rank)
        & !(board.pieces(color, PieceType::King) | board.pieces(color, PieceType::Rook)))
    .count_ones() as i32;

    let starting_rooks = (board.pieces(color, PieceType::Rook) & starting_rank & (FILE_A | FILE_H))
        .count_ones() as i32;

    // since it might be likely that the rook will have moved by then(and rooks are more common to be on the starting rank)
    if starting_rooks != 0 && info.phase() > 16 {
        non_developed_pieces += starting_rooks;
    }

    let bonus = if info.phase() < 16 { -8 } else { -5 };

    non_developed_pieces * bonus
}

fn available_moves(board: &Board, color: Color, info: &EvalInfo) -> i32 {
    let mut score = 0;

    // gives a beginning opening bonus so as not to overvalue development over other values.
    // For example in the beginning the queen has 0 moves so it slaps a -300 bonus on the eval.
    // tapered bonus will give an added index to the lookup table while in the opening
    let tapered_bonus = scale_by_phase(8, info.phase(), 18, 22) as u32;

    let enemy_color = color.opposite();

    let friends = board.occupancy_of(color);

    let pawn_moves = pawn_moves_bitboard(board, color).count_ones() as i32;

    // encourage pawn mobility in endgame
    let bonus = if info.phase() < 12 { 6 } else { 2 };

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

    let num_knights = board.pieces(color, PieceType::Knight).count_ones();
    let num_bishops = board.pieces(color, PieceType::Bishop).count_ones();
    let num_rooks = board.pieces(color, PieceType::Rook).count_ones();
    let num_queens = board.pieces(color, PieceType::Queen).count_ones();

    if num_knights != 0 {
        score += num_knights as i32
            * KNIGHT_MOVE[(available_knight_moves.count_ones() / num_knights + tapered_bonus / 4)
                .clamp(0, 7) as usize];
    }
    if num_bishops != 0 {
        score += num_bishops as i32
            * BISHOP_MOVE[(available_bishop_moves.count_ones() / num_bishops + tapered_bonus / 2)
                .clamp(0, 12) as usize];
    }
    if num_rooks != 0 {
        score += num_rooks as i32
            * ROOK_MOVE[(available_rook_moves.count_ones() / num_rooks + tapered_bonus).clamp(0, 13)
                as usize];
    }
    if num_queens != 0 {
        score += num_queens as i32
            * QUEEN_MOVE[(available_queen_moves.count_ones() / num_queens + tapered_bonus * 2)
                .clamp(0, 26) as usize];
    }

    score
}

// this might not be best file for this function
fn move_pressure(board: &Board, color: Color, info: &EvalInfo) -> i32 {
    let multiple_attacked =
        (info.attacked_by_two(color.opposite()) & board.occupancy_of(color)).count_ones() as i32;

    if multiple_attacked > 3 {
        multiple_attacked * -15
    } else if multiple_attacked >= 1 {
        -10
    } else {
        0
    }
}

fn move_aggression(board: &Board, color: Color, info: &EvalInfo) -> i32 {
    let enemy_half = match color {
        Color::White => RANK_MASKS[4] | RANK_MASKS[5] | RANK_MASKS[6] | RANK_MASKS[7],
        Color::Black => RANK_MASKS[0] | RANK_MASKS[1] | RANK_MASKS[2] | RANK_MASKS[3],
    };

    let num_attacks = (enemy_half & info.all_attacks(color)).count_ones() as i32;

    let num_pieces = (enemy_half & board.occupancy_of(color)).count_ones() as i32;

    num_attacks + (num_pieces * 4)
}

fn hanging_pieces(board: &Board, color: Color, info: &EvalInfo) -> i32 {
    let occupancy = board.occupancy_of(color) & !board.pieces(color, PieceType::King);

    let hanging = occupancy & info.all_attacks(color.opposite()) & !info.all_attacks(color);

    let hanging_major_pieces = hanging & !board.pieces(color, PieceType::Pawn);

    let weak = occupancy & !info.all_attacks(color) & !board.pieces(color, PieceType::Pawn); // not defended but not attacked. Could become a weakness

    -10 * hanging_major_pieces.count_ones() as i32
        + -5 * hanging.count_ones() as i32
        + -3 * weak.count_ones() as i32
}

fn space_bonus(board: &Board, color: Color, info: &EvalInfo) -> i32 {
    let pawns = board.pieces(color, PieceType::Pawn);

    if pawns == 0 {
        return 0;
    }

    let space_mask = pawn_space_bitboard(pawns, color);

    let available_space =
        space_mask & !info.all_attacks(color.opposite()) & !board.occupancy_of(color.opposite());

    scale_by_phase(available_space.count_ones() as i32 * 3, info.phase(), 8, 16)
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
            pawn_space &= !RANK_2;
            pawn_space &= !RANK_1;
        }
        Color::Black => {
            pawn_space = pawns << 8;
            pawn_space |= pawn_space << 8;
            pawn_space |= pawn_space << 16;
            pawn_space &= !RANK_7;
            pawn_space &= !RANK_8;
        }
    }

    pawn_space
}
