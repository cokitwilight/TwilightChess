use crate::bitboard::pop_lsb;
use crate::board::Board;
use crate::eval::EvalInfo;
use crate::eval::MAX_PHASE;
use crate::eval::eval::MAX_DANGER;
use crate::eval::king::{
    defender_danger_bonus, escape_score_danger_bonus, king_eval, king_eval_danger_raw,
    open_diagonal_danger_bonus, open_file_danger_bonus, pawn_shield_danger_score,
};
use crate::eval::knight::{knight_eval, knight_eval_raw, knight_outpost_bonus};
use crate::eval::lookup::KING_DANGER_TABLE;
use crate::eval::mobility::{
    available_moves, development_penalty, hanging_pieces, mobility_score, mobility_score_raw,
    move_aggression, move_pressure, space_bonus,
};
use crate::eval::pawn::{
    backwards_pawn_bonus, center_pawns_bonus, isolated_pawns_bonus, passed_pawn_bonus, pawn_chain,
    pawn_eval, pawn_eval_raw, pawn_storm_bonus, pawn_tempo_bonus, stacked_pawns_bonus,
};
use crate::eval::pst::{eg_pst_bonus_at, mg_pst_bonus_at};
use crate::eval::sliders::{
    bishop_blocked_by_pawns_bonus, bishop_pair_bonus, connected_diagonals_bonus,
    connected_file_bonus, rook_on_the_seventh, sliders_eval, sliders_eval_raw,
    straights_on_open_file, straights_xray_bonus, xray_pressure_diagonal_bonus,
};
use crate::types::{Color, PIECE_TYPES, PieceType};

/// The compact, white-perspective breakdown used by both the GUI and debugger.
///
/// This is deliberately separate from `evaluation()` and is never constructed by
/// search. The production evaluator retains its existing `&Board -> i32` API.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct EvalBreakdown {
    pub material: i32,
    pub pst: i32,
    pub mobility: i32,
    pub pawns: i32,
    pub knights: i32,
    pub sliders: i32,
    pub king: i32,
    pub tempo: i32,
    pub total: i32,
    pub side_to_move_total: i32,
    pub phase: i32,
}

pub fn evaluation_breakdown(board: &Board) -> EvalBreakdown {
    let eval_info = EvalInfo::calculate(board);
    let phase = board.phase();
    let eg_phase = MAX_PHASE - phase;
    let pst = (board.mg_pst() * phase + board.eg_pst() * eg_phase) / MAX_PHASE;
    let material = board.material();
    let mobility = mobility_score(board, &eval_info);
    let pawns = pawn_eval(board, &eval_info);
    let knights = knight_eval(board, &eval_info);
    let sliders = sliders_eval(board, &eval_info);
    let king = king_eval(board, &eval_info);
    let tempo = match board.side_to_move() {
        Color::White => 15,
        Color::Black => -15,
    };
    let total = material + pst + mobility + pawns + knights + sliders + king + tempo;
    let side_to_move_total = match board.side_to_move() {
        Color::White => total,
        Color::Black => -total,
    };

    EvalBreakdown {
        material,
        pst,
        mobility,
        pawns,
        knights,
        sliders,
        king,
        tempo,
        total,
        side_to_move_total,
        phase,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ScoreLine {
    pub name: &'static str,
    pub white: i32,
    pub black: i32,
    pub net: i32,
}

impl ScoreLine {
    fn from_sides(name: &'static str, white: i32, black: i32) -> Self {
        Self {
            name,
            white,
            black,
            net: white - black,
        }
    }

    fn with_net(name: &'static str, white: i32, black: i32, net: i32) -> Self {
        Self {
            name,
            white,
            black,
            net,
        }
    }
}

pub(super) struct SummaryBreakdown {
    pub components: Vec<ScoreLine>,
    pub total: ScoreLine,
    pub phase: i32,
    pub side_to_move: Color,
    pub side_to_move_total: i32,
}

pub(super) fn summary_breakdown(board: &Board, info: &EvalInfo) -> SummaryBreakdown {
    let compact = evaluation_breakdown(board);
    let material_white = material_for_color(board, Color::White);
    let material_black = material_for_color(board, Color::Black);

    let (white_mg, white_eg) = pst_for_color(board, Color::White);
    let phase = board.phase();
    let eg_phase = MAX_PHASE - phase;
    let white_pst = (white_mg * phase + white_eg * eg_phase) / MAX_PHASE;

    // The production evaluator tapers the net PST and performs one integer
    // division. Allocate any one-centipawn rounding difference to Black so the
    // displayed White - Black value exactly matches production.
    let black_pst = white_pst - compact.pst;

    let tempo_white = i32::from(board.side_to_move() == Color::White) * 15;
    let tempo_black = i32::from(board.side_to_move() == Color::Black) * 15;

    let components = vec![
        ScoreLine::from_sides("Material", material_white, material_black),
        ScoreLine::with_net("Piece-square tables", white_pst, black_pst, compact.pst),
        ScoreLine::from_sides(
            "Mobility",
            mobility_score_raw(board, Color::White, info),
            mobility_score_raw(board, Color::Black, info),
        ),
        ScoreLine::from_sides(
            "Pawns",
            pawn_eval_raw(board, Color::White, info),
            pawn_eval_raw(board, Color::Black, info),
        ),
        ScoreLine::from_sides(
            "Knights",
            knight_eval_raw(board, Color::White, info),
            knight_eval_raw(board, Color::Black, info),
        ),
        ScoreLine::from_sides(
            "Sliders",
            sliders_eval_raw(board, Color::White, info),
            sliders_eval_raw(board, Color::Black, info),
        ),
        ScoreLine::from_sides(
            "King safety",
            king_eval_danger_raw(board, Color::White, info),
            king_eval_danger_raw(board, Color::Black, info),
        ),
        ScoreLine::from_sides("Tempo", tempo_white, tempo_black),
    ];

    debug_assert_eq!(components[0].net, compact.material);
    debug_assert_eq!(components[1].net, compact.pst);
    debug_assert_eq!(components[2].net, compact.mobility);
    debug_assert_eq!(components[3].net, compact.pawns);
    debug_assert_eq!(components[4].net, compact.knights);
    debug_assert_eq!(components[5].net, compact.sliders);
    debug_assert_eq!(components[6].net, compact.king);
    debug_assert_eq!(components[7].net, compact.tempo);

    let total_white = components.iter().map(|line| line.white).sum();
    let total_black = components.iter().map(|line| line.black).sum();
    let total = ScoreLine::with_net("Total", total_white, total_black, compact.total);
    debug_assert_eq!(total.white - total.black, total.net);

    SummaryBreakdown {
        components,
        total,
        phase,
        side_to_move: board.side_to_move(),
        side_to_move_total: compact.side_to_move_total,
    }
}

pub(super) fn material_lines(board: &Board) -> Vec<ScoreLine> {
    PIECE_TYPES
        .into_iter()
        .map(|piece| {
            ScoreLine::from_sides(
                piece_name(piece),
                board.pieces(Color::White, piece).count_ones() as i32 * piece.value(),
                board.pieces(Color::Black, piece).count_ones() as i32 * piece.value(),
            )
        })
        .collect()
}

pub(super) fn middlegame_pst_lines(board: &Board) -> Vec<ScoreLine> {
    pst_lines(board, false)
}

pub(super) fn endgame_pst_lines(board: &Board) -> Vec<ScoreLine> {
    pst_lines(board, true)
}

pub(super) fn pawn_lines(board: &Board, info: &EvalInfo) -> Vec<ScoreLine> {
    let white_pawns = board.pieces(Color::White, PieceType::Pawn);
    let black_pawns = board.pieces(Color::Black, PieceType::Pawn);

    vec![
        ScoreLine::from_sides(
            "Stacked pawns",
            stacked_pawns_bonus(board, Color::White, white_pawns, info),
            stacked_pawns_bonus(board, Color::Black, black_pawns, info),
        ),
        ScoreLine::from_sides(
            "Center pawns",
            center_pawns_bonus(board, Color::White, white_pawns, info),
            center_pawns_bonus(board, Color::Black, black_pawns, info),
        ),
        ScoreLine::from_sides(
            "Pawn tempo",
            pawn_tempo_bonus(board, Color::White, white_pawns, info),
            pawn_tempo_bonus(board, Color::Black, black_pawns, info),
        ),
        ScoreLine::from_sides(
            "Passed pawns",
            passed_pawn_bonus(board, Color::White, white_pawns, info),
            passed_pawn_bonus(board, Color::Black, black_pawns, info),
        ),
        ScoreLine::from_sides(
            "Pawn storm",
            pawn_storm_bonus(board, Color::White, white_pawns, info),
            pawn_storm_bonus(board, Color::Black, black_pawns, info),
        ),
        ScoreLine::from_sides(
            "Pawn chain",
            pawn_chain(board, Color::White, white_pawns, info),
            pawn_chain(board, Color::Black, black_pawns, info),
        ),
        ScoreLine::from_sides(
            "Isolated pawns",
            isolated_pawns_bonus(board, Color::White, white_pawns, info),
            isolated_pawns_bonus(board, Color::Black, black_pawns, info),
        ),
        ScoreLine::from_sides(
            "Backward pawns",
            backwards_pawn_bonus(board, Color::White, white_pawns, info),
            backwards_pawn_bonus(board, Color::Black, black_pawns, info),
        ),
    ]
}

pub(super) fn knight_lines(board: &Board, info: &EvalInfo) -> Vec<ScoreLine> {
    vec![ScoreLine::from_sides(
        "Knight outposts",
        knight_outpost_bonus(
            board,
            Color::White,
            board.pieces(Color::White, PieceType::Knight),
            info,
        ),
        knight_outpost_bonus(
            board,
            Color::Black,
            board.pieces(Color::Black, PieceType::Knight),
            info,
        ),
    )]
}

pub(super) fn mobility_lines(board: &Board, info: &EvalInfo) -> Vec<ScoreLine> {
    side_feature_lines(
        [
            "Development",
            "Available moves",
            "Move pressure",
            "Hanging/weak pieces",
            "Aggression",
            "Space",
        ],
        [
            development_penalty,
            available_moves,
            move_pressure,
            hanging_pieces,
            move_aggression,
            space_bonus,
        ],
        board,
        info,
    )
}

pub(super) fn slider_lines(board: &Board, info: &EvalInfo) -> Vec<ScoreLine> {
    let white_diagonals = board.pieces(Color::White, PieceType::Bishop)
        | board.pieces(Color::White, PieceType::Queen);
    let black_diagonals = board.pieces(Color::Black, PieceType::Bishop)
        | board.pieces(Color::Black, PieceType::Queen);
    let white_straights =
        board.pieces(Color::White, PieceType::Rook) | board.pieces(Color::White, PieceType::Queen);
    let black_straights =
        board.pieces(Color::Black, PieceType::Rook) | board.pieces(Color::Black, PieceType::Queen);

    vec![
        ScoreLine::from_sides(
            "Connected diagonals (B/Q)",
            connected_diagonals_bonus(board, Color::White, white_diagonals, info),
            connected_diagonals_bonus(board, Color::Black, black_diagonals, info),
        ),
        ScoreLine::from_sides(
            "Diagonal x-ray pressure",
            xray_pressure_diagonal_bonus(board, Color::White, white_diagonals, info),
            xray_pressure_diagonal_bonus(board, Color::Black, black_diagonals, info),
        ),
        ScoreLine::from_sides(
            "Bishop pair",
            bishop_pair_bonus(board, Color::White),
            bishop_pair_bonus(board, Color::Black),
        ),
        ScoreLine::from_sides(
            "Bishops blocked by pawns",
            bishop_blocked_by_pawns_bonus(board, Color::White, info),
            bishop_blocked_by_pawns_bonus(board, Color::Black, info),
        ),
        ScoreLine::from_sides(
            "Connected files (R/Q)",
            connected_file_bonus(board, Color::White, white_straights, info),
            connected_file_bonus(board, Color::Black, black_straights, info),
        ),
        ScoreLine::from_sides(
            "Rook on seventh",
            rook_on_the_seventh(board, Color::White, info),
            rook_on_the_seventh(board, Color::Black, info),
        ),
        ScoreLine::from_sides(
            "Open/semi-open files",
            straights_on_open_file(board, Color::White, white_straights, info),
            straights_on_open_file(board, Color::Black, black_straights, info),
        ),
        ScoreLine::from_sides(
            "Straight x-ray pressure",
            straights_xray_bonus(board, Color::White, white_straights, info),
            straights_xray_bonus(board, Color::Black, black_straights, info),
        ),
    ]
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct KingSideBreakdown {
    pub attacker_weight: i32,
    pub pawn_shield: i32,
    pub open_files: i32,
    pub open_diagonals: i32,
    pub escape_squares: i32,
    pub defenders: i32,
    pub raw_danger: i32,
    pub clamped_danger: i32,
    pub table_penalty: i32,
    pub final_score: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct KingBreakdown {
    pub white: KingSideBreakdown,
    pub black: KingSideBreakdown,
}

pub(super) fn king_breakdown(board: &Board, info: &EvalInfo) -> KingBreakdown {
    KingBreakdown {
        white: king_side_breakdown(board, Color::White, info),
        black: king_side_breakdown(board, Color::Black, info),
    }
}

pub(super) fn tempo_lines(board: &Board) -> Vec<ScoreLine> {
    vec![ScoreLine::from_sides(
        "Side to move",
        i32::from(board.side_to_move() == Color::White) * 15,
        i32::from(board.side_to_move() == Color::Black) * 15,
    )]
}

fn side_feature_lines<const N: usize>(
    names: [&'static str; N],
    functions: [fn(&Board, Color, &EvalInfo) -> i32; N],
    board: &Board,
    info: &EvalInfo,
) -> Vec<ScoreLine> {
    names
        .into_iter()
        .zip(functions)
        .map(|(name, function)| {
            ScoreLine::from_sides(
                name,
                function(board, Color::White, info),
                function(board, Color::Black, info),
            )
        })
        .collect()
}

fn king_side_breakdown(board: &Board, color: Color, info: &EvalInfo) -> KingSideBreakdown {
    let king_sq = info.king_square(color);
    let attacker_weight = info.king_attack_weight(color);
    let pawn_shield = pawn_shield_danger_score(board, color, king_sq, info);
    let open_files = open_file_danger_bonus(board, color, king_sq, info);
    let open_diagonals = open_diagonal_danger_bonus(board, color, king_sq, info);
    let escape_squares = escape_score_danger_bonus(board, color, info);
    let defenders = defender_danger_bonus(board, color, info);
    let raw_danger =
        attacker_weight + pawn_shield + open_files + open_diagonals + escape_squares + defenders;
    let clamped_danger = raw_danger.clamp(0, MAX_DANGER as i32);
    let table_penalty = -KING_DANGER_TABLE[clamped_danger as usize];
    let final_score = king_eval_danger_raw(board, color, info);

    KingSideBreakdown {
        attacker_weight,
        pawn_shield,
        open_files,
        open_diagonals,
        escape_squares,
        defenders,
        raw_danger,
        clamped_danger,
        table_penalty,
        final_score,
    }
}

fn material_for_color(board: &Board, color: Color) -> i32 {
    PIECE_TYPES
        .into_iter()
        .map(|piece| board.pieces(color, piece).count_ones() as i32 * piece.value())
        .sum()
}

fn pst_for_color(board: &Board, color: Color) -> (i32, i32) {
    let mut middlegame = 0;
    let mut endgame = 0;

    for piece in PIECE_TYPES {
        let mut pieces = board.pieces(color, piece);
        while let Some(square) = pop_lsb(&mut pieces) {
            middlegame += mg_pst_bonus_at(color, piece, square);
            endgame += eg_pst_bonus_at(color, piece, square);
        }
    }

    (middlegame, endgame)
}

fn pst_lines(board: &Board, endgame: bool) -> Vec<ScoreLine> {
    PIECE_TYPES
        .into_iter()
        .map(|piece| {
            let score = |color| {
                let mut total = 0;
                let mut pieces = board.pieces(color, piece);
                while let Some(square) = pop_lsb(&mut pieces) {
                    total += if endgame {
                        eg_pst_bonus_at(color, piece, square)
                    } else {
                        mg_pst_bonus_at(color, piece, square)
                    };
                }
                total
            };

            ScoreLine::from_sides(piece_name(piece), score(Color::White), score(Color::Black))
        })
        .collect()
}

fn piece_name(piece: PieceType) -> &'static str {
    match piece {
        PieceType::Pawn => "Pawns",
        PieceType::Knight => "Knights",
        PieceType::Bishop => "Bishops",
        PieceType::Rook => "Rooks",
        PieceType::Queen => "Queens",
        PieceType::King => "King",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eval::evaluation;

    const TEST_FENS: &[&str] = &[
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
        "r1b1kbnr/2p2p1p/p2pP1p1/1p6/2B1P3/2N1BP2/PPP2P1P/R3K1R1 b Qkq - 0 11",
        "r1b1kbnr/2p2p1p/p2pP1p1/8/2p1P3/2N1BP2/PPP2P1P/R3K1R1 w Qkq - 0 12",
        "2r2rk1/6pp/2P1P3/3p4/p6P/2N2N2/PP3PP1/2R2RK1 w - - 0 1",
        "4k3/pppp4/8/8/8/8/P1P1P1P1/4K3 w - - 0 1",
    ];

    #[test]
    fn compact_breakdown_matches_production_evaluation() {
        for fen in TEST_FENS {
            let board = Board::from_fen(fen).expect("valid test FEN");
            let breakdown = evaluation_breakdown(&board);
            let summary = summary_breakdown(&board, &EvalInfo::calculate(&board));

            assert_eq!(breakdown.total, evaluation(&board), "FEN: {fen}");
            assert_eq!(summary.total.net, breakdown.total, "FEN: {fen}");
            assert_eq!(summary.total.white - summary.total.black, breakdown.total);
        }
    }

    #[test]
    fn named_bonus_rows_reconstruct_each_linear_section() {
        for fen in TEST_FENS {
            let board = Board::from_fen(fen).expect("valid test FEN");
            let info = EvalInfo::calculate(&board);

            assert_section(
                &pawn_lines(&board, &info),
                pawn_eval_raw(&board, Color::White, &info),
                pawn_eval_raw(&board, Color::Black, &info),
                fen,
            );
            assert_section(
                &knight_lines(&board, &info),
                knight_eval_raw(&board, Color::White, &info),
                knight_eval_raw(&board, Color::Black, &info),
                fen,
            );
            assert_section(
                &mobility_lines(&board, &info),
                mobility_score_raw(&board, Color::White, &info),
                mobility_score_raw(&board, Color::Black, &info),
                fen,
            );
            assert_section(
                &slider_lines(&board, &info),
                sliders_eval_raw(&board, Color::White, &info),
                sliders_eval_raw(&board, Color::Black, &info),
                fen,
            );

            let king = king_breakdown(&board, &info);
            assert_eq!(
                king.white.final_score,
                king_eval_danger_raw(&board, Color::White, &info),
                "FEN: {fen}"
            );
            assert_eq!(
                king.black.final_score,
                king_eval_danger_raw(&board, Color::Black, &info),
                "FEN: {fen}"
            );
        }
    }

    fn assert_section(lines: &[ScoreLine], expected_white: i32, expected_black: i32, fen: &str) {
        assert_eq!(
            lines.iter().map(|line| line.white).sum::<i32>(),
            expected_white,
            "white section mismatch for FEN: {fen}"
        );
        assert_eq!(
            lines.iter().map(|line| line.black).sum::<i32>(),
            expected_black,
            "black section mismatch for FEN: {fen}"
        );
    }
}
