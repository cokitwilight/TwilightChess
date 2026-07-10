use crate::bitboard::attacks::all_attacks;
use crate::bitboard::{
    Bitboard, FILE_A, FILE_B, FILE_C, FILE_D, FILE_E, FILE_F, FILE_G, FILE_H, FILE_MASKS, RANK_1,
    RANK_2, RANK_7, RANK_8, Square, bit, file_of, pop_lsb, rank_of, square,
};
use crate::board::Board;
use crate::eval::eval::EvalInfo;
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

    // straights
    score += connected_file_bonus(board, color, straight_sliders, info);
    score += rook_on_the_seventh(board, color, info);
    score += straights_on_open_file(board, color, straight_sliders, info);

    score
}

fn bishop_pair_bonus(board: &Board, color: Color) -> i32 {
    let bishops = board.pieces(color, PieceType::Bishop).count_ones();
    if bishops >= 2 {
        return 20;
    } else {
        return 0;
    }
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

    if info.phase() < 10 {
        // bishop will likely be open and given a huge bonus in an endgame
        score /= 2;
    }

    score
}

fn connected_file_bonus(board: &Board, color: Color, sliders: Bitboard, info: &EvalInfo) -> i32 {
    if sliders == 0 {
        return 0;
    }

    let mut score = 0;

    let mut sliders = sliders;

    let occupied = board.all_occupancy();

    let directions = &[(0, 1), (0, -1)];

    while let Some(sq) = pop_lsb(&mut sliders) {
        for (df, dr) in directions {
            let file = file_of(sq) as i8;
            let mut rank = rank_of(sq) as i8 + dr;

            while (0..8).contains(&file) && (0..8).contains(&rank) {
                let ray_sq = square(file as u8, rank as u8);
                let ray_mask = bit(ray_sq);

                if ray_mask & occupied != 0 {
                    if ray_mask & sliders != 0 {
                        let file_mask = FILE_MASKS[file as usize];

                        let enemy_pawns = (file_mask
                            & board.pieces(color.opposite(), PieceType::Pawn))
                        .count_ones() as i32;
                        let enemy_pieces = file_mask
                            & (board.occupancy_of(color.opposite())
                                & !board.pieces(color.opposite(), PieceType::Pawn));

                        let defended_pieces =
                            (enemy_pieces & info.all_attacks(color)).count_ones() as i32;

                        let undefended_pieces = (enemy_pieces & !info.all_attacks(color.opposite()))
                            .count_ones() as i32;

                        let sees_king_ring = file_mask & info.king_ring(color.opposite()) != 0;

                        if enemy_pawns >= 1 {
                            // not open file
                            score -= 10;
                        }

                        if sees_king_ring && enemy_pawns == 0 {
                            score += 60;
                        } else if sees_king_ring {
                            score += 10;
                        }
                        score += 5 * (defended_pieces - undefended_pieces);

                        score += 20; // general bonus for a doubles rook/queen
                        sliders &= !ray_mask; // removes the detected slider so the bonus isn't double counted
                    }

                    break;
                }
                rank += dr;
            }
        }
    }

    score
}

fn straights_on_open_file(board: &Board, color: Color, sliders: Bitboard, info: &EvalInfo) -> i32 {
    if sliders == 0 {
        return 0;
    }
    if info.phase() < 10 {
        return 0;
    }
    let open_files = open_file_mask(
        board.pieces(color, PieceType::Pawn) | board.pieces(color.opposite(), PieceType::Pawn),
    );

    let open_straights = (open_files & sliders).count_ones() as i32;

    return open_straights * 15;
}

fn rook_on_the_seventh(board: &Board, color: Color, info: &EvalInfo) -> i32 {
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

    if pawns & FILE_A == 0 {
        open_files |= FILE_A;
    }
    if pawns & FILE_B == 0 {
        open_files |= FILE_B;
    }
    if pawns & FILE_C == 0 {
        open_files |= FILE_C;
    }
    if pawns & FILE_D == 0 {
        open_files |= FILE_D;
    }
    if pawns & FILE_E == 0 {
        open_files |= FILE_E;
    }
    if pawns & FILE_F == 0 {
        open_files |= FILE_F;
    }
    if pawns & FILE_G == 0 {
        open_files |= FILE_G;
    }
    if pawns & FILE_H == 0 {
        open_files |= FILE_H;
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
