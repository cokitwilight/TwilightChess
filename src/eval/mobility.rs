use crate::bitboard::{
    Bitboard, FILE_A, FILE_H, RANK_1, RANK_2, RANK_3, RANK_6, RANK_7, RANK_8, RANK_MASKS,
    bishop_attacks, knight_attacks, pop_lsb, queen_attacks, rook_attacks,
};

use crate::board::Board;
use crate::eval::eval::EvalInfo;
use crate::eval::scale_by_phase;
use crate::types::{Color, PieceType};

const KNIGHT_MOVE: [i32; 9] = [-30, -10, -5, 0, 5, 20, 25, 35, 35]; // PREV MAX: 64
const BISHOP_MOVE: [i32; 14] = [-30, -30, -20, -5, -5, 0, 10, 20, 30, 30, 30, 35, 35, 35]; // PREV MAX: 78
const ROOK_MOVE: [i32; 15] = [-25, -15, -10, -5, 0, 0, 0, 10, 20, 25, 30, 30, 30, 30, 30]; // PREV MAX: 84
const QUEEN_MOVE: [i32; 28] = [
    -130, -80, -40, -30, -25, -15, -5, 0, 0, 0, 0, 0, 0, 10, 15, 20, 25, 30, 35, 40, 45, 50, 50,
    50, 50, 50, 50, 50,
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

// range: -35 <-> 0. Note with phase scaling the more realistic range is -24 <-> 0
// since this function does keep track of pieces going backwards to starting squares don't add a massive penalty
pub(super) fn development_penalty(board: &Board, color: Color, info: &EvalInfo) -> i32 {
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

    let bonus = if info.phase() < 16 { -5 } else { -3 };

    non_developed_pieces * bonus
}

// per pawn: range: 0 <-> ~80. // number of pawns moves varies based on starting rank
// per knight: range: -30 <-> 35
// per bishop: range: -30 <-> 35
// per rook: range: -25 <-> 30
// per queen: range: -130 <-> 50
pub(super) fn available_moves(board: &Board, color: Color, info: &EvalInfo) -> i32 {
    let mut score = 0;

    // gives a beginning opening bonus so as not to overvalue development over other values.
    // For example in the beginning the queen has 0 moves so it slaps a -300 bonus on the eval.
    // tapered bonus will give an added index to the lookup table while in the opening
    let tapered_bonus = scale_by_phase(8, info.phase(), 16, 24) as u32;

    let enemy_color = color.opposite();

    let occupancy = board.all_occupancy();

    let friends = board.occupancy_of(color);

    let pawn_moves = pawn_moves_bitboard(board, color).count_ones() as i32;

    // encourage pawn mobility in endgame
    let bonus = if info.phase() < 12 { 10 } else { 4 };

    score += pawn_moves * bonus;

    let pawn_attacks = info.attacks(enemy_color, PieceType::Pawn);
    let knight_and_bishop_attacks =
        info.attacks(enemy_color, PieceType::Knight) | info.attacks(enemy_color, PieceType::Bishop);

    let enemy_rook_attacks = info.attacks(enemy_color, PieceType::Rook);

    let mut knights = board.pieces(color, PieceType::Knight);

    while let Some(sq) = pop_lsb(&mut knights) {
        let knight_moves = knight_attacks(sq) & !friends & !pawn_attacks;
        score += KNIGHT_MOVE[(knight_moves.count_ones() + tapered_bonus / 4).min(8) as usize]
    }

    let mut bishops = board.pieces(color, PieceType::Bishop);

    while let Some(sq) = pop_lsb(&mut bishops) {
        let bishop_moves = bishop_attacks(sq, occupancy) & !friends & !pawn_attacks;
        score += BISHOP_MOVE[(bishop_moves.count_ones() + tapered_bonus / 2).min(13) as usize]
    }

    let mut rooks = board.pieces(color, PieceType::Rook);

    while let Some(sq) = pop_lsb(&mut rooks) {
        let rook_moves =
            rook_attacks(sq, occupancy) & !friends & !pawn_attacks & !knight_and_bishop_attacks;
        score += ROOK_MOVE[(rook_moves.count_ones() + tapered_bonus).min(14) as usize];
    }

    let mut queens = board.pieces(color, PieceType::Queen);

    while let Some(sq) = pop_lsb(&mut queens) {
        let queen_moves = queen_attacks(sq, occupancy)
            & !friends
            & !pawn_attacks
            & !knight_and_bishop_attacks
            & !enemy_rook_attacks;
        score += QUEEN_MOVE[(queen_moves.count_ones() + tapered_bonus * 2).min(27) as usize];
    }

    score
}

// range: -15 <-> 0
pub(super) fn move_pressure(board: &Board, color: Color, info: &EvalInfo) -> i32 {
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
// range: 0 <-> ~40
pub(super) fn move_aggression(board: &Board, color: Color, info: &EvalInfo) -> i32 {
    let enemy_half = match color {
        Color::White => RANK_MASKS[4] | RANK_MASKS[5] | RANK_MASKS[6] | RANK_MASKS[7],
        Color::Black => RANK_MASKS[0] | RANK_MASKS[1] | RANK_MASKS[2] | RANK_MASKS[3],
    };

    let num_attacks = (enemy_half & info.all_attacks(color)).count_ones() as i32;

    let num_pieces = (enemy_half & board.occupancy_of(color)).count_ones() as i32;

    num_attacks + (num_pieces * 4)
}

// range: ~ -100 <-> 0
pub(super) fn hanging_pieces(board: &Board, color: Color, info: &EvalInfo) -> i32 {
    let occupancy = board.occupancy_of(color) & !board.pieces(color, PieceType::King);

    let hanging = occupancy & info.all_attacks(color.opposite()) & !info.all_attacks(color);

    let hanging_pawns = hanging & board.pieces(color, PieceType::Pawn);

    let hanging_major_pieces = hanging & !board.pieces(color, PieceType::Pawn);

    let weak = occupancy & !info.all_attacks(color) & !board.pieces(color, PieceType::Pawn); // not defended but not attacked. Could become a weakness

    -30 * hanging_major_pieces.count_ones() as i32
        + -15 * hanging_pawns.count_ones() as i32
        + -7 * weak.count_ones() as i32
}

// range: 0 <-> ~27
pub(super) fn space_bonus(board: &Board, color: Color, info: &EvalInfo) -> i32 {
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
