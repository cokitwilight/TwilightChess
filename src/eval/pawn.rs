use crate::bitboard::{
    Bitboard, FILE_MASKS, RANK_2, RANK_3, RANK_6, RANK_7, RANK_MASKS, Square, bit, file_of,
    pop_lsb, rank_of,
};
use crate::board::Board;
use crate::types::{Color, PieceType};

use crate::eval::eval::{CENTER_SQUARES, EvalInfo};

pub fn pawn_eval(board: &Board, info: &EvalInfo) -> i32 {
    pawn_eval_raw(board, Color::White, info) - pawn_eval_raw(board, Color::Black, info)
}

pub fn pawn_eval_raw(board: &Board, color: Color, info: &EvalInfo) -> i32 {
    let mut score = 0;

    let pawns = board.pieces(color, PieceType::Pawn);

    score += stacked_pawns_bonus(board, color, pawns, info);
    score += center_pawns_bonus(board, color, pawns, info);
    score += pawn_tempo_bonus(board, color, pawns, info);
    score += passed_pawn_bonus(board, color, pawns, info);
    score += pawn_storm_bonus(board, color, pawns, info);
    score += pawn_chain(board, color, pawns, info);
    score += isolated_pawns_bonus(board, color, pawns, info);
    score += backwards_pawn_bonus(board, color, pawns, info);

    score
}

pub fn print_pawn_eval(board: &Board) {
    let info = &EvalInfo::calculate(board);
    let w_pawns = board.pieces(Color::White, PieceType::Pawn);
    let b_pawns = board.pieces(Color::Black, PieceType::Pawn);

    println!();
    println!("================ PAWN EVAL BREAKDOWN ================");
    println!(
        "{:<22} {:>8} {:>8} {:>10}",
        "Feature", "White", "Black", "Net"
    );
    println!("{:-<52}", "");

    print_pawn_eval_row(
        "Stacked pawns",
        stacked_pawns_bonus(board, Color::White, w_pawns, info),
        stacked_pawns_bonus(board, Color::White, b_pawns, info),
    );
    print_pawn_eval_row(
        "Center pawns",
        center_pawns_bonus(board, Color::White, w_pawns, info),
        center_pawns_bonus(board, Color::Black, b_pawns, info),
    );
    print_pawn_eval_row(
        "Pawn tempo",
        pawn_tempo_bonus(board, Color::White, w_pawns, info),
        pawn_tempo_bonus(board, Color::Black, b_pawns, info),
    );
    print_pawn_eval_row(
        "Passed pawns",
        passed_pawn_bonus(board, Color::White, w_pawns, info),
        passed_pawn_bonus(board, Color::Black, b_pawns, info),
    );
    print_pawn_eval_row(
        "Pawn storm",
        pawn_storm_bonus(board, Color::White, w_pawns, info),
        pawn_storm_bonus(board, Color::Black, b_pawns, info),
    );
    print_pawn_eval_row(
        "Pawn chain",
        pawn_chain(board, Color::White, w_pawns, info),
        pawn_chain(board, Color::Black, b_pawns, info),
    );
    print_pawn_eval_row(
        "Isolated pawns",
        isolated_pawns_bonus(board, Color::White, w_pawns, info),
        isolated_pawns_bonus(board, Color::Black, b_pawns, info),
    );
    print_pawn_eval_row(
        "Backward pawns",
        backwards_pawn_bonus(board, Color::White, w_pawns, info),
        backwards_pawn_bonus(board, Color::Black, b_pawns, info),
    );

    println!("{:-<52}", "");
}

fn print_pawn_eval_row(name: &str, white: i32, black: i32) {
    println!(
        "{:<22} {:>+8} {:>+8} {:>+10}",
        name,
        white,
        black,
        white - black,
    );
}

fn center_pawns_bonus(_board: &Board, _color: Color, pawns: Bitboard, info: &EvalInfo) -> i32 {
    if pawns == 0 {
        return 0;
    }

    if info.phase() < 10 {
        return 0;
    }

    let mut score = 0;

    let friendly_pawns = (CENTER_SQUARES & pawns).count_ones() as i32;

    if friendly_pawns == 0 {
        score -= 20;
    } else {
        score += 6 * friendly_pawns;
    }

    score
}

fn stacked_pawns_bonus(_board: &Board, _color: Color, pawns: Bitboard, _info: &EvalInfo) -> i32 {
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

fn isolated_pawns_bonus(_board: &Board, _color: Color, pawns: Bitboard, info: &EvalInfo) -> i32 {
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
    let penalty = if info.phase() > 20 {
        -3
    } else if info.phase() > 14 {
        -6
    } else {
        -8
    };
    return penalty * isolated_pawns.count_ones() as i32;
}

fn backwards_pawn_bonus(_board: &Board, color: Color, pawns: Bitboard, info: &EvalInfo) -> i32 {
    let mut score = 0;
    let mut backwards_pawns = pawns & !info.attacks(color, PieceType::Pawn);

    while let Some(sq) = pop_lsb(&mut backwards_pawns) {
        let backwards_mask = backwards_pawn_mask(sq, color);

        let forward = match color {
            Color::White => sq + 8,
            Color::Black => sq - 8,
        };

        if pawns & backwards_mask == 0 {
            let forward_bb = bit(forward);

            if forward_bb & info.all_attacks(color.opposite()) != 0
                && forward_bb & info.all_attacks(color) == 0
            {
                score -= 15; // backwards pawn
            }
        }
    }

    score
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

        let is_attacked = current_mask & info.all_attacks(color.opposite()) != 0;

        let is_blocked = forward_mask & board.occupancy_of(color.opposite()) != 0;

        if is_protected {
            bonus += 10;
            if current_mask & info.attacks(color, PieceType::Pawn) != 0 {
                bonus += 20;
            }
        }

        if is_attacked {
            bonus -= 10;
            if !is_protected {
                bonus -= 20;
            }
        }

        let available_squares =
            (forward_mask & info.all_attacks(color) & !info.all_attacks(color.opposite()))
                .count_ones() as i32;

        bonus += 5 * available_squares;

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

        if is_blocked {
            bonus /= 2;
        }

        if relative_rank == 4 {
            score += 10 + bonus;
        } else if relative_rank == 5 {
            score += 25 + bonus;
        } else {
            score += 50 + bonus;
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

    if info.phase() < 12 {
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
    // - not attacked by an enemy pawn,
    // - and not hanging(not defended while being attacked)
    let aggressive_pawns = pawns
        & enemy_half
        & !king_file_mask
        & !info.attacks(color.opposite(), PieceType::Pawn)
        & !(info.all_attacks(color.opposite()) & !info.all_attacks(color));

    let defended_pawns = aggressive_pawns & info.all_attacks(color);

    let bonus = match aggressive_pawns.count_ones() {
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

    let defended_bonus = match defended_pawns.count_ones() {
        0 => 0,
        1 => 12,
        2 => 22,
        3 => 42,
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

fn pawn_tempo_bonus(board: &Board, color: Color, _pawns: Bitboard, info: &EvalInfo) -> i32 {
    let pawn_attacks = (board.occupancy_of(color.opposite())
        & !board.pieces(color.opposite(), PieceType::Pawn))
        & info.attacks(color, PieceType::Pawn)
        & !board.pieces(color.opposite(), PieceType::Pawn);

    // finds all pawn attacks(left or right of a pawn) that attack a piece of higher value(not a pawn) and are not attacked by other pawns

    if pawn_attacks == 0 { 0 } else { 10 }
}

fn pawn_chain(_board: &Board, color: Color, pawns: Bitboard, info: &EvalInfo) -> i32 {
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

fn backwards_pawn_mask(sq: Square, color: Color) -> Bitboard {
    let file = file_of(sq);
    let mut mask = 0u64;

    if file != 0 {
        mask |= bit(sq - 1);
    }

    if file != 7 {
        mask |= bit(sq + 1);
    }

    match color {
        Color::White => {
            mask = mask >> 8;
            mask |= mask >> 8;
            mask |= mask >> 16;
            mask |= mask >> 32;
        }
        Color::Black => {
            mask = mask << 8;
            mask |= mask << 8;
            mask |= mask << 16;
            mask |= mask << 32;
        }
    }

    mask
}
