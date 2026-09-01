use crate::board::{Board, Move, MoveType};
use crate::engine::SearchContext;
use crate::engine::history::{HistoryKey, HistoryTables};
use crate::engine::ordering::see;
use crate::types::{Color, PieceType};

pub fn move_order_score(
    board: &Board,
    mv: Move,
    side_to_move: Color,
    ply: usize,
    context: &SearchContext,
    history: &HistoryTables,
    previous_best_move: Option<Move>,
    tt_best_move: Option<Move>,
    history_key: Option<HistoryKey>,
    see_value: Option<i32>,
    captured_piece: Option<PieceType>,
) -> i32 {
    if Some(mv) == previous_best_move {
        return 2_000_000;
    }
    if Some(mv) == tt_best_move {
        return 1_500_000;
    }

    let is_capture = matches!(mv.kind(), MoveType::Capture | MoveType::EnPassant);

    if is_capture {
        let see_score = if let Some(value) = see_value {
            value
        } else {
            let victim = if let Some(v) = captured_piece {
                Some(v)
            } else {
                board.piecetype_at(mv.to())
            };
            see(board, mv, victim)
        };
        let promo_bonus = mv.promotion().map_or(0, promotion_score);

        return if see_score >= 0 {
            // Winning/equal captures: one tier, ranked by SEE (+ promo bonus for capture-promotions)
            900_000 + see_score + promo_bonus
        } else {
            // Losing captures: still below killers/history, ranked so "least bad" goes first
            -600_000 + see_score
        };
        // FOR BENCHMARKING
        // return 1_000_000;
    }

    // Quiet promotions (no capture involved)
    if let Some(promo) = mv.promotion() {
        // Queen promotions are strong enough to rank with good captures;
        // under-promotions are almost always worse than a quiet move
        return match promo {
            PieceType::Queen => 800_000 + promotion_score(promo),
            _ => -500_000 + promotion_score(promo),
        };
    }

    if context.killer_moves.contains(ply, mv) {
        return 700_000;
    }

    let curr_key = if let Some(key) = history_key {
        key
    } else {
        let piece = board
            .piecetype_at(mv.from())
            .expect("No piece in board in move ordering!");

        HistoryKey::new(side_to_move, piece, mv.to())
    };

    history.get_quiet_score(&context.stack, ply, curr_key)
}

pub fn promotion_score(piece: PieceType) -> i32 {
    match piece {
        PieceType::Queen => 8_000,
        PieceType::Rook => 4_000,
        PieceType::Bishop => 3_000,
        PieceType::Knight => 3_000,
        PieceType::Pawn => 0,
        PieceType::King => 0,
    }
}
