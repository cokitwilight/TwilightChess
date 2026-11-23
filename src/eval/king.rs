use crate::bitboard::lookup::BETWEEN;
use crate::bitboard::{
    Bitboard, FILE_A, FILE_B, FILE_C, FILE_D, FILE_E, FILE_F, FILE_G, FILE_H, RANK_1, RANK_2,
    RANK_3, RANK_6, RANK_7, RANK_8, Square, bishop_attacks, bit, file_mask, file_of, king_attacks,
    knight_attacks, pop_lsb, rook_attacks,
};
use crate::board::Board;
use crate::eval::EvalInfo;
use crate::eval::eval::MAX_DANGER;
use crate::eval::lookup::{KING_DANGER_TABLE, KING_PAWN_SHIELD, KING_PAWN_TWO_SHIELD, RAY_MASKS};
use crate::eval::scale_by_phase;
use crate::types::{Color, PieceType};

// Indexed by the defending king's file.
const KING_FLANK: [Bitboard; 8] = [
    FILE_A | FILE_B | FILE_C,
    FILE_A | FILE_B | FILE_C | FILE_D,
    FILE_A | FILE_B | FILE_C | FILE_D,
    FILE_C | FILE_D | FILE_E | FILE_F,
    FILE_C | FILE_D | FILE_E | FILE_F,
    FILE_E | FILE_F | FILE_G | FILE_H,
    FILE_E | FILE_F | FILE_G | FILE_H,
    FILE_F | FILE_G | FILE_H,
];

pub(super) fn attack_zone(color: Color, square: Square) -> Bitboard {
    let mut zone = king_attacks(square);
    match color {
        Color::White if bit(square) & RANK_1 != 0 => zone |= zone << 8,
        Color::Black if bit(square) & RANK_8 != 0 => zone |= zone >> 8,
        _ => {}
    }
    match file_of(square) {
        0 => zone | (zone << 1),
        7 => zone | (zone >> 1),
        _ => zone,
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct KingAttackers {
    pub zone: Bitboard,
    pub pieces: i32,
    pub weight: i32,
    pub adjacent_contacts: i32,
}

impl KingAttackers {
    // Called once per attacking piece while its individual attack mask is available.
    pub(super) fn record(&mut self, piece: PieceType, attacks: Bitboard, neighbors: Bitboard) {
        if attacks & self.zone == 0 {
            return;
        }
        self.pieces += 1;
        self.weight += match piece {
            PieceType::Knight => 77,
            PieceType::Bishop => 55,
            PieceType::Rook => 44,
            PieceType::Queen => 10,
            _ => unreachable!("only non-pawn, non-king attackers are recorded"),
        };
        self.adjacent_contacts += (attacks & neighbors).count_ones() as i32;
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct KingPressure {
    pub flank_contacts: i32,
    pub weak_squares: i32,
    pub coordination: i32,
    pub safe_check_cp: i32,
    pub unsafe_checks: i32,
}

impl KingPressure {
    pub(super) fn index_bonus(self) -> i32 {
        self.coordination + 3 * self.weak_squares + 2 * self.unsafe_checks
    }

    pub(super) fn flank_cp(self) -> i32 {
        6 * self.flank_contacts
    }
}

// Score checking destinations on the static attack maps. Multiple safe destinations
// for the same piece type earn one bonus. Unsafe squares are shared across types.
fn checking_threats(board: &Board, defender: Color, info: &EvalInfo, weak: Bitboard) -> (i32, i32) {
    let enemy = defender.opposite();
    let king = info.king_square(defender);
    let occupancy = board.all_occupancy() ^ board.pieces(defender, PieceType::Queen);
    let straight = rook_attacks(king, occupancy);
    let diagonal = bishop_attacks(king, occupancy);
    let available = !board.occupancy_of(enemy);
    let safe = !info.all_attacks(defender) | (weak & info.king_double_attacks[enemy.idx()]);
    let queen_checks = (straight | diagonal) & !info.attacks(defender, PieceType::Queen);
    let mut safe_cp = 0;
    let mut unsafe_squares = 0;
    for (piece, checks, bonus) in [
        (PieceType::Queen, queen_checks, 90),
        (PieceType::Rook, straight, 90),
        (PieceType::Bishop, diagonal, 45),
        (PieceType::Knight, knight_attacks(king), 90),
    ] {
        let candidates = checks & info.attacks(enemy, piece) & available;
        if candidates & safe != 0 {
            safe_cp += bonus;
        } else if piece != PieceType::Queen {
            unsafe_squares |= candidates;
        }
    }
    unsafe_squares &= !info.attacks(defender, PieceType::Pawn);
    (safe_cp, unsafe_squares.count_ones() as i32)
}

pub(super) fn king_pressure(board: &Board, defender: Color, info: &EvalInfo) -> KingPressure {
    let enemy = defender.opposite();
    let attackers = info.king_attackers[defender.idx()];
    let enemy_attacks = info.all_attacks(enemy);
    let camp = match defender {
        Color::White => !(RANK_6 | RANK_7 | RANK_8),
        Color::Black => !(RANK_1 | RANK_2 | RANK_3),
    };
    let flank = KING_FLANK[file_of(info.king_square(defender)) as usize] & camp & enemy_attacks;
    let weak = enemy_attacks
        & !info.king_double_attacks[defender.idx()]
        & (!info.all_attacks(defender)
            | info.attacks(defender, PieceType::King)
            | info.attacks(defender, PieceType::Queen));
    let (safe_check_cp, unsafe_checks) = checking_threats(board, defender, info, weak);
    let pawn_contacts = (attackers.zone & info.attacks(enemy, PieceType::Pawn)).count_ones() as i32;
    KingPressure {
        flank_contacts: (flank.count_ones()
            + (flank & info.king_double_attacks[enemy.idx()]).count_ones())
            as i32,
        weak_squares: (weak & attackers.zone).count_ones() as i32,
        // Convert SF10's relative piece weights to the engine's smaller danger index.
        coordination: (attackers.pieces + pawn_contacts) * attackers.weight / 64
            + attackers.adjacent_contacts,
        safe_check_cp,
        unsafe_checks,
    }
}

pub fn king_eval(board: &Board, info: &EvalInfo) -> (i32, i32) {
    let (white_mg, white_eg) = king_eval_danger_raw(board, Color::White, info);
    let (black_mg, black_eg) = king_eval_danger_raw(board, Color::Black, info);
    (white_mg - black_mg, white_eg - black_eg)
}

pub(super) fn base_mg_danger(
    ring: i32,
    shield: i32,
    files: i32,
    diagonals: i32,
    escapes: i32,
    defenders: i32,
) -> i32 {
    // Files and defenders share one rounding step. Round half points upward.
    20 + 3 * ring + shield + diagonals + escapes + (3 * files + defenders + 1).div_euclid(2)
}

pub fn king_eval_danger_raw(board: &Board, color: Color, info: &EvalInfo) -> (i32, i32) {
    if info.phase() <= 4 {
        return (0, 0);
    }
    let king_sq = info.king_square(color);
    let pressure = king_pressure(board, color, info);
    let ring = info.king_attack_weight(color);
    let shield = pawn_shield_danger_score(board, color, king_sq, info);
    let files = open_file_danger_bonus(board, color, king_sq, info);
    let diagonals = open_diagonal_danger_bonus(board, color, king_sq, info);
    let escapes = escape_score_danger_bonus(board, color, info);
    let defenders = defender_danger_bonus(board, color, info);
    let mut mg = base_mg_danger(ring, shield.0, files.0, diagonals.0, escapes.0, defenders.0)
        + pressure.index_bonus();
    let mut eg = ring + shield.1 + files.1 + diagonals.1 + escapes.1 + defenders.1;

    mg = -KING_DANGER_TABLE[mg.clamp(0, MAX_DANGER as i32) as usize];
    mg -= pressure.flank_cp() + pressure.safe_check_cp;
    eg = -KING_DANGER_TABLE[eg.clamp(0, MAX_DANGER as i32) as usize];
    mg = scale_by_phase(mg, info.phase(), 4, 12);
    eg = scale_by_phase(eg, info.phase(), 4, 12);
    (mg, eg)
}

// range: -17 <-> 8
pub(super) fn pawn_shield_danger_score(
    board: &Board,
    color: Color,
    king_sq: Square,
    info: &EvalInfo,
) -> (i32, i32) {
    if info.phase() < 10 {
        return (0, 0); // no pawn shield evaluation in the endgame
    }

    if info.phase() > 20 && board.has_castling_rights() {
        match color {
            Color::White => {
                if king_sq == 4 {
                    return (0, 0); // dont give a bonus if king hasn't castled in opening
                }
            }
            Color::Black => {
                if king_sq == 60 {
                    return (0, 0);
                }
            }
        }
    }
    let mut mg = 0;
    let mut eg = mg;

    let pawns = board.pieces(color, PieceType::Pawn);

    let mut first_row_shield = KING_PAWN_SHIELD[color.idx()][king_sq as usize];

    match color {
        Color::White => {
            if king_sq < 56 {
                let front_mask = bit(king_sq + 8);

                let enemy_pawn = board.pieces(color.opposite(), PieceType::Pawn) & front_mask;

                first_row_shield |= enemy_pawn;
            }
        }
        Color::Black => {
            if king_sq > 7 {
                let front_mask = bit(king_sq - 8);

                let enemy_pawn = board.pieces(color.opposite(), PieceType::Pawn) & front_mask;

                first_row_shield |= enemy_pawn;
            }
        }
    }

    let mut total_pawn_shield = 0;

    let pawn_count = (pawns & first_row_shield).count_ones() as i32;

    total_pawn_shield += pawn_count;

    mg -= pawn_count * 2;
    eg -= pawn_count * 2;

    let second_row_shield = KING_PAWN_TWO_SHIELD[color.idx()][king_sq as usize];

    let second_pawn_count = (pawns & second_row_shield).count_ones() as i32;

    total_pawn_shield += second_pawn_count;

    mg -= second_pawn_count;
    eg -= second_pawn_count;

    if total_pawn_shield >= 3 {
        mg -= 8;
        eg -= 8;
    } else if total_pawn_shield == 0 {
        mg += 8;
        eg += 8;
    } else {
        mg += 1;
        eg += 1;
    }
    (mg, eg)
}

// range: 0 <-> 25
pub(super) fn open_file_danger_bonus(
    board: &Board,
    color: Color,
    king_sq: Square,
    info: &EvalInfo,
) -> (i32, i32) {
    if info.phase() < 10 {
        return (0, 0); // ignore in endgames
    }

    let mut mg = 0;
    let mut eg = mg;

    let king_file = file_of(king_sq);

    let enemy_sliders = board.pieces(color.opposite(), PieceType::Rook)
        | board.pieces(color.opposite(), PieceType::Queen);

    let friendly_pawns = board.pieces(color, PieceType::Pawn);

    for (offset, mg_penalty, eg_penalty) in [(-1, 1, 1), (0, 2, 2), (1, 1, 1)] {
        let file = king_file as i8 + offset;

        if !(0..=7).contains(&file) {
            continue;
        }

        let file_bb = file_mask(file as u8);

        if file_bb & friendly_pawns == 0 {
            mg += mg_penalty;
            eg += eg_penalty;
        }
        if file_bb & enemy_sliders != 0 {
            mg += 7;
            eg += 7;
        }
    }

    (mg, eg)
}

// range: 0 <-> 24. Note in just the forward two it would likely be 0-12/14
pub(super) fn open_diagonal_danger_bonus(
    board: &Board,
    color: Color,
    king_sq: Square,
    info: &EvalInfo,
) -> (i32, i32) {
    if info.phase() < 10 {
        return (0, 0);
    }

    let mut mg = 0;
    let mut eg = mg;

    let enemy_sliders = board.pieces(color.opposite(), PieceType::Bishop)
        | board.pieces(color.opposite(), PieceType::Queen);

    for (df, dr) in [(1, 1), (1, -1), (-1, 1), (-1, -1)] {
        let ray = RAY_MASKS[king_sq as usize][(df + 1) as usize][(dr + 1) as usize];
        let ray_length = ray.count_ones() as i32;

        if ray_length <= 1 {
            // lazily assume there is some danger but avoid computing logic on single square rays
            mg += 1;
            eg += 1;
            continue;
        }
        let mut sliders_on_ray = ray & enemy_sliders;

        if ray & enemy_sliders != 0 {
            mg += 6 * sliders_on_ray.count_ones() as i32 * sliders_on_ray.count_ones() as i32; // slider pointing at king
            eg += 6 * sliders_on_ray.count_ones() as i32 * sliders_on_ray.count_ones() as i32;

            let mut between = 0u64;
            while let Some(slider_sq) = pop_lsb(&mut sliders_on_ray) {
                between |= BETWEEN[slider_sq as usize][king_sq as usize];
            }

            if between & board.all_occupancy() != 0 {
                let pawns = (between & board.pieces(color, PieceType::Pawn)).count_ones() as i32; // MAYBE INCLUDE ENEMY PAWNS HERE
                if pawns == 0 {
                    mg += 4;
                    eg += 4;
                } else {
                    mg -= 4 * pawns * pawns;
                    eg -= 4 * pawns * pawns;
                }

                let others =
                    between & board.occupancy_of(color) & !board.pieces(color, PieceType::Pawn);

                mg -= others.count_ones() as i32;
                eg -= others.count_ones() as i32;
            } else {
                mg += 2; // direct check
                eg += 2;
            }
            continue;
        }

        let pawns_on_ray = ray & board.pieces(color, PieceType::Pawn);

        if pawns_on_ray == 0 {
            mg += ray_length; // this should be relative to the length of the ray.
            eg += ray_length;

            let others = ray & board.occupancy_of(color);

            mg -= others.count_ones() as i32;
            eg -= others.count_ones() as i32;
        } else {
            let pawns = pawns_on_ray.count_ones() as i32;
            mg -= pawns * pawns;
            eg -= pawns * pawns;
        }
    }

    (mg.max(-4), eg.max(-4))
}

// range: -6 <-> 12
pub(super) fn escape_score_danger_bonus(
    board: &Board,
    color: Color,
    info: &EvalInfo,
) -> (i32, i32) {
    if info.phase() < 10 {
        return (0, 0);
    }

    let all_attacks = info.all_attacks(color.opposite());

    let friends = board.occupancy_of(color);

    let king_ring = info.king_ring(color); // this includes the king itself

    let escape_squares = (king_ring & !all_attacks & !friends).count_ones() as i32;

    let mg = if escape_squares == 0 {
        12
    } else if escape_squares <= 1 {
        4
    } else if escape_squares <= 2 {
        1
    } else {
        -6
    };
    let eg = mg;
    (mg, eg)
}

// range: 0 <-> 40.  // reasonably more likely to be at max ~20
pub(super) fn defender_danger_bonus(board: &Board, color: Color, info: &EvalInfo) -> (i32, i32) {
    let defended_squares = info.king_ring(color) & info.attacked_by_two(color); // king is included in regular attacks

    let local_defenders =
        info.king_ring(color) & board.occupancy_of(color) & !board.pieces(color, PieceType::Pawn);

    let mut mg = 0;
    let mut eg = mg;

    // since this represents king danger the bonus is negative
    mg -= defended_squares.count_ones() as i32 * 2;
    eg -= defended_squares.count_ones() as i32 * 2;
    mg -= local_defenders.count_ones() as i32 * 3;
    eg -= local_defenders.count_ones() as i32 * 3;

    (mg, eg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bitboard::{A1, G1, H8, square_from_algebraic};

    fn pressure(fen: &str, defender: Color) -> KingPressure {
        let board = Board::from_fen(fen).unwrap();
        king_pressure(&board, defender, &EvalInfo::calculate(&board))
    }

    #[test]
    fn middlegame_danger_rounds_the_combined_file_and_defender_weights() {
        assert_eq!(base_mg_danger(2, -3, 7, 4, 1, -3), 37);
        assert_eq!(base_mg_danger(0, 0, 1, 0, 0, 0), 22);
        assert_eq!(base_mg_danger(0, 0, 0, 0, 0, -3), 19);
    }

    #[test]
    fn corner_zones_extend_inward_and_leave_the_escape_ring_alone() {
        let white = attack_zone(Color::White, A1);
        assert_eq!(white, 0x070707 & !bit(A1));
        assert_eq!(attack_zone(Color::Black, H8), white.reverse_bits());
        let board = Board::from_fen("7k/8/8/8/8/8/8/K7 w - - 0 1").unwrap();
        let info = EvalInfo::calculate(&board);
        assert_eq!(info.king_ring(Color::White), bit(A1) | king_attacks(A1));
    }

    #[test]
    fn flank_counts_contacts_in_camp_and_counts_overlap_twice() {
        let rook = pressure("k6r/8/8/8/8/8/8/6K1 w - - 0 1", Color::White);
        let both = pressure("k6r/8/3b4/8/8/8/8/6K1 w - - 0 1", Color::White);
        let mirrored = pressure("6k1/8/8/8/8/3B4/8/K6R b - - 0 1", Color::Black);
        assert_eq!(rook.flank_contacts, 5);
        assert_eq!(both.flank_contacts, 9);
        assert_eq!(both, mirrored);
    }

    #[test]
    fn a_rook_earns_one_safe_check_bonus_or_counts_unsafe_destinations() {
        // From h8 the rook can give check on a8 or h1. Nb6 covers a8, Nf2 covers h1.
        for (fen, safe, unsafe_count) in [
            ("7r/6k1/8/8/8/8/8/K7 w - - 0 1", 90, 0),
            ("7r/6k1/1N6/8/8/8/8/K7 w - - 0 1", 90, 0),
            ("7r/6k1/1N6/8/8/8/5N2/K7 w - - 0 1", 0, 2),
        ] {
            let value = pressure(fen, Color::White);
            assert_eq!(
                (value.safe_check_cp, value.unsafe_checks),
                (safe, unsafe_count)
            );
        }
    }

    #[test]
    fn pawn_and_king_together_defend_a_weak_zone_square() {
        let h3 = bit(square_from_algebraic("h3").unwrap());
        let bare = pressure("k7/8/4b3/8/8/6K1/8/8 w - - 0 1", Color::White);
        let board = Board::from_fen("k7/8/4b3/8/8/6K1/6P1/8 w - - 0 1").unwrap();
        let info = EvalInfo::calculate(&board);
        assert_ne!(info.king_double_attacks[Color::White.idx()] & h3, 0);
        assert_eq!(info.attacked_by_two(Color::White) & h3, 0);
        assert_eq!(
            king_pressure(&board, Color::White, &info).weak_squares,
            bare.weak_squares - 1
        );
    }

    #[test]
    fn participants_and_contacts_are_distinct() {
        let neighbors = king_attacks(G1);
        let mut group = KingAttackers {
            zone: attack_zone(Color::White, G1),
            ..Default::default()
        };
        group.record(
            PieceType::Knight,
            knight_attacks(square_from_algebraic("g3").unwrap()),
            neighbors,
        );
        group.record(PieceType::Bishop, FILE_A, neighbors);
        assert_eq!(
            (group.pieces, group.weight, group.adjacent_contacts),
            (1, 77, 2)
        );
    }
}
