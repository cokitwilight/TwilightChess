use crate::bitboard::{Square, square_from_algebraic};
use crate::board::{Board, Move};
use crate::types::PieceType;

use std::fmt;

impl Move {
    pub fn to_uci(&self) -> String {
        debug_assert!(self.from() < 64, "Invalid source square: {}", self.from());
        debug_assert!(self.to() < 64, "Invalid destination square: {}", self.to());

        let mut result = String::with_capacity(if self.promotion().is_some() { 5 } else { 4 });

        push_square(&mut result, self.from());
        push_square(&mut result, self.to());

        if let Some(piece) = self.promotion() {
            let promotion_char = match piece {
                PieceType::Knight => 'n',
                PieceType::Bishop => 'b',
                PieceType::Rook => 'r',
                PieceType::Queen => 'q',

                // These should never be valid promotion choices.
                PieceType::Pawn | PieceType::King => {
                    panic!("Invalid promotion piece: {:?}", piece);
                }
            };

            result.push(promotion_char);
        }

        result
    }
}

/// Resolves long algebraic UCI notation (for example `e2e4` or `a7a8q`)
/// to the matching legal move in `board`.
///
/// Looking the move up in the legal move list is important because UCI notation
/// does not encode internal move types such as castling or en passant.
pub fn parse_uci_move(board: &Board, input: &str) -> Result<Move, String> {
    let bytes = input.as_bytes();

    if bytes.len() != 4 && bytes.len() != 5 {
        return Err(format!(
            "UCI move must contain 4 or 5 ASCII characters: `{input}`"
        ));
    }

    if !input.is_ascii() {
        return Err(format!("UCI move must be ASCII: `{input}`"));
    }

    let from = square_from_algebraic(&input[0..2])
        .map_err(|error| format!("invalid source square in `{input}`: {error}"))?;
    let to = square_from_algebraic(&input[2..4])
        .map_err(|error| format!("invalid destination square in `{input}`: {error}"))?;

    let promotion = if bytes.len() == 5 {
        Some(match bytes[4].to_ascii_lowercase() {
            b'n' => PieceType::Knight,
            b'b' => PieceType::Bishop,
            b'r' => PieceType::Rook,
            b'q' => PieceType::Queen,
            _ => return Err(format!("invalid promotion piece in `{input}`")),
        })
    } else {
        None
    };

    let mut board = board.clone();
    let legal_moves = board.all_legal_moves();

    legal_moves
        .iter()
        .copied()
        .find(|mv| mv.from() == from && mv.to() == to && mv.promotion() == promotion)
        .ok_or_else(|| format!("illegal move `{input}`"))
}

/// Compatibility helper for callers that only need an optional legal move.
pub fn find_legal_move_from_uci(board: &Board, input: &str) -> Option<Move> {
    parse_uci_move(board, input).ok()
}

impl fmt::Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_uci())
    }
}

fn push_square(output: &mut String, square: Square) {
    let file = square % 8;
    let rank = square / 8;

    output.push(char::from(b'a' + file));
    output.push(char::from(b'1' + rank));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::{MoveType, STARTPOS_FEN};

    #[test]
    fn parses_legal_normal_and_castling_moves() {
        let start = Board::from_fen(STARTPOS_FEN).unwrap();
        assert_eq!(parse_uci_move(&start, "e2e4").unwrap().to_uci(), "e2e4");

        let castle_board = Board::from_fen("r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1").unwrap();
        let castle = parse_uci_move(&castle_board, "e1g1").unwrap();
        assert_eq!(castle.kind(), MoveType::Castle);
    }

    #[test]
    fn parses_en_passant_and_promotion_moves() {
        let en_passant_board = Board::from_fen("4k3/8/8/3pP3/8/8/8/4K3 w - d6 0 1").unwrap();
        let en_passant = parse_uci_move(&en_passant_board, "e5d6").unwrap();
        assert_eq!(en_passant.kind(), MoveType::EnPassant);

        let promotion_board = Board::from_fen("7k/P7/8/8/8/8/8/4K3 w - - 0 1").unwrap();
        let promotion = parse_uci_move(&promotion_board, "a7a8q").unwrap();
        assert_eq!(promotion.promotion(), Some(PieceType::Queen));
    }

    #[test]
    fn rejects_malformed_and_illegal_moves() {
        let board = Board::from_fen(STARTPOS_FEN).unwrap();

        assert!(parse_uci_move(&board, "e2e").is_err());
        assert!(parse_uci_move(&board, "e2e9").is_err());
        assert!(parse_uci_move(&board, "e2e5").is_err());
        assert!(parse_uci_move(&board, "e2e4q").is_err());
    }
}
