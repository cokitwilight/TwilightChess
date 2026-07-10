use crate::bitboard::{
    Bitboard, FILE_MASKS, RANK_2, RANK_3, RANK_6, RANK_7, RANK_MASKS, Square, bit, file_of,
    pawn_attacks, pop_lsb, rank_of,
};
use crate::board::Board;
use crate::types::{Color, PieceType};

use crate::eval::eval::EvalInfo;

pub fn pawn_eval(board: &Board, info: &EvalInfo) -> i32 {
    pawn_eval_raw(board, Color::White, info) - pawn_eval_raw(board, Color::Black, info)
}

pub fn pawn_eval_raw(board: &Board, color: Color, info: &EvalInfo) -> i32 {
    let mut score = 0;

    let pawns = board.pieces(color, PieceType::Pawn);

    score += stacked_pawns_bonus(board, color, pawns, info);
    score += passed_pawn_bonus(board, color, pawns, info);
    score += pawn_storm_bonus(board, color, pawns, info);
    score += pawn_chain(board, color, pawns, info);
    score += isolated_pawns_bonus(board, color, pawns, info);

    score
}

fn stacked_pawns_bonus(board: &Board, color: Color, pawns: Bitboard, info: &EvalInfo) -> i32 {
    if pawns == 0 {
        return 0;
    }
    let mut score = 0;

    for mask in FILE_MASKS {
        let pawns = (pawns & mask).count_ones() as i32;
        if pawns == 2 {
            score -= 10;
        } else if pawns == 3 {
            score -= 30;
        } else if pawns > 3 {
            score -= 15 * pawns;
        }
    }

    score
}

fn isolated_pawns_bonus(board: &Board, color: Color, pawns: Bitboard, info: &EvalInfo) -> i32 {
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
    return -15 * isolated_pawns.count_ones() as i32;
}

fn passed_pawn_bonus(board: &Board, color: Color, pawns: Bitboard, info: &EvalInfo) -> i32 {
    if pawns == 0 {
        return 0;
    }

    let mut score = 0;

    let enemy_pawns = board.pieces(color.opposite(), PieceType::Pawn);

    let mut pawns = pawns;

    while let Some(sq) = pop_lsb(&mut pawns) {
        let pawn_mask = passed_pawn_mask(sq, color);
        if enemy_pawns & pawn_mask != 0 {
            continue;
        }
        let mut bonus = 0;

        let current_mask = bit(sq);

        let mut forward_mask = current_mask;

        match color {
            Color::White => {
                forward_mask = forward_mask << 8;
                forward_mask |= forward_mask << 8;
                forward_mask |= forward_mask << 16;
            }
            Color::Black => {
                forward_mask = forward_mask >> 8;
                forward_mask |= forward_mask >> 8;
                forward_mask |= forward_mask >> 16;
            }
        }

        let is_protected = current_mask & info.all_attacks(color) != 0;

        if is_protected {
            bonus += 10;
            if current_mask & info.attacks(color, PieceType::Pawn) != 0 {
                bonus += 20;
            }
        }

        let protected_squares = (forward_mask & info.all_attacks(color)).count_ones() as i32;

        bonus += 5 * protected_squares;

        let rank = rank_of(sq);

        let relative_rank = match color {
            Color::White => rank,
            Color::Black => 7 - rank,
        };

        if relative_rank <= 3 {
            // passed pawn but not advanced
            score += 10;
            continue;
        }

        if relative_rank == 4 {
            score += 10 + bonus;
        } else if relative_rank == 5 {
            score += 30 + bonus;
        } else {
            score += 60 + bonus;
        }
    }

    if info.phase() < 12 {
        score *= 2;
    }

    score
}

fn pawn_storm_bonus(board: &Board, color: Color, pawns: Bitboard, info: &EvalInfo) -> i32 {
    if pawns == 0 {
        return 0;
    }
    if info.phase() > 20 {
        return 0;
    }

    let mut score = 0;

    let starting_mask = match color {
        Color::White => RANK_2 | RANK_3,
        Color::Black => RANK_7 | RANK_6,
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

    let starting_pawns = (pawns & starting_mask & !king_file_mask).count_ones();

    score -= starting_pawns as i32 * -8;

    let enemy_half = match color {
        Color::White => RANK_MASKS[4] | RANK_MASKS[5] | RANK_MASKS[6] | RANK_MASKS[7],
        Color::Black => RANK_MASKS[0] | RANK_MASKS[1] | RANK_MASKS[2] | RANK_MASKS[3],
    };

    // Friendly pawns that are:
    // - in the enemy half of the board,
    // - not on the king files,
    // - defended by at least one friendly piece,
    // - and not attacked by an enemy pawn.
    let aggressive_pawns =
        pawns & enemy_half & !king_file_mask & !info.attacks(color.opposite(), PieceType::Pawn);

    let defended_pawns = aggressive_pawns & info.all_attacks(color);

    // TODO: Maybe phalanx and non isolated pawns?
    // while let Some(sq) = pop_lsb(&mut aggressive_pawns) {
    //     let rank = rank_of(sq) as usize;
    //     let file = file_of(sq) as usize;

    //     let rank_mask = RANK_MASKS[rank];
    //     let mut file_mask = FILE_MASKS[file];

    //     if file > 0 {
    //         file_mask |= FILE_MASKS[file - 1];
    //     }

    //     if file < 7 {
    //         file_mask |= FILE_MASKS[file + 1];
    //     }
    // }

    let bonus = match aggressive_pawns.count_ones() {
        0 => -10,
        1 => 8,
        2 => 20,
        3 => 32,
        4 => 52,
        5 => 72,
        6 => 102,
        7 => 150,
        8 => 220,
        _ => unreachable!("Somehow more than 8 pawns in pawn_storm_bonus"),
    };

    let defended_bonus = match defended_pawns.count_ones() {
        0 => 0,
        1 => 16,
        2 => 32,
        3 => 52,
        4 => 72,
        5 => 150,
        6 => 190,
        7 => 250,
        8 => 300,
        _ => unreachable!("Somehow more than 8 pawns in pawn_storm_bonus"),
    };

    score += bonus + defended_bonus;

    if info.phase() < 10 {
        // don't aggressively add since passsed pawns already counts this
        score /= 2;
    }

    score
}

fn pawn_chain(board: &Board, color: Color, pawns: Bitboard, info: &EvalInfo) -> i32 {
    if pawns == 0 {
        return 0;
    }
    let defended_pawns = pawns & info.attacks(color, PieceType::Pawn);

    return defended_pawns.count_ones() as i32 * 4;
}

fn passed_pawn_mask(sq: Square, color: Color) -> Bitboard {
    let file = file_of(sq);
    let mut mask = bit(sq);

    if file != 0 {
        // pawn not on the outer left file
        mask |= bit(sq - 1);
    }

    if file != 7 {
        // pawn not on the outer right file
        mask |= bit(sq + 1);
    }

    match color {
        Color::White => {
            mask = mask << 8;
            mask |= mask << 8;
            mask |= mask << 16;
            mask |= mask << 32;
        }
        Color::Black => {
            mask = mask >> 8;
            mask |= mask >> 8;
            mask |= mask >> 16;
            mask |= mask >> 32;
        }
    }
    mask
}
