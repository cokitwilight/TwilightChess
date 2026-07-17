use crate::board::{Board, Move, MoveList, MoveType};
use crate::engine::Engine;
use crate::engine::SearchContext;
use crate::engine::ordering::see;
use crate::types::{Color, PieceType};

impl Engine {
    pub fn order_moves(
        &self,
        board: &Board,
        moves: &mut MoveList,
        side_to_move: Color,
        ply: usize,
        context: &mut SearchContext,
        previous_best_move: Option<Move>,
        tt_best_move: Option<Move>,
    ) {
        moves.sort_by_score(|mv| {
            self.move_order_score(
                board,
                mv,
                side_to_move,
                ply,
                context,
                previous_best_move,
                tt_best_move,
            )
        });
    }

    pub fn q_order_moves(&self, board: &Board, moves: &mut MoveList, tt_best_move: Option<Move>) {
        moves.sort_by_score(|mv| self.q_move_order_score(board, mv, tt_best_move));
    }

    pub fn move_order_score(
        &self,
        board: &Board,
        mv: Move,
        side_to_move: Color,
        ply: usize,
        context: &SearchContext,
        previous_best_move: Option<Move>,
        tt_best_move: Option<Move>,
    ) -> i32 {
        if Some(mv) == previous_best_move {
            return 2_000_000;
        }
        if Some(mv) == tt_best_move {
            return 1_500_000;
        }

        let is_capture = matches!(mv.kind, MoveType::Capture | MoveType::EnPassant);

        if is_capture {
            let see_score = see(board, mv);
            let promo_bonus = mv.promotion.map_or(0, promotion_score);

            return if see_score >= 0 {
                // Winning/equal captures: one tier, ranked by SEE (+ promo bonus for capture-promotions)
                900_000 + see_score + promo_bonus
            } else {
                // Losing captures: still below killers/history, ranked so "least bad" goes first
                -600_000 + see_score
            };
        }

        // Quiet promotions (no capture involved)
        if let Some(promo) = mv.promotion {
            // Queen promotions are strong enough to rank with good captures;
            // under-promotions are almost always worse than a quiet move and
            // should sit low unless you have specific tactical reasons to boost them
            return match promo {
                PieceType::Queen => 800_000 + promotion_score(promo),
                _ => -700_000 + promotion_score(promo),
            };
        }

        if context.killer_moves.contains(ply, mv) {
            return 700_000;
        }

        self.history.get(side_to_move, mv.from, mv.to).min(500_000)
    }
    pub fn q_move_order_score(&self, board: &Board, mv: Move, tt_best_move: Option<Move>) -> i32 {
        if Some(mv) == tt_best_move {
            return 1_500_000;
        }

        // qsearch move lists are captures/promotions only by construction —
        // no killers, no history, no quiet-move tier needed
        let see_score = see(board, mv);
        let promo_bonus = mv.promotion.map_or(0, promotion_score);

        see_score + promo_bonus
    }
}

#[allow(dead_code)]
fn mvv_lva_score(board: &Board, mv: Move) -> i32 {
    let attacker = board
        .piece_at(mv.from)
        .expect("move_order_score called with no attacker on mv.from")
        .kind;

    let victim = match mv.kind {
        MoveType::Capture => {
            board
                .piece_at(mv.to)
                .expect("capture move has no victim on mv.to")
                .kind
        }
        MoveType::EnPassant => PieceType::Pawn,

        _ => return 0,
    };

    let victim_value = mvv_lva_piece_value(victim);
    let attacker_value = mvv_lva_piece_value(attacker);

    // Main MVV-LVA idea:
    //
    // Higher victim value = better.
    // Lower attacker value = better.
    //
    // Multiply victim value so victim importance dominates attacker penalty.
    let mut score = victim_value * 10 - attacker_value;

    if let Some(promo) = mv.promotion {
        score += promotion_score(promo);
    }

    score
}

fn promotion_score(piece: PieceType) -> i32 {
    match piece {
        PieceType::Queen => 8_000,
        PieceType::Rook => 4_000,
        PieceType::Bishop => 3_000,
        PieceType::Knight => 3_000,
        PieceType::Pawn => 0,
        PieceType::King => 0,
    }
}

#[allow(dead_code)]
fn mvv_lva_piece_value(piece: PieceType) -> i32 {
    match piece {
        PieceType::Pawn => 100,
        PieceType::Knight => 300,
        PieceType::Bishop => 300,
        PieceType::Rook => 500,
        PieceType::Queen => 900,
        PieceType::King => 10_000,
    }
}
