use crate::bitboard::{
    Bitboard, Square, bishop_attacks, bit, king_attacks, knight_attacks, pawn_attacks_from_square,
    pop_lsb, queen_attacks, rook_attacks,
};
use crate::board::Board;
use crate::eval::{
    king::king_eval, knight::knight_eval, mobility::mobility_score, pawn::pawn_eval,
    phase::MAX_PHASE, sliders::sliders_eval,
};
use crate::types::{COLORS, Color, PIECE_TYPES, PieceType};

pub const CENTER_SQUARES: Bitboard = 0x0000_3C3C_3C3C_0000;

pub const BLACK_SQUARES: Bitboard = 0xAA55_AA55_AA55_AA55;

pub const WHITE_SQUARES: Bitboard = BLACK_SQUARES << 1;

const KING_ATTACK_WEIGHTS: [i32; 6] = [
    0,  // Pawn, handled separately
    20, // Knight
    20, // Bishop
    40, // Rook
    80, // Queen
    0,  // King
];

pub const KING_DANGER_TABLE: [i32; 101] = [
    0, 0, 0, 0, 0, 1, 1, 1, 2, 2, 3, 3, 4, 5, 6, 7, 8, 9, 10, 12, 14, 16, 18, 20, 22, 24, 27, 30,
    33, 36, 39, 42, 46, 50, 54, 58, 62, 67, 72, 77, 82, 87, 93, 99, 105, 111, 118, 125, 132, 139,
    146, 154, 162, 170, 178, 187, 196, 205, 214, 224, 234, 244, 254, 265, 276, 287, 298, 310, 322,
    334, 346, 359, 372, 385, 398, 412, 426, 440, 454, 469, 484, 499, 514, 530, 546, 562, 578, 595,
    612, 629, 646, 664, 682, 700, 718, 737, 756, 775, 794, 814, 834,
];

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
            king_attack_weight: [0; 2],

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

        let occupied = board.all_occupancy();

        for color in COLORS {
            let color_idx = color.idx();

            let mut once = 0u64;
            let mut twice = 0u64;

            let mut pawns = board.pieces(color, PieceType::Pawn);
            while let Some(sq) = pop_lsb(&mut pawns) {
                let attacks = pawn_attacks_from_square(sq, color);
                twice |= once & attacks;
                once |= attacks;
                eval_info.attacks[color_idx][PieceType::Pawn.idx()] |= attacks;
            }

            let mut knights = board.pieces(color, PieceType::Knight);
            while let Some(sq) = pop_lsb(&mut knights) {
                let attacks = knight_attacks(sq);
                twice |= once & attacks;
                once |= attacks;
                eval_info.attacks[color_idx][PieceType::Knight.idx()] |= attacks;
            }

            let mut bishops = board.pieces(color, PieceType::Bishop);
            while let Some(sq) = pop_lsb(&mut bishops) {
                let attacks = bishop_attacks(sq, occupied);
                twice |= once & attacks;
                once |= attacks;
                eval_info.attacks[color_idx][PieceType::Bishop.idx()] |= attacks;
            }

            let mut rooks = board.pieces(color, PieceType::Rook);
            while let Some(sq) = pop_lsb(&mut rooks) {
                let attacks = rook_attacks(sq, occupied);
                twice |= once & attacks;
                once |= attacks;
                eval_info.attacks[color_idx][PieceType::Rook.idx()] |= attacks;
            }

            let mut queens = board.pieces(color, PieceType::Queen);
            while let Some(sq) = pop_lsb(&mut queens) {
                let attacks = queen_attacks(sq, occupied);
                twice |= once & attacks;
                once |= attacks;
                eval_info.attacks[color_idx][PieceType::Queen.idx()] |= attacks;
            }

            let king_sq = eval_info.king_squares[color_idx];
            let attacks = king_attacks(king_sq);
            twice |= once & attacks;
            // once |= attacks;
            eval_info.attacks[color_idx][PieceType::King.idx()] |= attacks;

            eval_info.all_attacks[color_idx] = eval_info.attacks[color_idx][PieceType::Pawn.idx()]
                | eval_info.attacks[color_idx][PieceType::Knight.idx()]
                | eval_info.attacks[color_idx][PieceType::Bishop.idx()]
                | eval_info.attacks[color_idx][PieceType::Rook.idx()]
                | eval_info.attacks[color_idx][PieceType::Queen.idx()]
                | eval_info.attacks[color_idx][PieceType::King.idx()];

            eval_info.attacked_by_two[color_idx] = twice;
        }

        for attacker in COLORS {
            let defender = attacker.opposite();

            let attacker_idx = attacker.idx();
            let defender_idx = defender.idx();

            let king_ring = eval_info.king_ring[defender_idx];

            let mut danger = 0;

            for piece in PIECE_TYPES {
                if piece == PieceType::King {
                    continue;
                }

                let hits =
                    (eval_info.attacks[attacker_idx][piece.idx()] & king_ring).count_ones() as i32;

                if hits == 0 {
                    continue;
                }

                if piece == PieceType::Pawn {
                    danger += hits * 8;
                } else {
                    danger += KING_ATTACK_WEIGHTS[piece.idx()] + hits * 5;
                }
            }

            eval_info.king_attack_weight[defender_idx] = danger;
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

    // match board.side_to_move() {
    //     // for tempo
    //     Color::White => total_eval + 10,
    //     Color::Black => total_eval - 10,
    // };

    total_eval
}

pub fn lazy_eval(board: &Board) -> i32 {
    let phase = board.phase();
    let eg_phase = MAX_PHASE - phase;

    let mg_pst = board.mg_pst();
    let eg_pst = board.eg_pst();

    let pst_eval = (mg_pst * phase + eg_pst * eg_phase) / MAX_PHASE;
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
