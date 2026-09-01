use crate::bitboard::{
    Bitboard, Square, bishop_attacks, bit, king_attacks, knight_attacks, pawn_attacks_from_square,
    pop_lsb, queen_attacks, rook_attacks,
};
use crate::board::Board;
use crate::eval::MAX_PHASE;
use crate::eval::king::king_eval;
use crate::eval::knight::knight_eval;
pub use crate::eval::lookup::KING_DANGER_TABLE;
use crate::eval::mobility::mobility_score;
use crate::eval::pawn::pawn_eval;
use crate::eval::sliders::sliders_eval;
use crate::types::{COLORS, Color, PieceType};

pub const CENTER_SQUARES: Bitboard = 0x0000_3C3C_3C3C_0000;

pub const CENTER_4: Bitboard = 0x0000_0018_1800_0000;

pub const BLACK_SQUARES: Bitboard = 0xAA55_AA55_AA55_AA55;

pub const WHITE_SQUARES: Bitboard = !BLACK_SQUARES;

pub const MAX_DANGER: usize = 100;

#[derive(Clone, PartialEq, Eq)]
pub struct EvalInfo {
    // king info
    pub king_squares: [Square; 2], // indexed as color
    pub king_ring: [Bitboard; 2],
    pub king_attack_weight: [i32; 2], // color is the defender -- ...[white] = danger to the white king

    // attacker info
    pub attacks: [[Bitboard; 6]; 2], // Stores all attacks of each piece type [color][piece type]
    pub all_attacks: [Bitboard; 2],  // all attackers of each color
    pub attacked_by_two: [Bitboard; 2], // stores all squares that are attacked by two or more pieces

    // general
    pub phase: i32,
}

impl EvalInfo {
    pub fn calculate(board: &Board) -> EvalInfo {
        let mut eval_info = EvalInfo {
            king_squares: [0; 2],
            king_ring: [0; 2],
            king_attack_weight: [0i32; 2],

            attacks: [[0; 6]; 2],
            all_attacks: [0; 2],
            attacked_by_two: [0; 2],

            phase: board.phase(),
        };

        for color in COLORS {
            let mut king = board.pieces(color, PieceType::King);
            let king_sq = pop_lsb(&mut king).expect("No king in Eval Calculations!");

            eval_info.king_squares[color.idx()] = king_sq;
            eval_info.king_ring[color.idx()] = king_attacks(king_sq) | bit(king_sq);
        }

        let mut pressure = [0i8; 64]; // pressure is stored as white attacks - black attacks(so always from whites perspective)

        let king_rings = [
            king_attacks(eval_info.king_square(Color::White)),
            king_attacks(eval_info.king_square(Color::Black)),
        ];

        let king_zone = king_rings[0] | king_rings[1];

        let occupied = board.all_occupancy();

        for color in COLORS {
            let color_idx = color.idx();

            let mut once = 0u64;
            let mut twice = 0u64;

            let mut pawns = board.pieces(color, PieceType::Pawn);
            while let Some(sq) = pop_lsb(&mut pawns) {
                let attacks = pawn_attacks_from_square(color, sq);

                add_pressure(color, attacks, king_zone, &mut pressure);

                twice |= once & attacks;
                once |= attacks;
                eval_info.attacks[color_idx][PieceType::Pawn.idx()] |= attacks;
            }

            let mut knights = board.pieces(color, PieceType::Knight);
            while let Some(sq) = pop_lsb(&mut knights) {
                let attacks = knight_attacks(sq);

                add_pressure(color, attacks, king_zone, &mut pressure);

                twice |= once & attacks;
                once |= attacks;
                eval_info.attacks[color_idx][PieceType::Knight.idx()] |= attacks;
            }

            let mut bishops = board.pieces(color, PieceType::Bishop);
            while let Some(sq) = pop_lsb(&mut bishops) {
                let attacks = bishop_attacks(sq, occupied);

                add_pressure(color, attacks, king_zone, &mut pressure);

                twice |= once & attacks;
                once |= attacks;
                eval_info.attacks[color_idx][PieceType::Bishop.idx()] |= attacks;
            }

            let mut rooks = board.pieces(color, PieceType::Rook);
            while let Some(sq) = pop_lsb(&mut rooks) {
                let attacks = rook_attacks(sq, occupied);

                add_pressure(color, attacks, king_zone, &mut pressure);

                twice |= once & attacks;
                once |= attacks;
                eval_info.attacks[color_idx][PieceType::Rook.idx()] |= attacks;
            }

            let mut queens = board.pieces(color, PieceType::Queen);
            while let Some(sq) = pop_lsb(&mut queens) {
                let attacks = queen_attacks(sq, occupied);

                add_pressure(color, attacks, king_zone, &mut pressure);

                twice |= once & attacks;
                once |= attacks;
                eval_info.attacks[color_idx][PieceType::Queen.idx()] |= attacks;
            }

            let king_sq = eval_info.king_squares[color_idx];
            let attacks = king_attacks(king_sq);
            twice |= once & attacks;
            eval_info.attacks[color_idx][PieceType::King.idx()] |= attacks;

            eval_info.all_attacks[color_idx] = eval_info.attacks[color_idx][PieceType::Pawn.idx()]
                | eval_info.attacks[color_idx][PieceType::Knight.idx()]
                | eval_info.attacks[color_idx][PieceType::Bishop.idx()]
                | eval_info.attacks[color_idx][PieceType::Rook.idx()]
                | eval_info.attacks[color_idx][PieceType::Queen.idx()]
                | eval_info.attacks[color_idx][PieceType::King.idx()];

            eval_info.attacked_by_two[color_idx] = twice;
        }

        for defender in COLORS {
            let mut current_king_ring = king_rings[defender.idx()];
            let mut danger = 0i32;

            while let Some(sq) = pop_lsb(&mut current_king_ring) {
                let pressure = pressure[sq as usize] as i32;

                let surplus = match defender {
                    Color::White => -pressure,
                    Color::Black => pressure,
                };

                if surplus > 0 {
                    danger += surplus * surplus * surplus;
                }
            }

            eval_info.king_attack_weight[defender.idx()] = danger;
        }
        eval_info
    }

    // *****************
    // **** GETTERS ****
    // *****************

    pub fn king_square(&self, color: Color) -> Square {
        self.king_squares[color.idx()]
    }

    pub fn king_ring(&self, color: Color) -> Bitboard {
        self.king_ring[color.idx()]
    }

    pub fn king_attack_weight(&self, color: Color) -> i32 {
        self.king_attack_weight[color.idx()]
    }

    pub fn attacks(&self, color: Color, piece: PieceType) -> Bitboard {
        self.attacks[color.idx()][piece.idx()]
    }

    pub fn all_attacks(&self, color: Color) -> Bitboard {
        self.all_attacks[color.idx()]
    }

    pub fn attacked_by_two(&self, color: Color) -> Bitboard {
        self.attacked_by_two[color.idx()]
    }

    pub fn phase(&self) -> i32 {
        self.phase
    }
}

#[inline(always)]
fn add_pressure(attacker: Color, attacks: Bitboard, king_zone: Bitboard, pressure: &mut [i8; 64]) {
    let delta: i8 = match attacker {
        Color::White => 1,
        Color::Black => -1,
    };

    let mut hits = attacks & king_zone;

    while let Some(sq) = pop_lsb(&mut hits) {
        pressure[sq as usize] += delta;
    }
}

pub fn evaluation(board: &Board) -> i32 {
    let mut total_eval = 0;

    let eval_info = EvalInfo::calculate(board);

    let phase = board.phase();
    let eg_phase = MAX_PHASE - phase;

    let mg_pst = board.mg_pst();
    let eg_pst = board.eg_pst();

    let pst_eval = (mg_pst * phase + eg_pst * eg_phase) / MAX_PHASE;

    total_eval += board.material();
    total_eval += pst_eval;

    total_eval += mobility_score(board, &eval_info);

    total_eval += pawn_eval(board, &eval_info);
    total_eval += knight_eval(board, &eval_info);
    // bishop, rook, queen
    total_eval += sliders_eval(board, &eval_info);
    total_eval += king_eval(board, &eval_info);

    match board.side_to_move() {
        // for tempo
        Color::White => total_eval += 15,
        Color::Black => total_eval -= 15,
    };

    total_eval
}

pub fn lazy_eval(board: &Board) -> i32 {
    let phase = board.phase();

    let pst_eval = if phase > 12 {
        board.mg_pst()
    } else {
        board.eg_pst()
    };

    pst_eval + board.material()
}

pub fn lazy_eval_for_turn(board: &Board) -> i32 {
    let eval = lazy_eval(board);
    match board.side_to_move() {
        Color::White => eval,
        Color::Black => -eval,
    }
}

pub fn evaluation_for_turn(board: &Board) -> i32 {
    let eval = evaluation(board);
    match board.side_to_move() {
        Color::White => eval,
        Color::Black => -eval,
    }
}

// takes the score and scales based on the min phase(start) to the max phase(full)
// For example start = 4 and full = 12 means at phase 12 the score is just score
// at phase 4 the score is 0. At phase 8 it would be 8-4 / 12-4 = 4/8 = 50% of the original score
pub fn scale_by_phase(score: i32, phase: i32, start: i32, full: i32) -> i32 {
    if phase <= start {
        0
    } else if phase >= full {
        score
    } else {
        score * (phase - start) / (full - start)
    }
}
