// use crate::board::Board;
// use crate::eval::knight::knight_eval_raw;
// use crate::eval::mobility::mobility_score_raw;
// use crate::eval::pawn::pawn_eval_raw;
// use crate::eval::sliders::sliders_eval_raw;
// use crate::types::Color;

// use crate::eval::{eval::EvalInfo, king::king_eval_danger_index};

// pub struct IncrementalEvalInfo {
//     pub eval_info: EvalInfo,

//     // PST, Material, Phase are already incremental in board state
//     pub king_index: [i8; 2],
//     pub sliders: [i16; 2],

//     pub knights: [i16; 2],
//     pub pawns: [i16; 2],

//     pub mobility: [i16; 2],
// }

// impl IncrementalEvalInfo {
//     pub fn new(board: &Board) -> Self {
//         let info = EvalInfo::calculate(board);

//         let king_index = [
//             king_eval_danger_index(board, Color::White, &info),
//             king_eval_danger_index(board, Color::Black, &info),
//         ];
//         let sliders = [
//             sliders_eval_raw(board, Color::White, &info) as i16,
//             sliders_eval_raw(board, Color::Black, &info) as i16,
//         ];

//         let knights = [
//             knight_eval_raw(board, Color::White, &info) as i16,
//             knight_eval_raw(board, Color::Black, &info) as i16,
//         ];

//         let pawns = [
//             pawn_eval_raw(board, Color::White, &info) as i16,
//             pawn_eval_raw(board, Color::Black, &info) as i16,
//         ];

//         let mobility = [
//             mobility_score_raw(board, Color::White, &info) as i16,
//             mobility_score_raw(board, Color::Black, &info) as i16,
//         ];

//         Self {
//             eval_info: info,
//             king_index,
//             sliders,
//             knights,
//             pawns,
//             mobility,
//         }
//     }
// }
