use crate::bitboard::lookup::BETWEEN;
use crate::bitboard::{
    Bitboard, FILE_MASKS, RANK_1, RANK_2, RANK_7, RANK_8, Square, bit, file_of, pop_lsb, rank_of,
    square,
};
use crate::board::Board;
use crate::eval::eval::{BLACK_SQUARES, CENTER_SQUARES, EvalInfo, WHITE_SQUARES};
use crate::eval::scale_by_phase;
use crate::types::{Color, PieceType};

pub fn sliders_eval(board: &Board, info: &EvalInfo) -> i32 {
    sliders_eval_raw(board, Color::White, info) - sliders_eval_raw(board, Color::Black, info)
}

pub fn sliders_eval_raw(board: &Board, color: Color, info: &EvalInfo) -> i32 {
    let mut score = 0;

    let diagonal_sliders =
        board.pieces(color, PieceType::Bishop) | board.pieces(color, PieceType::Queen);
    let straight_sliders =
        board.pieces(color, PieceType::Rook) | board.pieces(color, PieceType::Queen);

    // diagonals
    score += connected_diagonals_bonus(board, color, diagonal_sliders, info);
    score += xray_pressure_diagonal_bonus(board, color, diagonal_sliders, info);
    score += bishop_pair_bonus(board, color);
    score += bishop_blocked_by_pawns_bonus(board, color, info);

    // straights
    score += connected_file_bonus(board, color, straight_sliders, info);
    score += rook_on_the_seventh(board, color, info);
    score += straights_on_open_file(board, color, straight_sliders, info);
    score += straights_xray_bonus(board, color, straight_sliders, info);

    score
}

fn bishop_pair_bonus(board: &Board, color: Color) -> i32 {
    let bishops = board.pieces(color, PieceType::Bishop).count_ones();
    if bishops >= 2 {
        return 30;
    } else {
        return 0;
    }
}

fn bishop_blocked_by_pawns_bonus(board: &Board, color: Color, _info: &EvalInfo) -> i32 {
    let black_bishop = BLACK_SQUARES & board.pieces(color, PieceType::Bishop);
    let white_bishop = WHITE_SQUARES & board.pieces(color, PieceType::Bishop);

    let mut score = 0;

    if black_bishop != 0 {
        let enemy_pawns =
            (BLACK_SQUARES & CENTER_SQUARES & board.pieces(color.opposite(), PieceType::Pawn))
                .count_ones() as i32;

        score -= 5 * enemy_pawns;
    }

    if white_bishop != 0 {
        let enemy_pawns =
            (WHITE_SQUARES & CENTER_SQUARES & board.pieces(color.opposite(), PieceType::Pawn))
                .count_ones() as i32;

        score -= 5 * enemy_pawns;
    }

    score
}

fn connected_diagonals_bonus(
    board: &Board,
    color: Color,
    sliders: Bitboard,
    info: &EvalInfo,
) -> i32 {
    if sliders == 0 {
        return 0;
    }

    // avoid adding a large bonus if the connected diagonal doesn't actually see anything

    let mut score = 0;

    let mut sliders = sliders;

    let occupied = board.all_occupancy();

    let directions = &[(1, 1), (1, -1), (-1, 1), (-1, -1)];

    while let Some(sq) = pop_lsb(&mut sliders) {
        for (df, dr) in directions {
            let mut file = file_of(sq) as i8 + df;
            let mut rank = rank_of(sq) as i8 + dr;

            while (0..8).contains(&file) && (0..8).contains(&rank) {
                let ray_sq = square(file as u8, rank as u8);
                let ray_mask = bit(ray_sq);

                if ray_mask & occupied != 0 {
                    if ray_mask & sliders != 0 {
                        let full_ray_mask = ray_bitboard(ray_sq, *df, *dr)
                            | ray_bitboard(ray_sq, -1 * *df, -1 * *dr);

                        let ray_length = full_ray_mask.count_ones();

                        let enemy_pawns = (full_ray_mask
                            & board.pieces(color.opposite(), PieceType::Pawn))
                        .count_ones() as i32;
                        let enemy_pieces = (full_ray_mask
                            & (board.occupancy_of(color.opposite())
                                & !board.pieces(color.opposite(), PieceType::Pawn)))
                        .count_ones() as i32;

                        let sees_king_ring = full_ray_mask & info.king_ring(color.opposite()) != 0;

                        if enemy_pawns >= 2 {
                            score -= 20;
                        }

                        if sees_king_ring && enemy_pawns <= 1 {
                            score += 20;
                        } else if sees_king_ring {
                            score += 5;
                        }

                        if ray_length >= 7 {
                            // longest possible for black and white square bishops
                            score += 20 + (enemy_pieces - enemy_pawns) * 3;
                        } else if ray_length >= 5 {
                            // second longest
                            score += 3 + (enemy_pieces - enemy_pawns) * 3;
                        } else {
                            // doubled in the wrong direction
                            score -= 15 + (enemy_pawns - enemy_pieces) * 3;
                        }

                        score += 20;
                        sliders &= !ray_mask; // dont double count pieces
                    }

                    break;
                }

                file += df;
                rank += dr;
            }
        }
    }

    score
}

fn xray_pressure_diagonal_bonus(
    board: &Board,
    color: Color,
    sliders: Bitboard,
    info: &EvalInfo,
) -> i32 {
    if sliders == 0 {
        return 0;
    }
    let mut score = 0;

    let mut sliders = sliders;

    let directions = &[(1, 1), (1, -1), (-1, 1), (-1, -1)];

    while let Some(sq) = pop_lsb(&mut sliders) {
        for (df, dr) in directions {
            let ray_mask = ray_bitboard(sq, *df, *dr);

            let ray_length = ray_mask.count_ones();

            if ray_length < 2 {
                // only count the long diagonals
                continue;
            }

            let enemy_pawns =
                (ray_mask & board.pieces(color.opposite(), PieceType::Pawn)).count_ones();
            let friendly_pawns = (ray_mask & board.pieces(color, PieceType::Pawn)).count_ones();

            let hanging_pieces = (ray_mask
                & (board.occupancy_of(color.opposite()) & !info.all_attacks(color.opposite())))
            .count_ones();

            let sees_king_ring = ray_mask & info.king_ring(color.opposite()) != 0;

            if enemy_pawns > 2 {
                // severly blocked
                score -= 20;
            } else if enemy_pawns > 0 {
                score -= 10;
            } else {
                score += 20
            }

            if friendly_pawns > 2 {
                // severly blocked but is helping the pawn chain
                score -= 15;
            } else if friendly_pawns > 0 {
                score -= 5;
            } else {
                score += 10;
            }

            if hanging_pieces > 1 {
                // maybe adjust this value
                score += 4 * hanging_pieces as i32;
            }

            if sees_king_ring {
                score += 10;
            }
        }
    }

    score = scale_by_phase(score, info.phase(), 4, 16);

    score
}

fn connected_file_bonus(board: &Board, color: Color, sliders: Bitboard, info: &EvalInfo) -> i32 {
    if sliders == 0 {
        return 0;
    }

    let mut score = 0;

    let mut sliders = sliders;

    let enemy_pawns = board.pieces(color.opposite(), PieceType::Pawn);

    let friendly_pawns = board.pieces(color, PieceType::Pawn);

    while let Some(sq) = pop_lsb(&mut sliders) {
        let file = FILE_MASKS[file_of(sq) as usize];

        let slider = file & sliders; // sliders shouldn't contain the original slider as it was popped(mutable)

        if slider.count_ones() < 1 {
            continue;
        }

        let other_sq = slider.trailing_zeros() as Square;

        let front_sq = match color {
            Color::White => other_sq,
            Color::Black => sq,
        };

        let end_square = match color {
            Color::White => square(file_of(sq), 7),

            Color::Black => square(file_of(sq), 0),
        };

        let between = BETWEEN[sq as usize][other_sq as usize];

        if between & board.all_occupancy() != 0 {
            continue; // not connected
        }

        let ray = BETWEEN[front_sq as usize][end_square as usize] | bit(end_square);

        // only count pawns defended by pawns as these can be a problem.
        let pawn_count =
            (ray & enemy_pawns & info.attacks(color.opposite(), PieceType::Pawn)).count_ones();

        let enemy_pieces = board.occupancy_of(color.opposite()) & !enemy_pawns & ray;

        let blockers =
            (enemy_pieces & info.attacks(color.opposite(), PieceType::Pawn)).count_ones();

        let blocked = blockers >= 1 || friendly_pawns & ray != 0;

        if pawn_count >= 1 || blocked {
            score -= 20;
        } else {
            score += 40;
        }

        score += enemy_pieces.count_ones() as i32 * 5;

        if info.king_ring(color.opposite()) & file != 0 {
            score += 30;
        }
    }

    score
}

fn straights_xray_bonus(board: &Board, color: Color, sliders: Bitboard, info: &EvalInfo) -> i32 {
    if sliders == 0 {
        return 0;
    }

    let mut straights = sliders;

    let enemy_occupancy = board.occupancy_of(color.opposite());
    let friendly_occupancy = board.occupancy_of(color);

    let enemy_pawns = board.pieces(color.opposite(), PieceType::Pawn);

    let mut score = 0;

    while let Some(sq) = pop_lsb(&mut straights) {
        let end_square = match color {
            Color::White => square(file_of(sq), 7),

            Color::Black => square(file_of(sq), 0),
        };
        let ray = BETWEEN[sq as usize][end_square as usize] | bit(end_square);

        let file = FILE_MASKS[file_of(sq) as usize];

        // hits =
        // all enemy pieces besides pawns on the ray
        // minus all the friendly pieces on the ray -> capped at 0
        // minus total enemy pawns

        let hits = ((enemy_occupancy & !enemy_pawns & ray).count_ones() as i32
            - ((friendly_occupancy & ray).count_ones() as i32 - 1))
            .max(0)
            - enemy_pawns.count_ones() as i32;

        score += hits * 5; // note open file already gives a bonus to a rook with no pawns on the file

        if file & info.king_ring(color.opposite()) != 0 {
            score += 10;
        }
    }

    scale_by_phase(score, info.phase(), 6, 12)
}

fn straights_on_open_file(board: &Board, color: Color, sliders: Bitboard, info: &EvalInfo) -> i32 {
    if sliders == 0 {
        return 0;
    }
    let mut score = 0;

    let friendly_pawns = board.pieces(color, PieceType::Pawn);
    let enemy_pawns = board.pieces(color.opposite(), PieceType::Pawn);

    let friendly_occ = board.occupancy_of(color) & !friendly_pawns;
    let enemy_occ = board.occupancy_of(color.opposite()) & !enemy_pawns;

    // add an xray bonus.
    // change to if the file is open from the rooks perseptive not the entire file(encourages lifted rooks)

    let mut sliders = sliders;

    while let Some(sq) = pop_lsb(&mut sliders) {
        let end_square = match color {
            Color::White => square(file_of(sq), 7),

            Color::Black => square(file_of(sq), 0),
        };

        let ray = BETWEEN[sq as usize][end_square as usize] | bit(end_square);

        let friendly_pawn_count = (ray & friendly_pawns).count_ones();
        let enemy_pawn_count =
            (ray & enemy_pawns & info.attacks(color.opposite(), PieceType::Pawn)).count_ones();

        let total_pawns = friendly_pawn_count + enemy_pawn_count;

        if friendly_pawn_count >= 1 {
            // later add if the pawn is a passed pawn give a bonus.
            // NOTE: passed pawn already has a is_defended() bonus so it may double count
            continue;
        } else if total_pawns != 0 {
            // semi-open. Although there can be more than 1 enemy pawns these pawns will be doubled and weak anyways
            score += 10;
        } else {
            score += 20;
        }

        let enemy_pieces = (enemy_occ & ray).count_ones() as i32;

        let friendly_pieces = (friendly_occ & ray).count_ones() as i32;

        score += (2 * enemy_pieces - friendly_pieces) * 5;

        if ray & info.king_ring(color.opposite()) != 0 {
            score += 30;
        }
    }

    scale_by_phase(score, info.phase(), 6, 12)
}

fn rook_on_the_seventh(board: &Board, color: Color, _info: &EvalInfo) -> i32 {
    let mut score = 0;

    let enemy_color = color.opposite();
    let seventh_row = match color {
        Color::White => RANK_7,
        Color::Black => RANK_2,
    };

    let last_row = match color {
        Color::White => RANK_8,
        Color::Black => RANK_1,
    };

    let rooks_on_7th = board.pieces(color, PieceType::Rook) & seventh_row;
    let count = rooks_on_7th.count_ones() as i32;

    if count == 0 {
        return 0;
    }

    score += 10 * count;

    if board.pieces(enemy_color, PieceType::King) & last_row != 0 {
        // king trapped by rook
        score += 20;
    }

    let targets = (board.occupancy_of(enemy_color) & seventh_row).count_ones() as i32;

    score += targets * 3;

    score
}

fn open_file_mask(pawns: Bitboard) -> Bitboard {
    let mut open_files = 0u64;

    for mask in FILE_MASKS {
        if pawns & mask == 0 {
            open_files |= mask;
        }
    }

    open_files
}

fn ray_bitboard(sq: Square, df: i8, dr: i8) -> Bitboard {
    let mut final_mask = bit(sq);

    let mut file = file_of(sq) as i8 + df;
    let mut rank = rank_of(sq) as i8 + dr;
    while (0..8).contains(&file) && (0..8).contains(&rank) {
        let ray_sq = square(file as u8, rank as u8);
        let ray_mask = bit(ray_sq);

        final_mask |= ray_mask;

        file += df;
        rank += dr;
    }

    final_mask
}
