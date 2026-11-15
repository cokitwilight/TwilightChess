use crate::board::Board;
use crate::engine::{Engine, MAX_PV, SearchContext, SearchResult, SearchTermination};
use crate::eval::eval::{BLACK_SQUARES, WHITE_SQUARES};
use crate::types::{Color, PieceType};

impl Engine {
    /// Search a position using a fresh context containing its limits and repetition history.
    /// Debug callers attach cancellation and finder state to the same context before calling.
    pub fn search(
        &mut self,
        board: &Board,
        context: &mut SearchContext,
        opening_allowed: bool,
        can_print: bool,
    ) -> SearchResult {
        let mut board = board.clone();

        if opening_allowed && let Some(book_mv) = self.get_book_move(&board) {
            if can_print {
                println!("Book Move. {}", book_mv);
            }

            return SearchResult {
                best_move: Some(book_mv),
                eval: 0,
                depth_reached: 0,
                seldepth: 0,
                stats: context.stats,
                pv: [None; MAX_PV],
                elapsed: context.elapsed(),
                termination: SearchTermination::BookMove,
                #[cfg(feature = "debug-uci")]
                iterations: Vec::new(),
            };
        }

        // IMPORTANT: increments generation for the transposition table
        self.tt.new_search();

        self.iterative_deepening(&mut board, context, can_print)
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

    let knights = w_knights | b_knights;
    let bishops = w_bishops | b_bishops;

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
