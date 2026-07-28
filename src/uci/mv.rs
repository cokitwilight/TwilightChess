use crate::bitboard::Square;
use crate::board::Move;
use crate::types::PieceType;

use std::fmt;

impl Move {
    pub fn to_uci(&self) -> String {
        debug_assert!(self.from < 64, "Invalid source square: {}", self.from);
        debug_assert!(self.to < 64, "Invalid destination square: {}", self.to);

        let mut result = String::with_capacity(if self.promotion.is_some() { 5 } else { 4 });

        push_square(&mut result, self.from);
        push_square(&mut result, self.to);

        if let Some(piece) = self.promotion {
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
