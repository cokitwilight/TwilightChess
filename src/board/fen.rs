use crate::bitboard::{Square, square_from_algebraic};
use crate::board::Board;
use crate::types::{Color, PieceType};

pub const STARTPOS_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

pub const WHITE_KINGSIDE: u8 = 0b0001;
pub const WHITE_QUEENSIDE: u8 = 0b0010;
pub const BLACK_KINGSIDE: u8 = 0b0100;
pub const BLACK_QUEENSIDE: u8 = 0b1000;

// ****************************
// **** FEN IMPLEMENTATION ****
// ****************************

impl Board {
    pub fn from_fen(fen: &str) -> Result<Board, String> {
        let parts: Vec<&str> = fen.split_whitespace().collect();

        if parts.len() != 6 {
            return Err(format!(
                "FEN must have 6 fields, found {}: {}",
                parts.len(),
                fen
            ));
        }

        let placement = parts[0];
        let side = parts[1];
        let castling = parts[2];
        let ep = parts[3];
        let halfmove = parts[4];
        let fullmove = parts[5];

        let mut board = Board::empty();

        // -------------------------
        // Piece placement
        // -------------------------
        let mut rank: i32 = 7;
        let mut file: i32 = 0;

        for ch in placement.chars() {
            match ch {
                '/' => {
                    if file != 8 {
                        return Err(format!("Invalid FEN rank width before '/': {}", placement));
                    }

                    rank -= 1;
                    file = 0;

                    if rank < 0 {
                        return Err(format!("Too many ranks in FEN: {}", placement));
                    }
                }

                '1'..='8' => {
                    let empty_count = ch.to_digit(10).unwrap() as i32;
                    file += empty_count;

                    if file > 8 {
                        return Err(format!("Too many files in FEN rank: {}", placement));
                    }
                }

                piece_char => {
                    let Some((color, kind)) = piece_from_fen_char(piece_char) else {
                        return Err(format!("Invalid piece char in FEN: {}", piece_char));
                    };

                    if file >= 8 || rank < 0 {
                        return Err(format!("Piece out of board in FEN: {}", placement));
                    }

                    let sq = (rank * 8 + file) as Square;
                    board.add_piece(color, kind, sq);

                    file += 1;
                }
            }
        }

        if rank != 0 || file != 8 {
            return Err(format!("Invalid FEN board shape: {}", placement));
        }

        // -------------------------
        // Side to move
        // -------------------------
        board.side_to_move = parse_side_to_move(side)?;

        // -------------------------
        // Castling rights
        // -------------------------
        board.castling_rights = parse_castling_rights(castling)?;

        // -------------------------
        // En passant
        // -------------------------
        board.en_passant = if ep == "-" {
            None
        } else {
            Some(square_from_algebraic(ep)?)
        };

        // -------------------------
        // Clocks
        // -------------------------
        board.halfmove_clock = halfmove
            .parse::<u16>()
            .map_err(|_| format!("Invalid halfmove clock: {}", halfmove))?;

        board.fullmove_number = fullmove
            .parse::<u16>()
            .map_err(|_| format!("Invalid fullmove number: {}", fullmove))?;

        board.hash = board.compute_hash_from_scratch();

        board.assert_hash();

        // TODO: Fix this later to not use clone
        board.update_material_and_phase();

        Ok(board)
    }
}

// *********************
// **** FEN HELPERS ****
// *********************

fn piece_from_fen_char(ch: char) -> Option<(Color, PieceType)> {
    let color = if ch.is_ascii_uppercase() {
        Color::White
    } else {
        Color::Black
    };

    let kind = match ch.to_ascii_lowercase() {
        'p' => PieceType::Pawn,
        'n' => PieceType::Knight,
        'b' => PieceType::Bishop,
        'r' => PieceType::Rook,
        'q' => PieceType::Queen,
        'k' => PieceType::King,
        _ => return None,
    };

    Some((color, kind))
}

fn parse_side_to_move(s: &str) -> Result<Color, String> {
    match s {
        "w" => Ok(Color::White),
        "b" => Ok(Color::Black),
        _ => Err(format!("Invalid side to move: {}", s)),
    }
}
fn parse_castling_rights(s: &str) -> Result<u8, String> {
    if s == "-" {
        return Ok(0);
    }

    let mut rights = 0u8;

    for ch in s.chars() {
        match ch {
            'K' => rights |= WHITE_KINGSIDE,
            'Q' => rights |= WHITE_QUEENSIDE,
            'k' => rights |= BLACK_KINGSIDE,
            'q' => rights |= BLACK_QUEENSIDE,
            _ => return Err(format!("Invalid castling rights char: {}", ch)),
        }
    }

    Ok(rights)
}

#[cfg(test)]
mod tests {
    use crate::board::{Board, MoveType};
    use crate::eval::material::calculate_material;
    use crate::eval::phase::calculate_phase;
    use crate::eval::pst::{eg_pst_bonus, mg_pst_bonus};

    fn assert_material_matches_recompute(board: &Board) {
        assert_eq!(
            board.material(),
            calculate_material(board),
            "cached material does not match full recomputation"
        );
    }

    fn assert_phase_matches_recompute(board: &Board) {
        assert_eq!(
            board.phase(),
            calculate_phase(board),
            "cached phase does not match full recomputation"
        );
    }

    fn assert_mg_pst_matches_recompute(board: &Board) {
        assert_eq!(
            board.mg_pst(),
            mg_pst_bonus(board),
            "cached middlegame PST does not match full recomputation"
        );
    }

    fn assert_eg_pst_matches_recompute(board: &Board) {
        assert_eq!(
            board.eg_pst(),
            eg_pst_bonus(board),
            "cached endgame PST does not match full recomputation"
        );
    }

    fn assert_eval_state_matches_recompute(board: &Board) {
        assert_material_matches_recompute(board);
        assert_phase_matches_recompute(board);
        assert_mg_pst_matches_recompute(board);
        assert_eg_pst_matches_recompute(board);
    }

    const TEST_FENS: &[&str] = &[
        // Starting position
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
        // Kiwipete: castling rights, mixed pieces, tactical position
        "r3k2r/p1ppqpb1/bn2pnp1/2pPN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
        // Simple castling test position
        "r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1",
        // En passant available: white can play e5xd6 en passant
        "4k3/8/8/3pP3/8/8/8/4K3 w - d6 0 1",
        // Promotion available
        "7k/P7/8/8/8/8/7p/4K3 w - - 0 1",
        // More imbalanced material
        "2r2rk1/pp3ppp/2n1pn2/q2p4/3P4/2PBPN2/PPQ2PPP/R4RK1 w - - 0 1",
    ];

    #[test]
    fn from_fen_initializes_eval_state_correctly() {
        for fen in TEST_FENS {
            let board = Board::from_fen(fen).unwrap();

            assert_eval_state_matches_recompute(&board);
        }
    }

    #[test]
    fn make_and_undo_preserve_incremental_eval_state() {
        for fen in TEST_FENS {
            let mut board = Board::from_fen(fen).unwrap();

            assert_eval_state_matches_recompute(&board);

            let original_material = board.material();
            let original_phase = board.phase();
            let original_mg_pst = board.mg_pst();
            let original_eg_pst = board.eg_pst();

            let moves = board.legal_moves(board.side_to_move());

            for mv in moves.iter().copied() {
                let undo = board.make_move(mv);

                assert_eval_state_matches_recompute(&board);

                board.undo_move(undo);

                assert_eval_state_matches_recompute(&board);

                assert_eq!(
                    board.material(),
                    original_material,
                    "material was not restored after undo"
                );
                assert_eq!(
                    board.phase(),
                    original_phase,
                    "phase was not restored after undo"
                );
                assert_eq!(
                    board.mg_pst(),
                    original_mg_pst,
                    "middlegame PST was not restored after undo"
                );
                assert_eq!(
                    board.eg_pst(),
                    original_eg_pst,
                    "endgame PST was not restored after undo"
                );
            }
        }
    }

    #[test]
    fn special_moves_update_incremental_eval_correctly() {
        let special_positions = [
            (
                "castling",
                "r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1",
                MoveType::Castle,
            ),
            (
                "en passant",
                "4k3/8/8/3pP3/8/8/8/4K3 w - d6 0 1",
                MoveType::EnPassant,
            ),
            (
                "promotion",
                "7k/P7/8/8/8/8/7p/4K3 w - - 0 1",
                MoveType::Normal,
            ),
        ];

        for (name, fen, expected_kind) in special_positions {
            let mut board = Board::from_fen(fen).unwrap();
            let moves = board.legal_moves(board.side_to_move());

            let mut saw_expected_move = false;

            for mv in moves.iter().copied() {
                if mv.kind() != expected_kind {
                    continue;
                }

                saw_expected_move = true;

                let undo = board.make_move(mv);

                assert_eval_state_matches_recompute(&board);

                board.undo_move(undo);

                assert_eval_state_matches_recompute(&board);
            }

            assert!(
                saw_expected_move,
                "test position did not generate expected special move: {name}"
            );
        }
    }
}
