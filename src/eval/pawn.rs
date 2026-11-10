use crate::bitboard::{
    Bitboard, FILE_MASKS, RANK_2, RANK_4, RANK_5, RANK_7, RANK_MASKS, bit, file_of, pop_lsb,
    rank_of,
};
use crate::board::Board;
pub use crate::eval::lookup::PAWN_WEAKNESS_TABLE;
use crate::eval::lookup::{BACKWARDS_PAWNS, PASSED_PAWNS};
use crate::eval::scale_by_phase;
use crate::types::{Color, PieceType};

use crate::eval::EvalInfo;
use crate::eval::eval::{CENTER_4, CENTER_SQUARES};

pub fn pawn_eval(board: &Board, info: &EvalInfo) -> (i32, i32) {
    let (white_mg, white_eg) = pawn_eval_raw(board, Color::White, info);
    let (black_mg, black_eg) = pawn_eval_raw(board, Color::Black, info);
    (white_mg - black_mg, white_eg - black_eg)
}

pub fn pawn_eval_raw(board: &Board, color: Color, info: &EvalInfo) -> (i32, i32) {
    let mut mg = 0;
    let mut eg = mg;

    let pawns = board.pieces(color, PieceType::Pawn);

    let (term_mg, term_eg) = stacked_pawns_bonus(board, color, pawns, info);
    mg += term_mg;
    eg += term_eg;
    let (term_mg, term_eg) = center_pawns_bonus(board, color, pawns, info);
    mg += term_mg;
    eg += term_eg;
    let (term_mg, term_eg) = pawn_tempo_bonus(board, color, pawns, info);
    mg += term_mg;
    eg += term_eg;
    let (term_mg, term_eg) = passed_pawn_bonus(board, color, pawns, info);
    mg += term_mg;
    eg += term_eg;
    let (term_mg, term_eg) = pawn_storm_bonus(board, color, pawns, info);
    mg += term_mg;
    eg += term_eg;
    let (term_mg, term_eg) = pawn_chain(board, color, pawns, info);
    mg += term_mg;
    eg += term_eg;
    let (term_mg, term_eg) = isolated_pawns_bonus(board, color, pawns, info);
    mg += term_mg;
    eg += term_eg;
    let (term_mg, term_eg) = backwards_pawn_bonus(board, color, pawns, info);
    mg += term_mg;
    eg += term_eg;

    (mg, eg)
}

pub(super) fn center_pawns_bonus(
    _board: &Board,
    _color: Color,
    pawns: Bitboard,
    info: &EvalInfo,
) -> (i32, i32) {
    if pawns == 0 {
        return (0, 0);
    }

    if info.phase() < 10 {
        return (0, 0);
    }

    let mut mg = 0;
    let mut eg = mg;

    let friendly_pawns = (CENTER_SQUARES & pawns).count_ones() as i32;

    let center_four = (pawns & CENTER_4).count_ones() as i32;

    mg += (4 * friendly_pawns) + (8 * center_four);
    eg += (4 * friendly_pawns) + (8 * center_four);

    (mg, eg)
}

pub(super) fn stacked_pawns_bonus(
    _board: &Board,
    color: Color,
    pawns: Bitboard,
    info: &EvalInfo,
) -> (i32, i32) {
    if pawns == 0 {
        return (0, 0);
    }
    let mut mg = 0;
    let mut eg = mg;

    let mut total = 0;

    for mask in FILE_MASKS {
        let pawns = pawns & mask;
        let pawn_count = pawns.count_ones() as i32;
        if pawn_count <= 1 {
            continue;
        }
        if mask & info.king_ring(color) != 0 {
            // doubled pawn in front of king
            mg -= 15;
            eg -= 15;
        }

        if pawns & CENTER_SQUARES != 0 {
            mg -= 15;
            eg -= 15;
        }

        mg -= pawn_count * 10;
        eg -= pawn_count * 10;
        total += 1;
    }
    mg -= PAWN_WEAKNESS_TABLE[total];
    eg -= PAWN_WEAKNESS_TABLE[total];

    (mg, eg)
}

pub(super) fn isolated_pawns_bonus(
    _board: &Board,
    _color: Color,
    pawns: Bitboard,
    info: &EvalInfo,
) -> (i32, i32) {
    let mut isolated_pawns = pawns;

    let mut pawn_copy = pawns;

    while let Some(sq) = pop_lsb(&mut pawn_copy) {
        let mut adjacent_files = 0u64;

        let file = file_of(sq) as usize;

        if file != 0 {
            adjacent_files |= FILE_MASKS[file - 1];
        }
        if file != 7 {
            adjacent_files |= FILE_MASKS[file + 1];
        }
        isolated_pawns &= !adjacent_files;
    }
    let mg_penalty = if info.phase() > 20 {
        -6
    } else if info.phase() > 14 {
        -8
    } else {
        -10
    };
    let eg_penalty = mg_penalty;
    let total = isolated_pawns.count_ones() as i32;
    let center_iso = (isolated_pawns & CENTER_SQUARES).count_ones() as i32;
    let mut mg = mg_penalty * (total + 2 * center_iso);
    let mut eg = eg_penalty * (total + 2 * center_iso);

    mg -= PAWN_WEAKNESS_TABLE[total as usize];
    eg -= PAWN_WEAKNESS_TABLE[total as usize];

    (mg, eg)
}

pub(super) fn backwards_pawn_bonus(
    _board: &Board,
    color: Color,
    pawns: Bitboard,
    info: &EvalInfo,
) -> (i32, i32) {
    let mut mg = 0;
    let mut eg = mg;
    let mut backwards_pawns = pawns & !info.attacks(color, PieceType::Pawn);

    let mut total = 0;

    while let Some(sq) = pop_lsb(&mut backwards_pawns) {
        let backwards_mask = BACKWARDS_PAWNS[color.idx()][sq as usize];

        let forward = match color {
            Color::White => sq + 8,
            Color::Black => sq - 8,
        };

        if pawns & backwards_mask == 0 {
            let forward_bb = bit(forward);

            if forward_bb & info.all_attacks(color.opposite()) != 0
                && forward_bb & info.all_attacks(color) == 0
            {
                mg -= 25; // backwards pawn
                eg -= 25;
                if RANK_MASKS[rank_of(sq) as usize] & info.king_ring(color) != 0 {
                    // backwards pawn in front of king
                    mg -= 10;
                    eg -= 10;
                }

                total += 1;
            }
        }
    }
    mg -= PAWN_WEAKNESS_TABLE[total];
    eg -= PAWN_WEAKNESS_TABLE[total];
    (mg, eg)
}

pub(super) fn passed_pawn_bonus(
    board: &Board,
    color: Color,
    pawns: Bitboard,
    info: &EvalInfo,
) -> (i32, i32) {
    if pawns == 0 {
        return (0, 0);
    }

    let mut mg = 0;
    let mut eg = mg;

    let enemy_pawns = board.pieces(color.opposite(), PieceType::Pawn);

    let mut pawns = pawns;

    let mut total = 0;

    while let Some(sq) = pop_lsb(&mut pawns) {
        let pawn_mask = PASSED_PAWNS[color.idx()][sq as usize];
        if enemy_pawns & pawn_mask != 0 {
            continue;
        }
        let mut mg_bonus = 0;
        let mut eg_bonus = mg_bonus;
        total += 1;

        let current_mask = bit(sq);

        let mut forward_mask = current_mask;

        match color {
            Color::White => {
                forward_mask <<= 8;
                forward_mask |= forward_mask << 8;
                forward_mask |= forward_mask << 16;
            }
            Color::Black => {
                forward_mask >>= 8;
                forward_mask |= forward_mask >> 8;
                forward_mask |= forward_mask >> 16;
            }
        }

        let is_protected = current_mask & info.all_attacks(color) != 0;

        let is_attacked = current_mask & info.all_attacks(color.opposite()) != 0;

        let is_blocked = forward_mask & board.occupancy_of(color.opposite()) != 0;

        if is_protected && !is_blocked {
            mg_bonus += 30;
            eg_bonus += 30;
        }

        if is_attacked {
            mg_bonus -= 10;
            eg_bonus -= 10;
            if !is_protected {
                mg_bonus -= 20;
                eg_bonus -= 20;
            }
        }

        let available_squares =
            (forward_mask & info.all_attacks(color) & !info.all_attacks(color.opposite()))
                .count_ones() as i32;

        mg_bonus += 5 * available_squares;
        eg_bonus += 5 * available_squares;

        let rank = rank_of(sq);

        let relative_rank = match color {
            Color::White => rank,
            Color::Black => 7 - rank,
        };

        if relative_rank <= 3 {
            // passed pawn but not advanced
            mg += 10;
            eg += 10;
            continue;
        }

        if is_blocked {
            mg_bonus /= 2;
            eg_bonus /= 2;
        }

        if relative_rank == 4 {
            mg += 10 + mg_bonus;
            eg += 10 + eg_bonus;
        } else if relative_rank == 5 {
            mg += 25 + mg_bonus;
            eg += 25 + eg_bonus;
        } else {
            mg += 50 + mg_bonus;
            eg += 50 + eg_bonus;
        }
    }

    if info.phase() < 12 {
        mg *= 2;
        eg *= 2;
    }

    // for now use the reverse
    mg += PAWN_WEAKNESS_TABLE[total];
    eg += PAWN_WEAKNESS_TABLE[total];

    (mg, eg)
}

pub(super) fn pawn_storm_bonus(
    board: &Board,
    color: Color,
    pawns: Bitboard,
    info: &EvalInfo,
) -> (i32, i32) {
    if pawns == 0 {
        return (0, 0);
    }

    let mut mg = 0;
    let mut eg = mg;

    let starting_mask = match color {
        Color::White => RANK_2,
        Color::Black => RANK_7,
    };

    let forward_mask = match color {
        Color::White => RANK_4,
        Color::Black => RANK_5,
    };

    let Some(king_sq) = pop_lsb(&mut board.pieces(color, PieceType::King)) else {
        panic!("No king in pawn_storm_bonus!");
    };

    let king_file = file_of(king_sq);

    let mut king_file_mask = FILE_MASKS[king_file as usize];

    if king_file != 0 {
        king_file_mask |= FILE_MASKS[king_file as usize - 1];
    }
    if king_file != 7 {
        king_file_mask |= FILE_MASKS[king_file as usize + 1];
    }

    let forward_pawns = (pawns & forward_mask & !king_file_mask).count_ones() as i32;
    let starting_pawns = (pawns & starting_mask & !king_file_mask).count_ones() as i32;

    mg -= starting_pawns * 4;
    eg -= starting_pawns * 4;
    mg += forward_pawns * 4;
    eg += forward_pawns * 4;

    // includes two rows ahead
    let enemy_half = match color {
        Color::White => RANK_MASKS[4] | RANK_MASKS[5] | RANK_MASKS[6] | RANK_MASKS[7],
        Color::Black => RANK_MASKS[0] | RANK_MASKS[1] | RANK_MASKS[2] | RANK_MASKS[3],
    };

    let attacking_pawns = pawns & enemy_half & (RANK_MASKS[3] | RANK_MASKS[4]);

    let mut mg_attacking_bonus = attacking_pawns.count_ones() as i32 * 4;
    let mut eg_attacking_bonus = mg_attacking_bonus;

    let mut current_chain = 0;

    for file in FILE_MASKS {
        if attacking_pawns & file != 0 {
            current_chain += 1;
            if current_chain >= 8 {
                mg_attacking_bonus += 80;
                eg_attacking_bonus += 80;
            }
        } else {
            if current_chain > 1 {
                mg_attacking_bonus += 20 * current_chain;
                eg_attacking_bonus += 20 * current_chain;
            } else {
                mg_attacking_bonus -= 5;
                eg_attacking_bonus -= 5;
            }
            current_chain = 0;
        }
    }

    // Friendly pawns that are:
    // - in the enemy half of the board,
    // - not on the king files,
    // - not attacked by an enemy pawn,
    // - and not hanging(not defended while being attacked)
    let aggressive_pawns = pawns
        & enemy_half
        & !king_file_mask
        & !info.attacks(color.opposite(), PieceType::Pawn)
        & !(info.all_attacks(color.opposite()) & !info.all_attacks(color));

    let mg_aggressive_bonus = match aggressive_pawns.count_ones() {
        0 => -10,
        1 => 20,
        2 => 35,
        3 => 60,
        4 => 92,
        5 => 140,
        6 => 180,
        7 => 220,
        8 => 320,
        _ => unreachable!("Somehow more than 8 pawns in pawn_storm_bonus"),
    };
    let eg_aggressive_bonus = mg_aggressive_bonus;

    mg += mg_attacking_bonus + mg_aggressive_bonus;
    eg += eg_attacking_bonus + eg_aggressive_bonus;

    mg = scale_by_phase(mg, info.phase(), 12, 18);
    eg = scale_by_phase(eg, info.phase(), 12, 18);
    (mg, eg)
}

pub(super) fn pawn_tempo_bonus(
    board: &Board,
    color: Color,
    _pawns: Bitboard,
    info: &EvalInfo,
) -> (i32, i32) {
    let pawn_attacks = (board.occupancy_of(color.opposite())
        & !board.pieces(color.opposite(), PieceType::Pawn))
        & info.attacks(color, PieceType::Pawn)
        & !board.pieces(color.opposite(), PieceType::Pawn);

    // finds all pawn attacks(left or right of a pawn) that attack a piece of higher value(not a pawn) and are not attacked by other pawns

    let mg = if pawn_attacks == 0 { 0 } else { 10 };
    let eg = mg;
    (mg, eg)
}

pub(super) fn pawn_chain(
    _board: &Board,
    color: Color,
    pawns: Bitboard,
    info: &EvalInfo,
) -> (i32, i32) {
    if pawns == 0 {
        return (0, 0);
    }
    let defended_pawns = pawns & info.attacks(color, PieceType::Pawn);

    // this is technically temporary/non formal backwards pawns
    let non_defended_pawns = match color {
        Color::White => pawns & !info.attacks(color, PieceType::Pawn) & !RANK_2,
        Color::Black => pawns & !info.attacks(color, PieceType::Pawn) & !RANK_7,
    };

    let mut mg = 0;
    let mut eg = mg;

    mg += defended_pawns.count_ones() as i32 * 4;
    eg += defended_pawns.count_ones() as i32 * 4;

    mg -= non_defended_pawns.count_ones() as i32 * 2;
    eg -= non_defended_pawns.count_ones() as i32 * 2;

    (mg, eg)
}
