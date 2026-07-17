use std::cmp::max;

use crate::bitboard::{
    Bitboard, Square, bishop_attacks, bit, file_of, king_attacks, knight_attacks,
    pawn_attacks_from_square, pop_lsb, rank_of, rook_attacks, square,
};
use crate::board::{Board, Move, MoveType};
use crate::types::{Color, PieceType};

pub fn see(board: &Board, mv: Move) -> i32 {
    let moving_piece = board
        .piece_at(mv.from)
        .expect("SEE called with no piece at mv.from");

    let target = mv.to; // this also includes en passant

    debug_assert_eq!(board.side_to_move(), moving_piece.color);

    let victim_square = match mv.kind {
        MoveType::EnPassant => square(file_of(mv.to), rank_of(mv.from)),
        _ => target,
    };

    let victim_value = match board.piece_at(victim_square) {
        Some(piece) => piece.kind.value(),
        None => {
            if mv.promotion.is_none() {
                return 0;
            }
            0
        }
    };

    let mut gains: Vec<i32> = Vec::with_capacity(32);

    let mut initial_gain = victim_value;

    if let Some(promotion) = mv.promotion {
        initial_gain += promotion.value() - PieceType::Pawn.value();
    }

    gains.push(initial_gain);

    let from_mask = bit(mv.from);

    let mut occupied = board.all_occupancy();
    let mut all_pieces = board.all_pieces();

    let victim_mask = bit(victim_square);

    if let Some(victim_piece) = board.piece_at(victim_square) {
        all_pieces[victim_piece.color.idx()][victim_piece.kind.idx()] &= !victim_mask;
    }

    if mv.kind == MoveType::EnPassant {
        occupied &= !victim_mask; // remove captured pawn
    }

    occupied &= !from_mask;
    occupied |= bit(target); // the capturing pawn lands on the empty ep target
    all_pieces[moving_piece.color.idx()][moving_piece.kind.idx()] &= !from_mask;

    let mut attackers = attackers_to_occ(&all_pieces, target, Color::White, occupied)
        | attackers_to_occ(&all_pieces, target, Color::Black, occupied);

    let mut side_to_move = board.side_to_move().opposite();

    let mut captured_value = mv.promotion.unwrap_or(moving_piece.kind).value();

    let mut all_diagonals = all_pieces[Color::White.idx()][PieceType::Bishop.idx()]
        | all_pieces[Color::Black.idx()][PieceType::Bishop.idx()]
        | all_pieces[Color::White.idx()][PieceType::Queen.idx()]
        | all_pieces[Color::Black.idx()][PieceType::Queen.idx()];

    let mut all_straights = all_pieces[Color::White.idx()][PieceType::Rook.idx()]
        | all_pieces[Color::Black.idx()][PieceType::Rook.idx()]
        | all_pieces[Color::White.idx()][PieceType::Queen.idx()]
        | all_pieces[Color::Black.idx()][PieceType::Queen.idx()];

    loop {
        // go from pawns -> knights -> bishops -> rooks -> queens -> king(add extra in check here)
        attackers &= occupied;

        let pawns = all_pieces[side_to_move.idx()][PieceType::Pawn.idx()];
        let knights = all_pieces[side_to_move.idx()][PieceType::Knight.idx()];
        let bishops = all_pieces[side_to_move.idx()][PieceType::Bishop.idx()];
        let rooks = all_pieces[side_to_move.idx()][PieceType::Rook.idx()];
        let queens = all_pieces[side_to_move.idx()][PieceType::Queen.idx()];
        let kings = all_pieces[side_to_move.idx()][PieceType::King.idx()];

        let side_occupancy = pawns | knights | bishops | rooks | queens | kings;

        let side_attackers = attackers & side_occupancy;

        if side_attackers == 0 {
            break;
        }

        let mut pawn_attackers = side_attackers & pawns;

        if pawn_attackers != 0 {
            let sq = pop_lsb(&mut pawn_attackers).expect("No pawn at sq in SEE!");

            let is_promotion_sq = match side_to_move {
                Color::White => rank_of(target) == 7,
                Color::Black => rank_of(target) == 0,
            };

            let attack_mask = bit(sq);

            let gain = if is_promotion_sq {
                captured_value + PieceType::Queen.value() - PieceType::Pawn.value()
            } else {
                captured_value
            };

            gains.push(gain);

            captured_value = if is_promotion_sq {
                PieceType::Queen.value()
            } else {
                PieceType::Pawn.value()
            };

            all_pieces[side_to_move.idx()][PieceType::Pawn.idx()] &= !attack_mask;
            occupied &= !attack_mask;
            attackers &= !attack_mask;

            attackers = add_xray_attacks(target, attackers, all_straights, all_diagonals, occupied);

            side_to_move = side_to_move.opposite();
            continue;
        }

        let mut knight_attackers = side_attackers & knights;

        if knight_attackers != 0 {
            let sq = pop_lsb(&mut knight_attackers).expect("No knight at sq in SEE!");

            let attacker_mask = bit(sq);

            gains.push(captured_value);

            captured_value = PieceType::Knight.value();

            all_pieces[side_to_move.idx()][PieceType::Knight.idx()] &= !attacker_mask;
            occupied &= !attacker_mask;
            attackers &= !attacker_mask;

            attackers = add_xray_attacks(target, attackers, all_straights, all_diagonals, occupied);

            side_to_move = side_to_move.opposite();
            continue;
        }

        let mut bishop_attackers = side_attackers & bishops;

        if bishop_attackers != 0 {
            let sq = pop_lsb(&mut bishop_attackers).expect("No bishop at sq in SEE!");

            let attacker_mask = bit(sq);

            gains.push(captured_value);

            captured_value = PieceType::Bishop.value();

            all_pieces[side_to_move.idx()][PieceType::Bishop.idx()] &= !attacker_mask;
            occupied &= !attacker_mask;
            attackers &= !attacker_mask;

            all_diagonals &= !attacker_mask;

            attackers = add_xray_attacks(target, attackers, all_straights, all_diagonals, occupied);

            side_to_move = side_to_move.opposite();
            continue;
        }

        let mut rook_attackers = side_attackers & rooks;

        if rook_attackers != 0 {
            let sq = pop_lsb(&mut rook_attackers).expect("No rook at sq in SEE!");

            let attacker_mask = bit(sq);

            gains.push(captured_value);

            captured_value = PieceType::Rook.value();

            all_pieces[side_to_move.idx()][PieceType::Rook.idx()] &= !attacker_mask;
            occupied &= !attacker_mask;
            attackers &= !attacker_mask;

            all_straights &= !attacker_mask;

            attackers = add_xray_attacks(target, attackers, all_straights, all_diagonals, occupied);

            side_to_move = side_to_move.opposite();
            continue;
        }

        let mut queen_attackers = side_attackers & queens;

        if queen_attackers != 0 {
            let sq = pop_lsb(&mut queen_attackers).expect("No queen at sq in SEE!");

            let attacker_mask = bit(sq);

            gains.push(captured_value);

            captured_value = PieceType::Queen.value();

            all_pieces[side_to_move.idx()][PieceType::Queen.idx()] &= !attacker_mask;
            occupied &= !attacker_mask;
            attackers &= !attacker_mask;

            all_diagonals &= !attacker_mask;
            all_straights &= !attacker_mask;

            attackers = add_xray_attacks(target, attackers, all_straights, all_diagonals, occupied);

            side_to_move = side_to_move.opposite();
            continue;
        }

        let mut king_attackers = side_attackers & kings;

        if king_attackers != 0 {
            let sq = pop_lsb(&mut king_attackers).expect("No king at sq in SEE!");

            let attacker_mask = bit(sq);

            let mut temp_attackers = attackers;
            let mut temp_occupied = occupied;

            temp_attackers &= !attacker_mask;
            temp_occupied &= !attacker_mask;

            temp_attackers = add_xray_attacks(
                target,
                temp_attackers,
                all_straights,
                all_diagonals,
                temp_occupied,
            );

            let enemy = side_to_move.opposite();

            let mut enemy_occupancy = all_pieces[enemy.idx()][PieceType::Pawn.idx()]
                | all_pieces[enemy.idx()][PieceType::Knight.idx()]
                | all_pieces[enemy.idx()][PieceType::Bishop.idx()]
                | all_pieces[enemy.idx()][PieceType::Rook.idx()]
                | all_pieces[enemy.idx()][PieceType::Queen.idx()]
                | all_pieces[enemy.idx()][PieceType::King.idx()];

            enemy_occupancy &= temp_occupied;

            temp_attackers &= enemy_occupancy;

            if temp_attackers != 0 {
                // opened up check
                break;
            }

            gains.push(captured_value);

            // should be the last capture anyways.
            // since we break none of this is needed.
            break;
        }

        break;
    }

    let mut value = 0;
    for i in (1..gains.len()).rev() {
        value = max(0, gains[i] - value);
    }
    let see_score = gains[0] - value;

    see_score
}

fn attackers_to_occ(
    all_pieces: &[[Bitboard; 6]; 2],
    target: Square,
    by: Color,
    occupied: Bitboard,
) -> Bitboard {
    let pawns = all_pieces[by.idx()][PieceType::Pawn.idx()];
    let knights = all_pieces[by.idx()][PieceType::Knight.idx()];
    let bishops = all_pieces[by.idx()][PieceType::Bishop.idx()];
    let rooks = all_pieces[by.idx()][PieceType::Rook.idx()];
    let queens = all_pieces[by.idx()][PieceType::Queen.idx()];
    let king = all_pieces[by.idx()][PieceType::King.idx()];

    let pawn_attackers = pawn_attacks_from_square(target, by.opposite()) & pawns;

    let knight_attackers = knight_attacks(target) & knights;

    let bishop_attackers = bishop_attacks(target, occupied) & (bishops | queens);

    let rook_attackers = rook_attacks(target, occupied) & (rooks | queens);

    let king_attackers = king_attacks(target) & king;

    pawn_attackers | knight_attackers | bishop_attackers | rook_attackers | king_attackers
}

fn add_xray_attacks(
    target: Square,
    attackers: Bitboard,
    straights: Bitboard,
    diagonals: Bitboard,
    occupied: Bitboard,
) -> Bitboard {
    let attackers = attackers
        | (bishop_attacks(target, occupied) & diagonals)
        | (rook_attacks(target, occupied) & straights);

    attackers & occupied
}
