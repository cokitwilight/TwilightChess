use crate::bitboard::{
    Bitboard, FILE_A, FILE_H, FILE_MASKS, RANK_2, RANK_7, RANK_MASKS, bit, black_pawn_attacks,
    file_of, pawn_attacks_from_square, pop_lsb, rank_of, rook_attacks, white_pawn_attacks,
};
use crate::board::Board;
use crate::eval::lookup::{BACKWARDS_PAWNS, PASSED_PAWNS};
use crate::types::{Color, PieceType};

use crate::eval::EvalInfo;
use crate::eval::eval::{CENTER_4, CENTER_FILES, CENTER_SQUARES};

// SF10 connected-pawn rank terms in its internal units, with and without a phalanx.
const PHALANX_RANK: [i32; 8] = [0, 18, 21, 41, 82, 137, 252, 0];
const CONNECTED_RANK: [i32; 8] = [0, 13, 24, 18, 65, 100, 175, 0];

const CHAIN_MG: [i32; 8] = [0, 2, 7, 16, 34, 50, 90, 0];
const CHAIN_EG: [i32; 8] = [0, 1, 2, 4, 20, 40, 96, 0];

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
    let (term_mg, term_eg) = phalanx_pawns(board, color, pawns, info);
    mg += term_mg;
    eg += term_eg;
    let (term_mg, term_eg) = pawn_safe_push(board, color, pawns, info);
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

    for mask in FILE_MASKS {
        let pawns = pawns & mask;
        let pawn_count = pawns.count_ones() as i32;
        if pawn_count <= 1 {
            continue;
        }

        if mask == FILE_A || mask == FILE_H {
            eg += -20 * (pawn_count - 1);
        }

        mg += -3 * (pawn_count - 1);
        eg += -5 * (pawn_count - 1);

        if info.attacks(color, PieceType::Pawn) & pawns == 0 {
            // undefended
            mg += -7;
            eg += -12;
        }
    }

    (mg, eg)
}

pub(super) fn isolated_pawns_bonus(
    _board: &Board,
    color: Color,
    pawns: Bitboard,
    info: &EvalInfo,
) -> (i32, i32) {
    let mut isolated_pawns = pawns;

    let mut pawn_copy = pawns;

    let mut mg = 0;
    let mut eg = 0;

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

    let attacked = (isolated_pawns
        & (info.attacks(color.opposite(), PieceType::Rook)
            | info.attacks(color.opposite(), PieceType::Queen)))
    .count_ones() as i32;

    mg += -3 * isolated_pawns.count_ones() as i32 + -4 * attacked;
    eg += -7 * isolated_pawns.count_ones() as i32 + -3 * attacked;

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

    while let Some(sq) = pop_lsb(&mut backwards_pawns) {
        let backwards_mask = BACKWARDS_PAWNS[color.idx()][sq as usize];

        if pawns & backwards_mask == 0 {
            let forward = match color {
                Color::White => sq + 8,
                Color::Black => sq - 8,
            };

            let forward_bb = bit(forward);

            if forward_bb
                & (info.all_attacks(color.opposite()) & !info.attacks(color, PieceType::Pawn))
                != 0
                && forward_bb & info.all_attacks(color) == 0
            {
                mg += -5; // backwards pawn
                eg += -11;
            }
        }
    }
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

    while let Some(sq) = pop_lsb(&mut pawns) {
        let pawn_mask = PASSED_PAWNS[color.idx()][sq as usize];
        if enemy_pawns & pawn_mask != 0 {
            continue;
        }
        let mut mg_bonus = 0;
        let mut eg_bonus = mg_bonus;

        let current_mask = bit(sq);

        let forward = match color {
            Color::White => current_mask << 8,
            Color::Black => current_mask >> 8,
        };

        let file = FILE_MASKS[file_of(sq) as usize];
        let forward_mask = pawn_mask & file;

        let is_candidate = current_mask & (FILE_A | FILE_H) != 0;

        let is_protected = current_mask & info.all_attacks(color) != 0;

        let is_attacked = current_mask & info.all_attacks(color.opposite()) != 0;

        let is_blocked = forward & board.all_occupancy() != 0;

        let path_blocked = forward_mask & board.all_occupancy() != 0;

        let protected_behind_passer = rook_attacks(sq, board.all_occupancy())
            & file
            & !forward_mask
            & (board.pieces(color, PieceType::Rook) | board.pieces(color, PieceType::Queen))
            != 0;

        if is_protected {
            mg_bonus += 15;
            eg_bonus += 25;
        }

        if protected_behind_passer {
            mg_bonus += 10;
            eg_bonus += 30;
        }

        if is_attacked {
            mg_bonus -= 10;
            eg_bonus -= 20;
            if !is_protected {
                mg_bonus -= 20;
                eg_bonus -= 30;
            }
        }

        if is_candidate {
            mg_bonus += 15;
            eg_bonus += 35;
        }

        if !path_blocked {
            let available_squares =
                (forward_mask & info.all_attacks(color) & !info.all_attacks(color.opposite()))
                    .count_ones() as i32;

            mg_bonus += 7 * available_squares;
            eg_bonus += 12 * available_squares;
        }

        let rank = rank_of(sq);

        let relative_rank = match color {
            Color::White => rank,
            Color::Black => 7 - rank,
        };

        let (base_mg, base_eg) = match relative_rank {
            1 => (2, 9),
            2 => (6, 11),
            3 => (5, 15),
            4 => (27, 30),
            5 => (78, 80),
            6 => (130, 120),
            _ => (0, 0),
        };
        mg += base_mg;
        eg += base_eg;
        if is_blocked {
            mg -= base_mg / 4;
            eg -= base_eg / 2;
        }
        if relative_rank >= 4 {
            mg += mg_bonus;
            eg += eg_bonus;
        }
    }

    (mg, eg)
}

// REWRITE
pub(super) fn pawn_storm_bonus(
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

    let starting_mask = match color {
        Color::White => RANK_2,
        Color::Black => RANK_7,
    };

    let king_sq = info.king_square(color);

    let king_file = file_of(king_sq);

    let mut king_file_mask = FILE_MASKS[king_file as usize];

    if king_file != 0 {
        king_file_mask |= FILE_MASKS[king_file as usize - 1];
    }
    if king_file != 7 {
        king_file_mask |= FILE_MASKS[king_file as usize + 1];
    }

    // punish pawns on starting row
    let starting_pawns = (pawns & starting_mask & !king_file_mask).count_ones() as i32;

    mg -= starting_pawns * 3;
    eg -= starting_pawns * 6;

    // includes two rows ahead
    let enemy_half = match color {
        Color::White => RANK_MASKS[4] | RANK_MASKS[5] | RANK_MASKS[6] | RANK_MASKS[7],
        Color::Black => RANK_MASKS[0] | RANK_MASKS[1] | RANK_MASKS[2] | RANK_MASKS[3],
    };

    let advanced = (pawns & enemy_half & !king_file_mask).count_ones() as i32;

    mg += 4 * advanced;
    eg += 6 * advanced;
    // let chains and phalanx be done separately

    (mg, eg)
}

pub(super) fn pawn_tempo_bonus(
    board: &Board,
    color: Color,
    pawns: Bitboard,
    info: &EvalInfo,
) -> (i32, i32) {
    let mut mg = 0;
    let mut eg = 0;

    let safe_pawns = pawns & (!info.all_attacks(color.opposite()) | info.all_attacks(color));

    let attacks = match color {
        Color::White => white_pawn_attacks(safe_pawns),
        Color::Black => black_pawn_attacks(safe_pawns),
    };

    let pieces = board.occupancy_of(color.opposite())
        & !board.pieces(color.opposite(), PieceType::King)
        & !board.pieces(color.opposite(), PieceType::Pawn);

    let attack_count = (attacks & pieces).count_ones() as i32;

    mg += attack_count * 80;
    eg += attack_count * 40;

    (mg, eg)
}

pub(super) fn pawn_safe_push(
    board: &Board,
    color: Color,
    pawns: Bitboard,
    info: &EvalInfo,
) -> (i32, i32) {
    let mut mg = 0;
    let mut eg = 0;

    let mut pushes = match color {
        Color::White => (pawns << 8) & !board.all_occupancy(),
        Color::Black => (pawns >> 8) & !board.all_occupancy(),
    };
    // !enemy_pawn_attacks & (!enemy_all_attacks | friendly_all_attacks)
    pushes = pushes
        & !info.attacks(color.opposite(), PieceType::Pawn)
        & (!info.all_attacks(color.opposite()) | info.all_attacks(color));

    let targets = match color {
        Color::White => {
            white_pawn_attacks(pushes)
                & board.occupancy_of(color.opposite())
                & !board.pieces(color.opposite(), PieceType::King)
                & !board.pieces(color.opposite(), PieceType::Pawn)
        }
        Color::Black => {
            black_pawn_attacks(pushes)
                & board.occupancy_of(color.opposite())
                & !board.pieces(color.opposite(), PieceType::King)
                & !board.pieces(color.opposite(), PieceType::Pawn)
        }
    };

    let count = targets.count_ones() as i32;

    mg += 15 * count;
    eg += 12 * count;

    (mg, eg)
}

pub(super) fn phalanx_pawns(
    board: &Board,
    color: Color,
    pawns: Bitboard,
    _info: &EvalInfo,
) -> (i32, i32) {
    let mut mg = 0;
    let mut eg = 0;

    let enemy_pawns = board.pieces(color.opposite(), PieceType::Pawn);
    let mut members = pawns & (((pawns & !FILE_H) << 1) | ((pawns & !FILE_A) >> 1));
    while let Some(sq) = pop_lsb(&mut members) {
        let rank = rank_of(sq);

        let relative_rank = match color {
            Color::White => rank,
            Color::Black => 7 - rank,
        } as usize;

        let ahead = PASSED_PAWNS[color.idx()][sq as usize] & FILE_MASKS[file_of(sq) as usize];
        let opposed = u32::from(ahead & enemy_pawns != 0);
        let supporters =
            (pawn_attacks_from_square(color.opposite(), sq) & pawns).count_ones() as i32;
        let support = 17 * supporters;
        let with_phalanx = support + (PHALANX_RANK[relative_rank] >> opposed);
        // A supported pawn already receives a chain score. Add only the phalanx
        // increment for that pawn, preserving the engine's independent chain tuning.
        let without_phalanx = if supporters == 0 {
            0
        } else {
            support + (CONNECTED_RANK[relative_rank] >> opposed)
        };
        let eg_rank = relative_rank as i32 - 2;
        mg += with_phalanx - without_phalanx;
        eg += with_phalanx * eg_rank / 4 - without_phalanx * eg_rank / 4;
    }

    // Round the side's total once, including negative second-rank EG scores.
    let to_cp = |value: i32| (value * 100 + value.signum() * 104) / 208;
    (to_cp(mg), to_cp(eg))
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

    let mut mg = 0;
    let mut eg = mg;

    let mut pawns_copy = pawns;

    while let Some(sq) = pop_lsb(&mut pawns_copy) {
        let rank = rank_of(sq);

        let relative_rank = match color {
            Color::White => rank,
            Color::Black => 7 - rank,
        } as usize;

        let square = bit(sq);

        let supporters = pawn_attacks_from_square(color.opposite(), sq) & pawns;
        let supports = pawn_attacks_from_square(color, sq) & pawns;

        let is_base = supporters == 0 && supports != 0;
        let is_head = supporters != 0 && supports == 0 && (square & CENTER_FILES != 0);

        if supporters != 0 || supports != 0 {
            mg += CHAIN_MG[relative_rank];
            eg += CHAIN_EG[relative_rank];

            if is_base {
                let protected = square & info.all_attacks(color) != 0;
                let attacked = square & info.all_attacks(color.opposite()) != 0;

                if attacked && !protected {
                    mg += -24;
                    eg += -18;
                } else if attacked {
                    // pawn chain is still weaker since a more valuable piece must defend it
                    mg += -7;
                    eg += -4;
                }
            }

            if is_head {
                let support_count = supporters.count_ones() as i32;
                mg += 6 * support_count;
                eg += 4 * support_count;
            }
        }
    }

    let chain_targets = ((pawns & CENTER_FILES) & info.attacks(color.opposite(), PieceType::Pawn))
        .count_ones() as i32;

    mg += -12 * chain_targets;
    eg += -7 * chain_targets;

    (mg, eg)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn either_color_blocking_promotion_reduces_both_passer_scores() {
        for (color, open, blocked) in [
            (
                Color::White,
                "7k/4P3/8/8/8/8/8/K7 w - - 0 1",
                [
                    "4N2k/4P3/8/8/8/8/8/K7 w - - 0 1",
                    "4n2k/4P3/8/8/8/8/8/K7 w - - 0 1",
                ],
            ),
            (
                Color::Black,
                "k7/8/8/8/8/8/4p3/7K b - - 0 1",
                [
                    "k7/8/8/8/8/8/4p3/4n2K b - - 0 1",
                    "k7/8/8/8/8/8/4p3/4N2K b - - 0 1",
                ],
            ),
        ] {
            let score = |fen| {
                let board = Board::from_fen(fen).unwrap();
                let pawns = board.pieces(color, PieceType::Pawn);
                passed_pawn_bonus(&board, color, pawns, &EvalInfo::calculate(&board))
            };
            let (open_mg, open_eg) = score(open);
            for fen in blocked {
                let (mg, eg) = score(fen);
                assert!(mg < open_mg && eg < open_eg, "{fen}");
            }
        }
    }
}
