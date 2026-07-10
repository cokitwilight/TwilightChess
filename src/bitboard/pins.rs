use crate::bitboard::lookup::{BETWEEN, squares_aligned};
use crate::bitboard::{
    Bitboard, Square, bit, file_of, knight_attacks, pawn_attacks_from_square, pop_lsb, rank_of,
    square,
};
use crate::board::Board;
use crate::types::{Color, PieceType};

pub fn generate_pin_masks(board: &Board, color: Color, king_sq: Square) -> [Bitboard; 64] {
    let mut pin_mask = [!0u64; 64];

    let straights = [(1, 0), (-1, 0), (0, 1), (0, -1)];
    let diagonals = [(1, 1), (1, -1), (-1, 1), (-1, -1)];

    let enemies = board.occupancy_of(color.opposite());

    let enemy_straights = board.pieces(color.opposite(), PieceType::Rook)
        | board.pieces(color.opposite(), PieceType::Queen);

    let enemy_diagonals = board.pieces(color.opposite(), PieceType::Bishop)
        | board.pieces(color.opposite(), PieceType::Queen);

    let friends = board.occupancy_of(color);

    for (df, dr) in straights {
        let mut file = file_of(king_sq) as i8 + df;
        let mut rank = rank_of(king_sq) as i8 + dr;

        while (0..8).contains(&file) && (0..8).contains(&rank) {
            // scans outward until it finds first enemy slider
            let sq = square(file as u8, rank as u8);
            let sq_mask = bit(sq);

            if sq_mask & enemies == 0 {
                file += df;
                rank += dr;
                continue;
            }

            // First piece is not a slider
            if sq_mask & enemy_straights == 0 {
                break;
            }

            // only enemy slider can be on sq
            let between_mask = BETWEEN[king_sq as usize][sq as usize];

            let mut blockers = between_mask & friends;

            if blockers.count_ones() == 1 {
                let pinned_sq = pop_lsb(&mut blockers).unwrap(); // since count ones returns 1 this should never be none

                pin_mask[pinned_sq as usize] = between_mask | bit(king_sq) | bit(sq);
                break;
            }
            break;
        }
    }

    for (df, dr) in diagonals {
        let mut file = file_of(king_sq) as i8 + df;
        let mut rank = rank_of(king_sq) as i8 + dr;

        while (0..8).contains(&file) && (0..8).contains(&rank) {
            // scans outward until it finds first enemy slider
            let sq = square(file as u8, rank as u8);
            let sq_mask = bit(sq);

            if sq_mask & enemies == 0 {
                file += df;
                rank += dr;
                continue;
            }

            // First piece is not a slider
            if sq_mask & enemy_diagonals == 0 {
                break;
            }

            // only enemy slider can be on sq
            let between_mask = BETWEEN[king_sq as usize][sq as usize];

            let mut blockers = between_mask & friends;

            if blockers.count_ones() == 1 {
                let pinned_sq = pop_lsb(&mut blockers).unwrap(); // since count ones returns 1 this should never be none

                pin_mask[pinned_sq as usize] = between_mask | bit(king_sq) | bit(sq);
                break;
            }
            break;
        }
    }

    pin_mask
}

pub fn generate_checkers_and_check_mask(
    board: &Board,
    color: Color,
    king_sq: Square,
) -> (Bitboard, Bitboard) {
    let enemy = color.opposite();

    let occupied = board.all_occupancy();
    let friends = board.occupancy_of(color);
    let enemies = board.occupancy_of(enemy);

    let enemy_straights =
        board.pieces(enemy, PieceType::Rook) | board.pieces(enemy, PieceType::Queen);

    let enemy_diagonals =
        board.pieces(enemy, PieceType::Bishop) | board.pieces(enemy, PieceType::Queen);

    let mut checkers = 0u64;

    let straights = [(1, 0), (-1, 0), (0, 1), (0, -1)];
    let diagonals = [(1, 1), (1, -1), (-1, 1), (-1, -1)];

    for (df, dr) in straights {
        let mut file = file_of(king_sq) as i8 + df;
        let mut rank = rank_of(king_sq) as i8 + dr;

        while (0..8).contains(&file) && (0..8).contains(&rank) {
            // scans outward until it finds first enemy slider
            let sq = square(file as u8, rank as u8);
            let sq_mask = bit(sq);

            // empty square
            if sq_mask & occupied == 0 {
                file += df;
                rank += dr;
                continue;
            }

            // First piece is a friendly
            if sq_mask & friends != 0 {
                break;
            }

            if sq_mask & enemies != 0 {
                if sq_mask & enemy_straights != 0 {
                    checkers |= sq_mask;
                }
                break;
            }
            break;
        }
    }

    for (df, dr) in diagonals {
        let mut file = file_of(king_sq) as i8 + df;
        let mut rank = rank_of(king_sq) as i8 + dr;

        while (0..8).contains(&file) && (0..8).contains(&rank) {
            // scans outward until it finds first enemy slider
            let sq = square(file as u8, rank as u8);
            let sq_mask = bit(sq);

            // empty square
            if sq_mask & occupied == 0 {
                file += df;
                rank += dr;
                continue;
            }

            // First piece is a friendly
            if sq_mask & friends != 0 {
                break;
            }

            if sq_mask & enemies != 0 {
                if sq_mask & enemy_diagonals != 0 {
                    checkers |= sq_mask;
                }
                break;
            }
            break;
        }
    }

    checkers |= knight_attacks(king_sq) & board.pieces(enemy, PieceType::Knight);

    checkers |= pawn_attacks_from_square(king_sq, color) & board.pieces(enemy, PieceType::Pawn);

    let check_mask = match checkers.count_ones() {
        0 => !0u64,
        1 => {
            let checker_sq = checkers.trailing_zeros() as Square;
            let checker_bb = bit(checker_sq);

            let valid_slider_check = squares_aligned(king_sq, checker_sq)
                && checker_bb & (enemy_diagonals | enemy_straights) != 0;

            if valid_slider_check {
                BETWEEN[king_sq as usize][checker_sq as usize] | checker_bb
            } else {
                checker_bb
            }
        }
        _ => 0u64,
    };

    (checkers, check_mask)
}
