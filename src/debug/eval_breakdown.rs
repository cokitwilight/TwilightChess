#![allow(dead_code)]

use crate::board::Board;
use crate::eval::phase::MAX_PHASE;
use crate::eval::{
    eval::EvalInfo, king::king_eval, knight::knight_eval, mobility::mobility_score,
    pawn::pawn_eval, sliders::sliders_eval,
};
use crate::types::Color;

#[derive(Debug, Clone, Copy, Default)]
pub struct EvalBreakdown {
    pub material: i32,
    pub pst: i32,
    pub mobility: i32,
    pub pawns: i32,
    pub knights: i32,
    pub sliders: i32,
    pub king: i32,
    pub total: i32,
    pub side_to_move_total: i32,
    pub phase: i32,
}

pub fn evaluation_breakdown(board: &Board) -> EvalBreakdown {
    let eval_info = EvalInfo::calculate(board);

    let phase = board.phase();
    let eg_phase = MAX_PHASE - phase;

    let mg_pst = board.mg_pst();
    let eg_pst = board.eg_pst();

    let mut pst = (mg_pst * phase + eg_pst * eg_phase) / MAX_PHASE;
    // for debugging pst

    pst /= 2;

    let material = board.material();
    let mobility = mobility_score(board, &eval_info);
    let pawns = pawn_eval(board, &eval_info);
    let knights = knight_eval(board, &eval_info);
    let sliders = sliders_eval(board, &eval_info);
    let king = king_eval(board, &eval_info);

    let total = material + pst + mobility + pawns + knights + sliders + king;

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
        total,
        side_to_move_total,
        phase,
    }
}

#[cfg(test)]
mod eval_tests {
    use crate::eval::pawn::print_pawn_eval;
    use crate::eval::{eval::evaluation, evaluation_for_turn};

    use super::*;

    #[test]
    fn print_eval_breakdowns_for_fens() {
        let positions = [
            (
                "Startpos",
                "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
            ),
            (
                "White up a pawn",
                "rnbqkbnr/pppp1ppp/4p3/8/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 2",
            ),
            (
                "Kiwipete",
                "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
            ),
            (
                "Open king-ish position",
                "rnbq1rk1/ppp2ppp/4pn2/3p4/1b1P4/2NBPN2/PP3PPP/R1BQ1RK1 w - - 0 7",
            ),
            ("Endgame", "8/8/3k4/8/3K4/8/4P3/8 w - - 0 1"),
            // 1. Multiple attackers around enemy king
            (
                "KingAttack_3Plus_WhiteAttacksBlackKing",
                "r4rk1/pp3ppp/2n1bn2/3p2NQ/3P4/3BPN2/PP3PPP/R4RK1 w - - 0 1", //"White Qh5, Bd3, and Ng5 all attack squares around Kg8, especially h7/f7.",
            ),
            // 2. Late middlegame with multiple passed pawns
            (
                "LateMG_WhiteTwoPassedPawns",
                "2r2rk1/6pp/2P1P3/3p4/p6P/2N2N2/PP3PP1/2R2RK1 w - - 0 1",
                //"White has advanced passed pawns on c6 and e6.",
            ),
            (
                "LateMG_BothSidesPassedPawns",
                "2r2rk1/6pp/2P1P3/3p4/8/p1N2N1p/5P2/2R2RK1 w - - 0 1",
                //"White has c6/e6 passers; Black has dangerous a3/h3 passers.",
            ),
            // 3. Isolated pawns vs normal pawn structure
            (
                "PawnStructure_WhiteIsolatedPawns",
                "4k3/pppp4/8/8/8/8/P1P1P1P1/4K3 w - - 0 1",
                //"White pawns on a2/c2/e2/g2 are all isolated.",
            ),
            (
                "PawnStructure_WhiteNormalChain",
                "4k3/pppp4/8/8/8/8/PPPP4/4K3 w - - 0 1",
                //"Same number of white pawns, but connected on a2/b2/c2/d2.",
            ),
            // 4. Bishop x-ray to king vs no bishop x-ray
            (
                "BishopXray_ToBlackKing",
                "6k1/p4p2/8/8/8/1B4K1/8/8 w - - 0 1",
                //"White bishop b3 x-rays Kg8 through the f7 pawn.",
            ),
            (
                "BishopXray_BlockedNoXray",
                "6k1/5p2/8/3p4/8/1B4K1/8/8 w - - 0 1",
                //"Same bishop/king geometry, but d5 blocks the diagonal before f7/Kg8.",
            ),
            // 5. Rook x-ray to king vs no rook x-ray, same file
            (
                "RookXray_SameFile_ToBlackKing",
                "4k3/p3p3/8/8/8/8/P7/4R1K1 w - - 0 1",
                //"White rook e1 x-rays Ke8 through the e7 pawn.",
            ),
            (
                "RookXray_SameFile_BlockedNoXray",
                "4k3/4p3/8/4p3/8/8/P7/4R1K1 w - - 0 1",
                //"Rook and king are still on the e-file, but e5 blocks before the e7 pawn/king.",
            ),
            (
                "Test Position 1.",
                "r1b2k2/pp2p2Q/2p1p1p1/3q1rN1/8/2P5/P1P2PPP/R1B2RK1 w - - 0 1",
            ),
            (
                "Test Position 2.",
                "8/k7/2Q5/pp6/4p3/2p1P1q1/P5Np/7K w - - 0 1",
            ),
        ];

        for (name, fen) in positions {
            let board = Board::from_fen(fen).expect("valid FEN");
            let b = evaluation_breakdown(&board);

            println!();
            println!("==============================");
            println!("{name}");
            println!("{fen}");
            println!("Side to move: {:?}", board.side_to_move());
            println!("Phase: {}", b.phase);
            println!("------------------------------");
            println!("{:<16} {:>8}", "Material", b.material);
            println!("{:<16} {:>8}", "PST", b.pst);
            println!("{:<16} {:>8}", "Mobility", b.mobility);
            println!("{:<16} {:>8}", "Pawns", b.pawns);
            println!("{:<16} {:>8}", "Knights", b.knights);
            println!("{:<16} {:>8}", "Sliders", b.sliders);
            println!("{:<16} {:>8}", "King", b.king);
            println!("------------------------------");
            println!("{:<16} {:>8}", "Total", b.total);
            println!("{:<16} {:>8}", "STM total", b.side_to_move_total);

            print_pawn_eval(&board);

            assert_eq!(
                b.total,
                b.material + b.pst + b.mobility + b.pawns + b.knights + b.sliders + b.king,
                "eval breakdown components do not sum to total for {name}"
            );

            assert_eq!(
                evaluation(&board),
                b.total,
                "evaluation() differs from evaluation_breakdown().total for {name}"
            );

            assert_eq!(
                evaluation_for_turn(&board),
                b.side_to_move_total,
                "evaluation_for_turn() differs from breakdown for {name}"
            );
        }
    }
}
