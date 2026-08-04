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

    // technically we only call on legal moves but this is a good safegaurd regardless
    if see_in_check(&all_pieces, occupied, side_to_move) {
        // if the side to move is in check, then we can't capture anything
        return 0;
    }

    'see_loop: loop {
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

        while let Some(sq) = pop_lsb(&mut pawn_attackers) {
            let attack_mask = bit(sq);

            occupied &= !attack_mask;
            all_pieces[side_to_move.idx()][PieceType::Pawn.idx()] &= !attack_mask;

            if see_in_check(&all_pieces, occupied, side_to_move) {
                // revert changes
                occupied |= attack_mask;
                all_pieces[side_to_move.idx()][PieceType::Pawn.idx()] |= attack_mask;

                // look for other legal pawns
                continue;
            }
            // pawn is legal

            attackers &= !attack_mask;

            let is_promotion_sq = match side_to_move {
                Color::White => rank_of(target) == 7,
                Color::Black => rank_of(target) == 0,
            };

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
            attackers = add_xray_attacks(target, attackers, all_straights, all_diagonals, occupied);

            side_to_move = side_to_move.opposite();
            continue 'see_loop;
        }

        let mut knight_attackers = side_attackers & knights;

        while let Some(sq) = pop_lsb(&mut knight_attackers) {
            let attacker_mask = bit(sq);

            all_pieces[side_to_move.idx()][PieceType::Knight.idx()] &= !attacker_mask;
            occupied &= !attacker_mask;

            if see_in_check(&all_pieces, occupied, side_to_move) {
                // revert changes
                all_pieces[side_to_move.idx()][PieceType::Knight.idx()] |= attacker_mask;
                occupied |= attacker_mask;

                continue;
            }
            attackers &= !attacker_mask;

            gains.push(captured_value);

            captured_value = PieceType::Knight.value();

            attackers = add_xray_attacks(target, attackers, all_straights, all_diagonals, occupied);

            side_to_move = side_to_move.opposite();
            continue 'see_loop;
        }

        let mut bishop_attackers = side_attackers & bishops;

        while let Some(sq) = pop_lsb(&mut bishop_attackers) {
            let attacker_mask = bit(sq);

            all_pieces[side_to_move.idx()][PieceType::Bishop.idx()] &= !attacker_mask;
            occupied &= !attacker_mask;

            if see_in_check(&all_pieces, occupied, side_to_move) {
                // revert changes
                all_pieces[side_to_move.idx()][PieceType::Bishop.idx()] |= attacker_mask;
                occupied |= attacker_mask;

                continue;
            }
            // legal bishop capture

            gains.push(captured_value);

            captured_value = PieceType::Bishop.value();

            attackers &= !attacker_mask;

            all_diagonals &= !attacker_mask;

            attackers = add_xray_attacks(target, attackers, all_straights, all_diagonals, occupied);

            side_to_move = side_to_move.opposite();
            continue 'see_loop;
        }

        let mut rook_attackers = side_attackers & rooks;

        while let Some(sq) = pop_lsb(&mut rook_attackers) {
            let attacker_mask = bit(sq);

            all_pieces[side_to_move.idx()][PieceType::Rook.idx()] &= !attacker_mask;
            occupied &= !attacker_mask;

            if see_in_check(&all_pieces, occupied, side_to_move) {
                // revert changes
                all_pieces[side_to_move.idx()][PieceType::Rook.idx()] |= attacker_mask;
                occupied |= attacker_mask;

                continue;
            }

            // legal rook capture

            gains.push(captured_value);

            captured_value = PieceType::Rook.value();

            attackers &= !attacker_mask;

            all_straights &= !attacker_mask;

            attackers = add_xray_attacks(target, attackers, all_straights, all_diagonals, occupied);

            side_to_move = side_to_move.opposite();
            continue 'see_loop;
        }

        let mut queen_attackers = side_attackers & queens;

        while let Some(sq) = pop_lsb(&mut queen_attackers) {
            let attacker_mask = bit(sq);

            all_pieces[side_to_move.idx()][PieceType::Queen.idx()] &= !attacker_mask;
            occupied &= !attacker_mask;

            if see_in_check(&all_pieces, occupied, side_to_move) {
                all_pieces[side_to_move.idx()][PieceType::Rook.idx()] |= attacker_mask;
                occupied |= attacker_mask;

                continue;
            }

            gains.push(captured_value);

            captured_value = PieceType::Queen.value();

            attackers &= !attacker_mask;

            all_diagonals &= !attacker_mask;
            all_straights &= !attacker_mask;

            attackers = add_xray_attacks(target, attackers, all_straights, all_diagonals, occupied);

            side_to_move = side_to_move.opposite();
            continue 'see_loop;
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
    gains[0] - value
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

fn see_in_check(pieces: &[[Bitboard; 6]; 2], occupied: Bitboard, color: Color) -> bool {
    let king = pieces[color.idx()][PieceType::King.idx()];
    debug_assert!(king != 0, "No king found for {:?}", color);

    debug_assert!(
        king.count_ones() == 1,
        "Expected exactly one king for {:?}, found {}",
        color,
        king.count_ones()
    );

    let king_sq = king.trailing_zeros() as Square;
    square_attacked(pieces, occupied, king_sq, color.opposite())
}

fn square_attacked(pieces: &[[Bitboard; 6]; 2], occupied: Bitboard, sq: Square, by: Color) -> bool {
    // only check sliders for now
    // let pawns = pieces[by.idx()][PieceType::Pawn.idx()];
    // let knights = pieces[by.idx()][PieceType::Knight.idx()];
    let bishops = pieces[by.idx()][PieceType::Bishop.idx()];
    let rooks = pieces[by.idx()][PieceType::Rook.idx()];
    let queens = pieces[by.idx()][PieceType::Queen.idx()];
    let king = pieces[by.idx()][PieceType::King.idx()];

    // let pawn_attackers = match by {
    //     Color::White => pawn_attacks_from_square(sq, Color::Black) & pawns,
    //     Color::Black => pawn_attacks_from_square(sq, Color::White) & pawns,
    // };

    // if pawn_attackers != 0 {
    //     return true;
    // }

    // // -------------------------
    // // Knights
    // // -------------------------
    // if knight_attacks(sq) & knights != 0 {
    //     return true;
    // }

    // -------------------------
    // Kings
    // -------------------------
    if king_attacks(sq) & king != 0 {
        return true;
    }

    // -------------------------
    // Bishops / Queens
    // -------------------------
    let diagonal_attackers = bishops | queens;

    if bishop_attacks(sq, occupied) & diagonal_attackers != 0 {
        return true;
    }
    // -------------------------
    // Rooks / Queens
    // -------------------------
    let straight_attackers = rooks | queens;

    if rook_attacks(sq, occupied) & straight_attackers != 0 {
        return true;
    }

    false
}

// regular see knight can take. Target square is D5
// rn1qkb1r/pp2pppp/2p1bn2/3p4/4P3/2N2B2/PPPP1PPP/R1BQK1NR w KQkq - 0 1

// legal see knight cant take. Target square is D5
// rn1qk2r/pp3ppp/2p1bn2/3p4/1b1PP3/2N2B2/PPP2PPP/R1BQK1NR w KQkq - 0 1

#[cfg(test)]
mod see_tests {
    use super::*;

    /// Confirms that `capture_move` is legal, runs SEE, and compares its result.
    #[track_caller]
    fn assert_legal_see(fen: &str, capture_move: Move, expected_see: i32) {
        let mut board = Board::from_fen(fen).unwrap();

        assert_eq!(
            board.side_to_move(),
            Color::White,
            "These SEE tests expect White to move.\nFEN: {fen}"
        );

        let moves = board.all_legal_moves();

        let valid_move = moves.iter().any(|mv| *mv == capture_move);

        assert!(
            valid_move,
            "{capture_move:?} is not a valid move!\nFEN: {fen}"
        );

        let actual_see = see(&board, capture_move);

        println!(
            "Move: {capture_move:?}, Actual SEE: {actual_see}, \
             Expected SEE: {expected_see}\nFEN: {fen}"
        );

        assert_eq!(
            actual_see, expected_see,
            "Incorrect SEE for {capture_move:?}\nFEN: {fen}"
        );
    }

    /// Used for captures that are pseudo-legal but illegal because they expose
    /// the moving side's king.
    #[track_caller]
    fn assert_move_is_illegal(fen: &str, capture_move: Move) {
        let mut board = Board::from_fen(fen).unwrap();

        assert_eq!(board.side_to_move(), Color::White);

        let moves = board.all_legal_moves();

        let valid_move = moves.iter().any(|mv| *mv == capture_move);

        assert!(
            !valid_move,
            "{capture_move:?} was unexpectedly generated as legal!\nFEN: {fen}"
        );
    }

    mod regular_see {
        use super::*;

        #[test]
        fn undefended_pawn_capture_wins_pawn() {
            /*
                8  . . . . k . . .
                7  . . . . . . . .
                6  . . . . . . . .
                5  . . . p . . . .
                4  . . . . P . . .
                3  . . . . . . . .
                2  . . . . . . . .
                1  . . . . K . . .

                   a b c d e f g h

                1. exd5

                The pawn on d5 is undefended, so White wins one pawn.
            */
            let fen = "4k3/8/8/3p4/4P3/8/8/4K3 w - - 0 1";

            let capture_move = Move {
                from: 28, // e4
                to: 35,   // d5
                kind: MoveType::Capture,
                promotion: None,
            };

            let expected_see = PieceType::Pawn.value();

            assert_legal_see(fen, capture_move, expected_see);
        }

        #[test]
        fn pawn_capture_followed_by_equal_pawn_recapture_is_zero() {
            /*
                8  . . . . k . . .
                7  . . . . . . . .
                6  . . p . . . . .
                5  . . . p . . . .
                4  . . . . P . . .
                3  . . . . . . . .
                2  . . . . . . . .
                1  . . . . K . . .

                1. exd5 cxd5

                White captures a pawn and then loses the capturing pawn.
            */
            let fen = "4k3/8/2p5/3p4/4P3/8/8/4K3 w - - 0 1";

            let capture_move = Move {
                from: 28, // e4
                to: 35,   // d5
                kind: MoveType::Capture,
                promotion: None,
            };

            let expected_see = 0;

            assert_legal_see(fen, capture_move, expected_see);
        }

        #[test]
        fn queen_captures_pawn_and_is_lost_to_pawn() {
            /*
                8  . . . . k . . .
                7  . . . . . . . .
                6  . . p . . . . .
                5  . . . p . . . .
                4  . . . . Q . . .
                3  . . . . . . . .
                2  . . . . . . . .
                1  . . . . K . . .

                1. Qxd5 cxd5

                White gains a pawn but loses the queen.
            */
            let fen = "4k3/8/2p5/3p4/4Q3/8/8/4K3 w - - 0 1";

            let capture_move = Move {
                from: 28, // e4
                to: 35,   // d5
                kind: MoveType::Capture,
                promotion: None,
            };

            let expected_see = PieceType::Pawn.value() - PieceType::Queen.value();

            assert_legal_see(fen, capture_move, expected_see);
        }
    }

    mod legal_see {
        use super::*;

        #[test]
        fn pinned_knight_cannot_recapture() {
            /*
                8  . . . . . k . .
                7  . . . . . . . .
                6  . . . . . n . .
                5  . . . p . . . .
                4  . . . . P . . .
                3  . . . . . . . .
                2  . . . . . . . .
                1  . . . . K R . .

                The black knight on f6 geometrically attacks d5.

                However, it is pinned to the black king on f8 by the
                white rook on f1.

                1. exd5

                Black cannot legally play Nxd5 because moving the knight
                would expose the king to the rook.
            */
            let fen = "5k2/8/5n2/3p4/4P3/8/8/4KR2 w - - 0 1";

            let capture_move = Move {
                from: 28, // e4
                to: 35,   // d5
                kind: MoveType::Capture,
                promotion: None,
            };

            let expected_see = PieceType::Pawn.value();

            assert_legal_see(fen, capture_move, expected_see);
        }

        #[test]
        fn skips_pinned_knight_and_finds_second_legal_knight() {
            /*
                8  . k . . . . . .
                7  . . . . . . . .
                6  . n . . . n . .
                5  . . . p . . . .
                4  . . . . P . . .
                3  . . . . . . . .
                2  . . . . . . . .
                1  . R . . K . . .

                Both black knights attack d5:

                - b6 knight: pinned to the king on b8 by the rook on b1
                - f6 knight: legal attacker

                Since b6 has the lower square index, a pop_lsb-based SEE
                encounters the pinned knight first.

                Legal SEE must reject Nb6xd5 and continue looking until it
                finds Nf6xd5.

                1. exd5 Nfxd5

                The final SEE is zero.
            */
            let fen = "1k6/8/1n3n2/3p4/4P3/8/8/1R2K3 w - - 0 1";

            let capture_move = Move {
                from: 28, // e4
                to: 35,   // d5
                kind: MoveType::Capture,
                promotion: None,
            };

            let expected_see = 0;

            assert_legal_see(fen, capture_move, expected_see);
        }

        #[test]
        fn capturing_the_piece_giving_check_is_legal() {
            /*
                8  . . . . k . . .
                7  . . . . . . . .
                6  . . . . . . . .
                5  . . . . . . . .
                4  . . . . . . . .
                3  . . . . . n . .
                2  . . . . Q . . .
                1  . . . . K . . .

                The knight on f3 is checking the white king on e1.

                1. Qxf3

                This tests that legal SEE removes the captured knight from
                the temporary knight bitboard before checking king safety.
            */
            let fen = "4k3/8/8/8/8/5n2/4Q3/4K3 w - - 0 1";

            let capture_move = Move {
                from: 12, // e2
                to: 21,   // f3
                kind: MoveType::Capture,
                promotion: None,
            };

            let expected_see = PieceType::Knight.value();

            assert_legal_see(fen, capture_move, expected_see);
        }
    }

    mod xray_attacks {
        use super::*;

        #[test]
        fn moving_queen_reveals_bishop_attacker() {
            /*
                8  . . . . . . . k
                7  . . . . . . . .
                6  . . . . . n . .
                5  . . . p . . . .
                4  . . . . Q . . .
                3  . . . . . . . .
                2  . . . . . . B .
                1  . . . . . . . K

                Initially, the queen on e4 blocks the bishop on g2:

                    g2 - f3 - e4 - d5

                Exchange:

                    1. Qxd5 Nxd5
                    2. Bxd5

                Material result:

                    + pawn
                    - queen
                    + knight

                The bishop itself is not added to the score because it survives.
            */
            let fen = "7k/8/5n2/3p4/4Q3/8/6B1/7K w - - 0 1";

            let capture_move = Move {
                from: 28, // e4
                to: 35,   // d5
                kind: MoveType::Capture,
                promotion: None,
            };

            let expected_see =
                PieceType::Pawn.value() - PieceType::Queen.value() + PieceType::Knight.value();

            assert_legal_see(fen, capture_move, expected_see);
        }
    }

    mod en_passant {
        use super::*;

        #[test]
        fn undefended_en_passant_wins_pawn() {
            /*
                8  . . . . k . . .
                7  . . . . . . . .
                6  . . . . . . . .
                5  . . . p P . . .
                4  . . . . . . . .
                3  . . . . . . . .
                2  . . . . . . . .
                1  . . . . K . . .

                En-passant target: d6

                1. exd6 e.p.

                The captured pawn is on d5, not d6.
            */
            let fen = "4k3/8/8/3pP3/8/8/8/4K3 w - d6 0 1";

            let capture_move = Move {
                from: 36, // e5
                to: 43,   // d6
                kind: MoveType::EnPassant,
                promotion: None,
            };

            let expected_see = PieceType::Pawn.value();

            assert_legal_see(fen, capture_move, expected_see);
        }

        #[test]
        fn en_passant_pawn_is_recaptured() {
            /*
                8  . . . . k . . .
                7  . . p . . . . .
                6  . . . . . . . .
                5  . . . p P . . .
                4  . . . . . . . .
                3  . . . . . . . .
                2  . . . . . . . .
                1  . . . . K . . .

                En-passant target: d6

                1. exd6 e.p. cxd6

                White gains the d5 pawn and then loses the e5 pawn.
            */
            let fen = "4k3/2p5/8/3pP3/8/8/8/4K3 w - d6 0 1";

            let capture_move = Move {
                from: 36, // e5
                to: 43,   // d6
                kind: MoveType::EnPassant,
                promotion: None,
            };

            let expected_see = 0;

            assert_legal_see(fen, capture_move, expected_see);
        }

        #[test]
        fn en_passant_that_exposes_rook_check_is_illegal() {
            /*
                8  k . . . . . . .
                7  . . . . . . . .
                6  . . . . . . . .
                5  . . . . K P p r
                4  . . . . . . . .
                3  . . . . . . . .
                2  . . . . . . . .
                1  . . . . . . . .

                En-passant target: g6

                Attempted move:

                    1. fxg6 e.p.??

                Before the move, f5 and g5 block the rook on h5.

                En passant simultaneously removes:

                    - White pawn from f5
                    - Black pawn from g5

                The rook would then attack the white king on e5:

                    h5 -> g5 -> f5 -> e5

                Therefore, the en-passant capture is illegal.
            */
            let fen = "k7/8/8/4KPpr/8/8/8/8 w - g6 0 1";

            let capture_move = Move {
                from: 37, // f5
                to: 46,   // g6
                kind: MoveType::EnPassant,
                promotion: None,
            };

            assert_move_is_illegal(fen, capture_move);
        }
    }

    mod king_capture_edge_cases {
        use super::*;

        #[test]
        fn king_cannot_recapture_onto_defended_target() {
            /*
                8  . . . . k . . .
                7  . . . r . . . .
                6  . . . . . . . .
                5  . . . . . . . .
                4  . . . . . . B .
                3  . . . . . . . .
                2  . . . . . . . .
                1  K . . R . . . .

                1. Rxd7

                The black king on e8 geometrically attacks d7, but d7 is
                defended by the bishop on g4:

                    g4 - f5 - e6 - d7

                Therefore, 1...Kxd7 is illegal and must not be included in SEE.
            */
            let fen = "4k3/3r4/8/8/6B1/8/8/K2R4 w - - 0 1";

            let capture_move = Move {
                from: 3, // d1
                to: 51,  // d7
                kind: MoveType::Capture,
                promotion: None,
            };

            let expected_see = PieceType::Rook.value();

            assert_legal_see(fen, capture_move, expected_see);
        }
    }

    mod promotion_edge_cases {
        use super::*;

        #[test]
        fn promotion_capture_includes_promotion_material_gain() {
            /*
                8  k . . . . . . r
                7  . . . . . . P .
                6  . . . . . . . .
                5  . . . . . . . .
                4  . . . . . . . .
                3  . . . . . . . .
                2  . . . . . . . .
                1  K . . . . . . .

                1. gxh8=Q+

                White gains:

                    rook value
                    + queen value
                    - pawn value

                The pawn is replaced by a queen, so only the increase from
                pawn to queen is added as promotion material.
            */
            let fen = "k6r/6P1/8/8/8/8/8/K7 w - - 0 1";

            let capture_move = Move {
                from: 54, // g7
                to: 63,   // h8
                kind: MoveType::Capture,
                promotion: Some(PieceType::Queen),
            };

            let expected_see =
                PieceType::Rook.value() + PieceType::Queen.value() - PieceType::Pawn.value();

            assert_legal_see(fen, capture_move, expected_see);
        }
    }
}
