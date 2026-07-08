use crate::board::Board;
use crate::types::{COLORS, Color, PIECE_TYPES};

// FOR DEBUGGING
pub fn calculate_material(board: &Board) -> i32 {
    let mut material = 0;

    for color in COLORS {
        let sign = match color {
            Color::White => 1,
            Color::Black => -1,
        };
        for piece_type in PIECE_TYPES {
            let piece_bb = board.pieces(color, piece_type);
            let piece_count = piece_bb.count_ones() as i32;
            let piece_value = piece_type.value();

            material += piece_count * piece_value * sign;
        }
    }

    material
}
