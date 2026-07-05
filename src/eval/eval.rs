use crate::{
    board::Board,
    eval::{
        king::king_eval,
        mobility::mobility_score,
        phase::MAX_PHASE,
        pst::{self, pst_bonus},
    },
    types::{Color, PieceType},
};

pub fn evaluation(board: &Board) -> i32 {
    let mut total_eval = 0;

    let phase = board.phase();
    let eg_phase = MAX_PHASE - phase;

    let mg_pst = board.mg_pst();
    let eg_pst = board.eg_pst();

    let pst_eval = (mg_pst * phase + eg_pst * eg_phase) / MAX_PHASE;

    total_eval += board.material();
    total_eval += pst_eval;

    total_eval += mobility_score(board, phase);

    total_eval += king_eval(board, phase);

    total_eval
}

pub fn evaluation_for_turn(board: &Board) -> i32 {
    let eval = evaluation(board);
    match board.side_to_move() {
        Color::White => eval,
        Color::Black => -eval,
    }
}
