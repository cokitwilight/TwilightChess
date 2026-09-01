use crate::bitboard::lookup::BETWEEN;
use crate::bitboard::{
    Bitboard, FILE_MASKS, RANK_1, RANK_2, RANK_7, RANK_8, Square, bishop_attacks, bit, file_of,
    pop_lsb, rank_of, square,
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

pub(super) fn bishop_pair_bonus(board: &Board, color: Color) -> i32 {
    let bishops = board.pieces(color, PieceType::Bishop).count_ones();
    if bishops >= 2 {
        return 30;
    } else {
        return 0;
    }
}

pub(super) fn bishop_blocked_by_pawns_bonus(board: &Board, color: Color, _info: &EvalInfo) -> i32 {
    let black_bishop = BLACK_SQUARES & board.pieces(color, PieceType::Bishop);
    let white_bishop = WHITE_SQUARES & board.pieces(color, PieceType::Bishop);

    let mut score = 0;

    if black_bishop != 0 {
        let enemy_pawns =
            (BLACK_SQUARES & CENTER_SQUARES & board.pieces(color.opposite(), PieceType::Pawn))
                .count_ones() as i32;

        let friendly_pawns = (BLACK_SQUARES & CENTER_SQUARES & board.pieces(color, PieceType::Pawn))
            .count_ones() as i32;

        score -= (2 * enemy_pawns) + (5 * friendly_pawns);
    }

    if white_bishop != 0 {
        let enemy_pawns =
            (WHITE_SQUARES & CENTER_SQUARES & board.pieces(color.opposite(), PieceType::Pawn))
                .count_ones() as i32;

        let friendly_pawns = (WHITE_SQUARES & CENTER_SQUARES & board.pieces(color, PieceType::Pawn))
            .count_ones() as i32;

        score -= (2 * enemy_pawns) + (5 * friendly_pawns);
    }

    score
}

pub(super) fn connected_diagonals_bonus(
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

    let enemy_pawns = board.pieces(color.opposite(), PieceType::Pawn);
    let friendly_pawns = board.pieces(color, PieceType::Pawn);

    let relevant_blockers = enemy_pawns
        | friendly_pawns
        | (board.occupancy_of(color.opposite()) & info.attacks(color.opposite(), PieceType::Pawn));

    // relevant blockers includes all pawns and all pieces that are defended by a pawn

    let dy = match color {
        Color::White => 1,
        Color::Black => -1,
    };

    while let Some(sq) = pop_lsb(&mut sliders) {
        let attacks = bishop_attacks(sq, occupied);

        let mut connected = attacks & sliders;

        while let Some(other_sq) = pop_lsb(&mut connected) {
            // connected slider
            score += 15;

            // remaining bonuses should only apply if the connected sliders are actually powerful

            let sq_rank = rank_of(sq);
            let other_rank = rank_of(other_sq);

            let forward_sq = match color {
                Color::White => {
                    if sq_rank > other_rank {
                        sq
                    } else {
                        other_sq
                    }
                }
                Color::Black => {
                    if sq_rank < other_rank {
                        sq
                    } else {
                        other_sq
                    }
                }
            };
            let rear_sq = if forward_sq == sq { other_sq } else { sq };
            let dx = if file_of(forward_sq) > file_of(rear_sq) {
                1
            } else {
                -1
            };

            let edge_sq = ray_edge_square(dx, dy, sq);
            let backwards_edge_sq = ray_edge_square(-dx, -dy, sq);

            let edge_bb = bit(edge_sq);
            let backwards_bb = bit(backwards_edge_sq);

            if bit(forward_sq) & board.pieces(color, PieceType::Queen) != 0 {
                // led by the queen
                score += 15;
            } else {
                score -= 10;
            }

            let total_ray_length =
                (BETWEEN[backwards_edge_sq as usize][edge_sq as usize] | edge_bb | backwards_bb)
                    .count_ones() as i32; // between does not include edges

            if total_ray_length <= 4 {
                // most powerful rays are along 5,6,7 squares long for either black or white squared bishops so avoid anymore bonuses
                continue;
            }

            let xray = BETWEEN[forward_sq as usize][edge_sq as usize] | edge_bb;

            let blocked_ray = bishop_attacks(forward_sq, relevant_blockers) & xray; // only use pawns in the blocked ray

            score += blocked_ray.count_ones() as i32; // prefers longer unblocked rays

            let enemy_pawns = xray & enemy_pawns;

            let friendly_pawns = xray & friendly_pawns;

            // enemy pawns can become targets while friendly pawns generally block the bishop so overvalue friendly pawns

            score -= (enemy_pawns.count_ones() as i32 + friendly_pawns.count_ones() as i32 * 3) * 4;

            let king_ring = info.king_ring(color.opposite());

            if xray & king_ring != 0 {
                if blocked_ray & king_ring != 0 {
                    score += 30; // directly sees the king ring

                    if (blocked_ray & king_ring)
                        & (info.attacked_by_two(color.opposite())
                            & board.occupancy_of(color.opposite()))
                        != 0
                    {
                        score -= 20;
                        // sees the king ring but the square is defended
                    } else {
                        score += 20;
                    }
                } else {
                    score += 10;
                }
            } else {
                score -= 15;
            }
        }
    }

    score
}

pub(super) fn xray_pressure_diagonal_bonus(
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

    let enemy_pawns = board.pieces(color.opposite(), PieceType::Pawn);
    let friendly_pawns = board.pieces(color, PieceType::Pawn);

    let relevant_blockers = enemy_pawns
        | friendly_pawns
        | (board.occupancy_of(color.opposite()) & info.attacks(color.opposite(), PieceType::Pawn));

    let directions: &[(i8, i8); 4] = &[(1, 1), (-1, -1), (-1, 1), (1, -1)];

    while let Some(sq) = pop_lsb(&mut sliders) {
        for (dx, dy) in directions {
            let edge_sq = ray_edge_square(*dx, *dy, sq);
            let full_xray = BETWEEN[sq as usize][edge_sq as usize] | bit(edge_sq);

            if full_xray.count_ones() < 3 {
                continue;
            }

            let blocked_ray = bishop_attacks(sq, relevant_blockers) & full_xray;

            score += blocked_ray.count_ones() as i32 * 2; // prefers longer unblocked rays

            let enemy_pawns = full_xray & enemy_pawns;

            let friendly_pawns = full_xray & friendly_pawns;

            // enemy pawns can become targets while friendly pawns generally block the bishop so overvalue friendly pawns

            score -= (enemy_pawns.count_ones() as i32 + friendly_pawns.count_ones() as i32 * 3) * 4;

            let king_ring = info.king_ring(color.opposite());

            if full_xray & king_ring != 0 {
                if blocked_ray & king_ring != 0 {
                    score += 20; // directly sees the king ring

                    if (blocked_ray & king_ring)
                        & (info.attacked_by_two(color.opposite())
                            & board.occupancy_of(color.opposite()))
                        != 0
                    {
                        score -= 15;
                        // sees the king ring but the square is defended
                    } else {
                        score += 15;
                    }
                } else {
                    score += 10;
                }
            } else {
                score -= 10;
            }
        }
    }

    score
}

pub(super) fn connected_file_bonus(
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

pub(super) fn straights_xray_bonus(
    board: &Board,
    color: Color,
    sliders: Bitboard,
    info: &EvalInfo,
) -> i32 {
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

        let enemy_pawns_on_ray = enemy_pawns & ray;
        let enemies_on_ray = enemy_occupancy & ray;

        let hits = ((enemies_on_ray & !enemy_pawns_on_ray).count_ones() as i32
            - ((friendly_occupancy & ray).count_ones() as i32 - 1))
            .max(0)
            - enemy_pawns_on_ray.count_ones() as i32;

        score += hits * 5; // note open file already gives a bonus to a rook with no pawns on the file

        if file & info.king_ring(color.opposite()) != 0 {
            score += 10;
        }
    }

    scale_by_phase(score, info.phase(), 6, 12)
}

pub(super) fn straights_on_open_file(
    board: &Board,
    color: Color,
    sliders: Bitboard,
    info: &EvalInfo,
) -> i32 {
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

pub(super) fn rook_on_the_seventh(board: &Board, color: Color, _info: &EvalInfo) -> i32 {
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

#[allow(dead_code)]
fn open_file_mask(pawns: Bitboard) -> Bitboard {
    let mut open_files = 0u64;

    for mask in FILE_MASKS {
        if pawns & mask == 0 {
            open_files |= mask;
        }
    }

    open_files
}

#[inline(always)]
fn ray_edge_square(dx: i8, dy: i8, sq: u8) -> Square {
    debug_assert!(dx == -1 || dx == 1);
    debug_assert!(dy == -1 || dy == 1);
    debug_assert!(sq < 64);

    let file = sq & 7;
    let rank = sq >> 3;

    // XOR with 7 reverses a three-bit coordinate: x -> 7 - x.
    let file_steps = file ^ (7 * (dx > 0) as u8);
    let rank_steps = rank ^ (7 * (dy > 0) as u8);
    let steps = file_steps.min(rank_steps) as i16;

    let stride = dx as i16 + 8 * dy as i16;
    (sq as i16 + steps * stride) as Square
}
