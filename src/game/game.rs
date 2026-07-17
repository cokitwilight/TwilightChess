use crate::bitboard::Square;
use crate::board::{Board, Move, MoveList, STARTPOS_FEN};
use crate::game::GameState;

pub struct Game {
    pub board: Board,
    pub state: GameState,
    pub repetition_history: Vec<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveError {
    GameAlreadyOver,
    IllegalMove(Move),
}

impl Game {
    pub fn new() -> Self {
        let board = Board::from_fen(STARTPOS_FEN).unwrap();
        let state = GameState::Ongoing;
        let repetition_history: Vec<u64> = Vec::new();

        Self {
            board,
            state,
            repetition_history,
        }
    }
    pub fn game_state(&mut self) -> GameState {
        let side_to_move = self.board.side_to_move();
        let legal_moves = self.board.legal_moves(side_to_move);

        if legal_moves.is_empty() {
            if self.board.in_check(side_to_move) {
                GameState::Checkmate {
                    winner: side_to_move.opposite(),
                }
            } else {
                GameState::Stalemate
            }
        } else if self.board.halfmove_clock >= 100 {
            GameState::DrawByFiftyMoveRule
        } else {
            GameState::Ongoing
        }

        // TODO: ADD insufficient material and Repeated positions
    }

    pub fn play_move(&mut self, mv: Move) -> Result<(), MoveError> {
        if self.state != GameState::Ongoing {
            return Err(MoveError::GameAlreadyOver);
        }

        // Validate against the real legal move list.
        //
        // Important:
        // Use the legal move returned by the move generator, not blindly `mv`.
        // This helps if the UI passes a move with correct from/to but missing
        // promotion/castle/en-passant details.
        let legal_moves = self.board.legal_moves(self.board.side_to_move());

        let Some(legal_mv) = legal_moves
            .iter()
            .copied()
            .find(|candidate| moves_match_for_play(*candidate, mv))
        else {
            return Err(MoveError::IllegalMove(mv));
        };

        self.board.make_move(legal_mv);

        self.repetition_history.push(self.board.hash());

        self.update_state();

        Ok(())
    }

    pub fn legal_moves_from(&mut self, sq: Square) -> MoveList {
        let turn = self.board.side_to_move();
        self.board.legal_moves_at(sq, turn)
    }

    fn update_state(&mut self) {
        if self.is_threefold_repetition() {
            self.state = GameState::DrawByRepetition;
            return;
        }

        self.state = self.board.game_state_basic();
    }

    fn is_threefold_repetition(&self) -> bool {
        let current_hash = self.board.hash();

        self.repetition_history
            .iter()
            .filter(|&&hash| hash == current_hash)
            .count()
            >= 3
    }

    pub fn board(&self) -> &Board {
        &self.board
    }

    pub fn board_mut(&mut self) -> &mut Board {
        &mut self.board
    }

    pub fn state(&self) -> GameState {
        self.state
    }

    pub fn repetition_history(&self) -> &Vec<u64> {
        &self.repetition_history
    }
}

// HELPER
fn moves_match_for_play(legal: Move, requested: Move) -> bool {
    legal.from == requested.from
        && legal.to == requested.to
        && legal.kind == requested.kind
        && legal.promotion == requested.promotion
}
