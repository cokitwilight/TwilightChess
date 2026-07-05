use eframe::epaint::color;
use eframe::wgpu::wgc::pipeline::ImplicitLayoutError::Pipeline;

use crate::bitboard::attacks::{all_knight_attacks, pawn_attacks};
use crate::bitboard::rays::{all_bishop_attacks, all_queen_attacks, all_rook_attacks};
use crate::bitboard::{
    Bitboard, NOT_FILE_A, NOT_FILE_H, Square, bit, file_mask, file_of, king_attacks, pop_lsb,
    rank_of, square,
};
use crate::board::Board;
use crate::eval::phase;
use crate::types::{Color, PieceType};

pub fn king_eval(board: &Board, phase: i32) -> i32 {
    king_eval_raw(board, Color::White, phase) - king_eval_raw(board, Color::Black, phase)
}

pub fn king_eval_raw(board: &Board, color: Color, phase: i32) -> i32 {
    let mut score = 0;

    let Some(king_sq) = pop_lsb(&mut board.pieces(color, PieceType::King)) else {
        panic!("No king in board.pieces in king_eval_raw!");
    };

    score += king_ring_safety(board, color, king_sq, phase);
    score += pawn_shield_score(board, color, king_sq, phase);
    score += open_file_bonus(board, color, king_sq, phase);
    score += open_diagonal_bonus(board, color, king_sq, phase);

    score
}

fn king_ring_safety(board: &Board, color: Color, king_sq: Square, phase: i32) -> i32 {
    let attacking_color = color.opposite();
    let mut ring = king_attacks(king_sq); // get the squares surrounding the king
    let occupancy = board.all_occupancy();
    let mut score = 0;

    let Some(enemy_king_sq) = pop_lsb(&mut board.pieces(attacking_color, PieceType::King)) else {
        panic!("No king in king ring safety!");
    };

    let pawn_attack = pawn_attacks(
        board.pieces(attacking_color, PieceType::Pawn),
        attacking_color,
    );
    let knight_attacks = all_knight_attacks(board.pieces(attacking_color, PieceType::Knight));
    let king_attacks = king_attacks(enemy_king_sq);
    let bishop_attacks =
        all_bishop_attacks(board.pieces(attacking_color, PieceType::Bishop), occupancy);
    let rook_attacks = all_rook_attacks(board.pieces(attacking_color, PieceType::Rook), occupancy);
    let queen_attacks =
        all_queen_attacks(board.pieces(attacking_color, PieceType::Queen), occupancy);

    let pawn_defenders = pawn_attacks(board.pieces(color, PieceType::Pawn), color);
    let knight_defenders = all_knight_attacks(board.pieces(color, PieceType::Knight));
    let bishop_defenders = all_bishop_attacks(board.pieces(color, PieceType::Bishop), occupancy);
    let rook_defenders = all_rook_attacks(board.pieces(color, PieceType::Rook), occupancy);
    let queen_defenders = all_queen_attacks(board.pieces(color, PieceType::Queen), occupancy);

    let mut attacker_count = 0;
    let mut defender_count = 1; // by default, the king is considered a defender of its own ring

    while let Some(ring_sq) = pop_lsb(&mut ring) {
        let sq_bb = bit(ring_sq);
        if pawn_attack & sq_bb != 0 {
            score -= 10; // penalize if the square is attacked by a pawn specifically
            attacker_count += 1;
        }
        if knight_attacks & sq_bb != 0 {
            score -= 3; // penalize if the square is attacked by a knight specifically
            attacker_count += 1;
        }
        if king_attacks & sq_bb != 0 {
            attacker_count += 1;
        }
        if bishop_attacks & sq_bb != 0 {
            attacker_count += 1;
        }
        if rook_attacks & sq_bb != 0 {
            attacker_count += 1;
        }
        if queen_attacks & sq_bb != 0 {
            attacker_count += 1;
        }

        if pawn_defenders & sq_bb != 0 {
            score += 5; // reward if the square is defended by a pawn specifically
            defender_count += 1;
        }
        if knight_defenders & sq_bb != 0 {
            defender_count += 1;
        }
        if bishop_defenders & sq_bb != 0 {
            defender_count += 1;
        }
        if rook_defenders & sq_bb != 0 {
            defender_count += 1;
        }
        if queen_defenders & sq_bb != 0 {
            score -= 5; // prefer a more aggressive queen vs defending the king
            defender_count += 1;
        }

        if attacker_count > defender_count {
            score -= 10 * (attacker_count - defender_count) as i32; // penalize if attackers outnumber defenders
        }

        if defender_count <= 1 {
            score -= 3; // penalize if the king is left with no defenders
        }

        if defender_count > attacker_count {
            score += 3; // slightly reward if defenders outnumber attackers. This will most likely be true for most squares so keep that in mind
        }

        attacker_count = 0;
        defender_count = 1; // reset for the next square
    }

    score
}

fn pawn_shield_score(board: &Board, color: Color, king_sq: Square, phase: i32) -> i32 {
    if phase < 10 {
        return 0; // no pawn shield evaluation in the endgame
    }

    if phase > 20 {
        match color {
            Color::White => {
                if king_sq == 4 {
                    return 0; // dont give a bonus if king hasn't castled in opening
                }
            }
            Color::Black => {
                if king_sq == 60 {
                    return 0;
                }
            }
        }
    }
    let mut score = 0;

    let pawns = board.pieces(color, PieceType::Pawn);

    let first_row_shield = generate_king_shield(color, king_sq);

    let mut total_pawn_shield = 0;

    let pawn_count = (pawns & first_row_shield).count_ones() as i32;

    total_pawn_shield += pawn_count;

    score += (pawn_count * 10) + ((3 - pawn_count) * -10);

    let second_row_shield = generate_king_shield_two_forward(color, king_sq);

    let second_pawn_count = (pawns & second_row_shield).count_ones() as i32;

    total_pawn_shield += second_pawn_count;

    score += second_pawn_count * 5;

    if total_pawn_shield >= 3 {
        score += 20;
    } else if total_pawn_shield == 0 {
        score -= 30;
    } else {
        score -= 10;
    }
    score
}

fn open_file_bonus(board: &Board, color: Color, king_sq: Square, phase: i32) -> i32 {
    if phase < 10 {
        return 0; // ignore in endgames
    }

    let mut score = 0;

    let mut king_file = file_of(king_sq);

    let enemy_sliders = board.pieces(color.opposite(), PieceType::Rook)
        | board.pieces(color.opposite(), PieceType::Queen);

    let friendly_pawns = board.pieces(color, PieceType::Pawn);

    for (offset, penalty) in [(-1, 7), (0, 15), (1, 7)] {
        let file = king_file as i8 + offset;

        if !(0..=7).contains(&file) {
            continue;
        }

        let file_bb = file_mask(file as u8);

        if file_bb & friendly_pawns == 0 {
            score -= penalty;
        }
        if file_bb & enemy_sliders != 0 {
            score -= 10;
        }
    }

    score
}

fn open_diagonal_bonus(board: &Board, color: Color, king_sq: Square, phase: i32) -> i32 {
    if phase < 10 {
        return 0;
    }

    let mut score = 0;

    let occupied = board.all_occupancy();

    let enemy_sliders = board.pieces(color.opposite(), PieceType::Bishop)
        | board.pieces(color.opposite(), PieceType::Queen);

    let friendly_pawns = board.pieces(color, PieceType::Pawn);

    for (df, dr) in [(1, 1), (1, -1), (-1, 1), (-1, -1)] {
        let mut file = file_of(king_sq) as i8 + df;
        let mut rank = rank_of(king_sq) as i8 + dr;

        let mut open_diagonal = true;

        while (0..8).contains(&file) && (0..8).contains(&rank) {
            let to = square(file as u8, rank as u8);
            let to_mask = bit(to);

            if occupied & to_mask != 0 {
                open_diagonal = false;
                if to_mask & enemy_sliders != 0 {
                    // break when you meet a bishop/queen else keep checking for potention weakness
                    score -= 10;
                    break;
                }
            }

            file += df;
            rank += dr;
        }
        if open_diagonal {
            score -= 10;
        }
    }

    score
}

// TODO: Do this later
// fn escape_score_bonus(board: &Board, color: Color, king_sq: Square, phase: i32) -> i32 {
//     if phase < 10 {
//         return 0;
//     }

//     let all_attacks = all_attacks(board, color.opposite());  // this might be too expensive for the value it adds

//     let king_ring = king_attacks(king_sq);

// }

fn generate_king_shield(color: Color, sq: Square) -> Bitboard {
    let b = bit(sq);

    let mut shield = 0u64;

    match color {
        Color::White => {
            // left shifts for white
            shield |= b << 8;
            shield |= (b & NOT_FILE_H) << 9;
            shield |= (b & NOT_FILE_A) << 7;
        }
        Color::Black => {
            // right shifts for black
            shield |= b >> 8;
            shield |= (b & NOT_FILE_H) >> 7;
            shield |= (b & NOT_FILE_A) >> 9;
        }
    }

    shield
}

fn generate_king_shield_two_forward(color: Color, sq: Square) -> Bitboard {
    let b = bit(sq);

    let mut shield = 0u64;

    match color {
        Color::White => {
            // left shifts for white
            shield |= b << 16;
            shield |= (b & NOT_FILE_H) << 17;
            shield |= (b & NOT_FILE_A) << 15;
        }
        Color::Black => {
            // right shifts for black
            shield |= b >> 16;
            shield |= (b & NOT_FILE_H) >> 15;
            shield |= (b & NOT_FILE_A) >> 17;
        }
    }

    shield
}
