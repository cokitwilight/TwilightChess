use std::time::Instant;

use crate::board::Board;
use crate::engine::{Engine, SearchContext, SearchLimits, SearchResult};
use crate::eval::eval::{BLACK_SQUARES, WHITE_SQUARES};
use crate::types::{Color, PieceType};

impl Engine {
    pub fn search(
        &mut self,
        board: &Board,
        limits: SearchLimits,
        repetition_history: &Vec<u64>,
        opening_allowed: bool,
        can_print: bool,
    ) -> SearchResult {
        // CHECK IF CLONING HERE IS OK FOR REPETITION HISTORY, OR IF WE SHOULD PASS A REFERENCE
        let mut context = SearchContext::new(limits, repetition_history.clone());
        let mut board = board.clone();

        let start = Instant::now();

        if opening_allowed && let Some(book_mv) = self.get_book_move(&board) {
            // let piece = board.piece_at(book_mv.from).unwrap();

            if can_print {
                println!("Book Move");
            }

            // println!(
            //     "Book Move: {:?} {} to {}. End Stats: nodes={}, qnodes={}",
            //     piece.kind,
            //     square_name(book_mv.from),
            //     square_name(book_mv.to),
            //     self.nodes,
            //     self.qnodes
            // );

            return SearchResult {
                best_move: Some(book_mv),
                eval: 0,
                depth_reached: 0,
                stats: context.stats,
                pv: Vec::new(),
                elapsed: start.elapsed(),
            };
        }

        // iterative deepening here

        // let adjusted_depth = adjusted_depth_for_phase(context.limits.max_depth, board.phase());

        // context.limits.max_depth = adjusted_depth;

        // increments generation for the transposition table
        self.tt.new_search();

        let search_result = self.iterative_deepening(&mut board, &mut context, can_print);

        let elapsed = start.elapsed();

        SearchResult {
            best_move: search_result.best_move,
            eval: search_result.eval,
            depth_reached: search_result.depth_reached,
            stats: context.stats,
            pv: Vec::new(), // TODO: Implement principal variation
            elapsed,
        }
    }
}

pub fn adjusted_depth_for_phase(base_depth: usize, phase: i32) -> usize {
    if phase <= 6 {
        base_depth + 2
    } else if phase <= 12 {
        base_depth + 1
    } else {
        base_depth
    }
}

pub fn is_insufficient_material(board: &Board) -> bool {
    // for now returns true for only king vs king, king vs lone bishop, king vs lone knight
    let white_invalid_pieces = board.pieces(Color::White, PieceType::Rook)
        | board.pieces(Color::White, PieceType::Pawn)
        | board.pieces(Color::White, PieceType::Queen);

    let black_invalid_pieces = board.pieces(Color::Black, PieceType::Rook)
        | board.pieces(Color::Black, PieceType::Pawn)
        | board.pieces(Color::Black, PieceType::Queen);

    if white_invalid_pieces | black_invalid_pieces != 0 {
        return false;
    }

    // includes only knight and bishop
    let w_knights = board.pieces(Color::White, PieceType::Knight);
    let b_knights = board.pieces(Color::Black, PieceType::Knight);

    let w_bishops = board.pieces(Color::White, PieceType::Bishop);
    let b_bishops = board.pieces(Color::Black, PieceType::Bishop);

    let knights = w_knights & b_knights;
    let bishops = w_bishops & b_bishops;

    let knight_count = knights.count_ones();
    let bishop_count = bishops.count_ones();
    let minor_count = knight_count + bishop_count;

    // king vs king.        king vs bishop/knight
    if minor_count == 0 || minor_count == 1 {
        return true;
    }

    // any knights that don't satisfy the pervious minor count can theoretically result in checkmate
    if knight_count != 0 {
        return false;
    }

    // only multiple bishops remain
    // if the bishops are on same color then it is a stalemate
    let b_bishops = bishops & BLACK_SQUARES;
    let w_bishops = bishops & WHITE_SQUARES;

    b_bishops == 0 || w_bishops == 0
}
