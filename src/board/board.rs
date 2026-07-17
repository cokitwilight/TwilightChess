use crate::bitboard::{
    Bitboard, Square, bishop_attacks, bit, king_attacks, knight_attacks, pawn_attacks_from_square,
    rook_attacks, square_to_algebraic,
};
use crate::board::MoveList;
use crate::eval::phase::MAX_PHASE;
use crate::game::GameState;
use crate::moves::legal::{all_legal_capture_moves, all_legal_moves_at};
use crate::moves::pseudo::all_pseudo_capture_moves;
use crate::moves::{all_legal_moves, all_pseudo_moves};
use crate::types::{COLORS, Color, PIECE_TYPES, Piece, PieceType};

pub const WHITE_KINGSIDE: u8 = 0b0001;
pub const WHITE_QUEENSIDE: u8 = 0b0010;
pub const BLACK_KINGSIDE: u8 = 0b0100;
pub const BLACK_QUEENSIDE: u8 = 0b1000;

#[derive(Clone, PartialEq, Eq)]
pub struct Board {
    pub pieces: [[Bitboard; 6]; 2],
    pub occupancy: [Bitboard; 2],
    pub all_occupancy: Bitboard,

    pub side_to_move: Color,

    pub castling_rights: u8,

    pub en_passant: Option<Square>,

    pub halfmove_clock: u16,

    pub fullmove_number: u16,

    pub hash: u64,
    pub material: i32,

    pub phase: i32,

    pub mg_pst: i32,

    pub eg_pst: i32,
}

impl Board {
    // **********************
    // **** DECLARATIONS ****
    // **********************
    pub fn empty() -> Self {
        Self {
            pieces: [[0; 6]; 2],
            occupancy: [0; 2],
            all_occupancy: 0,

            side_to_move: Color::White,

            castling_rights: 0,
            en_passant: None,
            halfmove_clock: 0,
            fullmove_number: 1,
            hash: 0,
            material: 0, // in centi pawns
            phase: 24,   // 24 is the default
            mg_pst: 0,
            eg_pst: 0,
        }
    }

    // FOR COPY
    // ********************
    // **** BLANK_COPY ****
    // ********************

    // *****************
    // **** GETTERS ****
    // *****************

    #[inline]
    pub fn pieces(&self, color: Color, piece: PieceType) -> Bitboard {
        // returns the bitboard for the exact color and piece type
        self.pieces[color as usize][piece as usize]
    }
    pub fn all_pieces(&self) -> [[Bitboard; 6]; 2] {
        self.pieces
    }
    #[inline]
    pub fn occupancy_of(&self, color: Color) -> Bitboard {
        self.occupancy[color.idx()]
    }

    #[inline]
    pub fn all_occupancy(&self) -> Bitboard {
        self.all_occupancy
    }

    #[inline]
    pub fn side_to_move(&self) -> Color {
        self.side_to_move
    }

    pub fn has_castling_right(&self, right: u8) -> bool {
        self.castling_rights & right != 0
    }

    pub fn en_passant(&self) -> Option<Square> {
        self.en_passant
    }

    #[inline]
    pub fn hash(&self) -> u64 {
        self.hash
    }

    pub fn material(&self) -> i32 {
        self.material
    }

    pub fn phase(&self) -> i32 {
        self.phase
    }

    pub fn mg_pst(&self) -> i32 {
        self.mg_pst
    }

    pub fn eg_pst(&self) -> i32 {
        self.eg_pst
    }

    pub fn halfmove_clock(&self) -> u16 {
        self.halfmove_clock
    }

    // *****************
    // **** SETTERS ****
    // *****************

    pub fn add_piece(&mut self, color: Color, piece: PieceType, sq: Square) {
        let m = bit(sq);

        debug_assert_eq!(
            self.all_occupancy & m,
            0,
            "Tried to add piece to occupied square {}",
            sq
        );

        self.pieces[color.idx()][piece.idx()] |= m;
        self.occupancy[color.idx()] |= m;
        self.all_occupancy |= m;
    }

    pub fn remove_piece(&mut self, color: Color, piece: PieceType, sq: Square) {
        let m = bit(sq);

        debug_assert_ne!(
            self.pieces[color.idx()][piece.idx()] & m,
            0,
            "Tried to remove missing {:?} {:?} from square {}",
            color,
            piece,
            sq
        );

        self.pieces[color.idx()][piece.idx()] &= !m;
        self.occupancy[color.idx()] &= !m;
        self.all_occupancy &= !m;
    }

    pub fn update_material_and_phase(&mut self) {
        let mut material = 0;
        let mut phase = 0;

        for color in COLORS {
            let sign = match color {
                Color::White => 1,
                Color::Black => -1,
            };

            let pawns = self.pieces(color, PieceType::Pawn).count_ones() as i32;
            let knights = self.pieces(color, PieceType::Knight).count_ones() as i32;
            let bishops = self.pieces(color, PieceType::Bishop).count_ones() as i32;
            let rooks = self.pieces(color, PieceType::Rook).count_ones() as i32;
            let queens = self.pieces(color, PieceType::Queen).count_ones() as i32;

            material +=
                sign * (100 * pawns + 310 * knights + 330 * bishops + 500 * rooks + 900 * queens);

            phase += knights + bishops + 2 * rooks + 4 * queens;
        }

        phase = phase.min(MAX_PHASE);

        let mg_pst_bonus = crate::eval::pst::mg_pst_bonus(self);
        let eg_pst_bonus = crate::eval::pst::eg_pst_bonus(self);

        self.material = material;
        self.phase = phase;
        self.mg_pst = mg_pst_bonus;
        self.eg_pst = eg_pst_bonus;
    }
    // *************************
    // **** MOVE GENERATION ****
    // *************************

    pub fn legal_moves(&mut self, color: Color) -> MoveList {
        let mut legal_moves = MoveList::new();
        all_legal_moves(self, color, &mut legal_moves);
        legal_moves
    }

    pub fn all_legal_moves(&mut self) -> MoveList {
        let mut legal_moves = MoveList::new();
        all_legal_moves(self, self.side_to_move, &mut legal_moves);
        legal_moves
    }

    pub fn all_pseudo_moves(&mut self) -> MoveList {
        let mut pseudo_moves = MoveList::new();
        all_pseudo_moves(self, self.side_to_move, &mut pseudo_moves);
        pseudo_moves
    }

    pub fn all_legal_capture_moves(&mut self) -> MoveList {
        let mut captures = MoveList::new();
        all_legal_capture_moves(self, self.side_to_move, &mut captures);
        captures
    }

    pub fn all_pseudo_capture_moves(&mut self) -> MoveList {
        let mut pseudo_moves = MoveList::new();
        all_pseudo_capture_moves(self, self.side_to_move, &mut pseudo_moves);
        pseudo_moves
    }

    pub fn legal_moves_at(&mut self, sq: Square, color: Color) -> MoveList {
        let mut legal_moves = MoveList::new();
        all_legal_moves_at(self, color, sq, &mut legal_moves);
        legal_moves
    }

    pub fn all_legal_moves_at(&mut self, sq: Square) -> MoveList {
        let mut legal_moves = MoveList::new();
        all_legal_moves_at(self, self.side_to_move, sq, &mut legal_moves);
        legal_moves
    }
    // ********************
    // **** GAME STATE ****
    // ********************

    pub fn game_state_basic(&mut self) -> GameState {
        let side_to_move = self.side_to_move;
        let legal_moves = self.legal_moves(side_to_move);

        if legal_moves.is_empty() {
            if self.in_check(side_to_move) {
                return GameState::Checkmate {
                    winner: side_to_move.opposite(),
                };
            } else {
                return GameState::Stalemate;
            }
        }

        if self.halfmove_clock >= 100 {
            return GameState::DrawByFiftyMoveRule;
        }

        // if self.insufficient_material() {
        //     return GameState::DrawByInsufficientMaterial;
        // }

        GameState::Ongoing
    }

    // *************************
    // **** GENERAL PURPOSE ****
    // *************************

    pub fn in_check(&self, color: Color) -> bool {
        let king = self.pieces[color.idx()][PieceType::King.idx()];
        debug_assert!(king != 0, "No king found for {:?}", color);

        debug_assert!(
            king.count_ones() == 1,
            "Expected exactly one king for {:?}, found {}",
            color,
            king.count_ones()
        );

        let king_sq = king.trailing_zeros() as Square;
        self.square_attacked_by(king_sq, color.opposite())
    }

    pub fn square_attacked_by(&self, sq: Square, by: Color) -> bool {
        let occupied = self.all_occupancy();

        let pawns = self.pieces(by, PieceType::Pawn);
        let knights = self.pieces(by, PieceType::Knight);
        let bishops = self.pieces(by, PieceType::Bishop);
        let rooks = self.pieces(by, PieceType::Rook);
        let queens = self.pieces(by, PieceType::Queen);
        let king = self.pieces(by, PieceType::King);

        let pawn_attackers = match by {
            Color::White => pawn_attacks_from_square(sq, Color::Black) & pawns,
            Color::Black => pawn_attacks_from_square(sq, Color::White) & pawns,
        };

        if pawn_attackers != 0 {
            return true;
        }

        // -------------------------
        // Knights
        // -------------------------
        if knight_attacks(sq) & knights != 0 {
            return true;
        }

        // -------------------------
        // Kings
        // -------------------------
        if king_attacks(sq) & king != 0 {
            return true;
        }

        // -------------------------
        // Bishops / Queens
        // -------------------------
        let diagonal_attackers = bishops | queens;

        if bishop_attacks(sq, occupied) & diagonal_attackers != 0 {
            return true;
        }
        // -------------------------
        // Rooks / Queens
        // -------------------------
        let straight_attackers = rooks | queens;

        if rook_attacks(sq, occupied) & straight_attackers != 0 {
            return true;
        }

        false
    }

    pub fn piece_at(&self, sq: Square) -> Option<Piece> {
        // for debugging and stuff now
        let m = bit(sq);

        for color in COLORS {
            for kind in PIECE_TYPES {
                if self.pieces(color, kind) & m != 0 {
                    return Some(Piece { color, kind });
                }
            }
        }

        None
    }
    pub fn print_board(&self) {
        println!("  +-----------------+");

        for rank in (0..8).rev() {
            print!("{} |", rank + 1);

            for file in 0..8 {
                let sq = (rank * 8 + file) as Square;

                match self.piece_at(sq) {
                    Some(piece) => {
                        print!(" {}", Self::piece_to_char(piece));
                    }
                    None => {
                        print!(" .");
                    }
                }
            }

            println!(" |");
        }

        println!("  +-----------------+");
        println!("    a b c d e f g h");
        println!("Side: {:?}", self.side_to_move);
        println!("Castling: {}", self.castling_rights_string());
        println!("En passant: {:?}", self.en_passant.map(square_to_algebraic));
    }

    // *************************
    // **** GENERAL HELPERS ****
    // *************************

    fn piece_to_char(piece: Piece) -> char {
        let ch = match piece.kind {
            PieceType::Pawn => 'p',
            PieceType::Knight => 'n',
            PieceType::Bishop => 'b',
            PieceType::Rook => 'r',
            PieceType::Queen => 'q',
            PieceType::King => 'k',
        };

        match piece.color {
            Color::White => ch.to_ascii_uppercase(),
            Color::Black => ch,
        }
    }
    fn castling_rights_string(&self) -> String {
        let mut s = String::new();

        if self.castling_rights & WHITE_KINGSIDE != 0 {
            s.push('K');
        }
        if self.castling_rights & WHITE_QUEENSIDE != 0 {
            s.push('Q');
        }
        if self.castling_rights & BLACK_KINGSIDE != 0 {
            s.push('k');
        }
        if self.castling_rights & BLACK_QUEENSIDE != 0 {
            s.push('q');
        }

        if s.is_empty() {
            s.push('-');
        }

        s
    }

    // *******************
    // **** DEBUGGING ****
    // *******************

    pub fn assert_hash(&self) {
        debug_assert_eq!(
            self.hash,
            self.compute_hash_from_scratch(),
            "Stored hash does not match computed hash from scratch!."
        );
    }
}

// ********************
// **** UNIT TESTS ****
// ********************

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::{Move, MoveType, STARTPOS_FEN};
    use crate::types::*;

    // *********************
    // **** BOARD STATE ****
    // *********************
    #[test]
    pub fn assert_valid() {
        // checks if there are any overlapping pieces. Use this in make_move/any changing board states
        let board = Board::from_fen(STARTPOS_FEN).expect("From fen produced None.");
        let mut seen = 0u64;

        for color in [Color::White, Color::Black] {
            let mut color_occ = 0u64;

            for piece in [
                PieceType::Pawn,
                PieceType::Knight,
                PieceType::Bishop,
                PieceType::Rook,
                PieceType::Queen,
                PieceType::King,
            ] {
                let bb = board.pieces(color, piece);

                debug_assert_eq!(
                    seen & bb,
                    0,
                    "Overlapping pieces detected for {:?} {:?}",
                    color,
                    piece
                );

                seen |= bb;
                color_occ |= bb;
            }

            debug_assert_eq!(
                color_occ,
                board.occupancy[color.idx()],
                "Bad cached occupancy for {:?}",
                color
            );
        }

        debug_assert_eq!(
            board.occupancy[Color::White.idx()] | board.occupancy[Color::Black.idx()],
            board.all_occupancy,
            "Bad all_occupancy"
        );

        debug_assert_eq!(
            board.pieces(Color::White, PieceType::King).count_ones(),
            1,
            "White must have exactly one king"
        );

        debug_assert_eq!(
            board.pieces(Color::Black, PieceType::King).count_ones(),
            1,
            "Black must have exactly one king"
        );
        debug_assert_eq!(
            board.hash,
            board.compute_hash_from_scratch(),
            "Board hash mismatch"
        );
    }

    // ***********************
    // **** BOARD HASHING ****
    // ***********************
    #[test]
    fn startpos_hash_is_stable() {
        let b1 = Board::from_fen(STARTPOS_FEN).unwrap();
        let b2 = Board::from_fen(STARTPOS_FEN).unwrap();

        assert_eq!(b1.hash(), b2.hash());
        assert_eq!(b1.hash(), b1.compute_hash_from_scratch());
    }
    #[test]
    fn side_to_move_changes_hash() {
        let white = Board::from_fen("8/8/8/8/8/8/8/4K2k w - - 0 1").unwrap();

        let black = Board::from_fen("8/8/8/8/8/8/8/4K2k b - - 0 1").unwrap();

        assert_ne!(white.hash(), black.hash());
    }
    #[test]
    fn castling_rights_change_hash() {
        let no_castle = Board::from_fen("r3k2r/8/8/8/8/8/8/R3K2R w - - 0 1").unwrap();

        let castle = Board::from_fen("r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1").unwrap();

        assert_ne!(no_castle.hash(), castle.hash());
    }
    #[test]
    fn en_passant_changes_hash() {
        let no_ep = Board::from_fen("8/8/8/8/4Pp2/8/8/4K2k w - - 0 1").unwrap();

        let ep = Board::from_fen("8/8/8/8/4Pp2/8/8/4K2k w - f6 0 1").unwrap();

        assert_ne!(no_ep.hash(), ep.hash());
    }
    #[test]
    fn piece_square_changes_hash() {
        let b1 = Board::from_fen("8/8/8/8/8/8/4N3/4K2k w - - 0 1").unwrap();

        let b2 = Board::from_fen("8/8/8/8/8/8/5N2/4K2k w - - 0 1").unwrap();

        assert_ne!(b1.hash(), b2.hash());
    }

    // ************************
    // **** MAKE_MOVE/UNDO ****
    // ************************

    fn sq(file: u8, rank: u8) -> Square {
        rank * 8 + file
    }

    fn d5() -> Square {
        sq(3, 4)
    }
    fn d6() -> Square {
        sq(3, 5)
    }
    fn e1() -> Square {
        sq(4, 0)
    }
    fn e2() -> Square {
        sq(4, 1)
    }
    fn e3() -> Square {
        sq(4, 2)
    }
    fn e4() -> Square {
        sq(4, 3)
    }
    fn e5() -> Square {
        sq(4, 4)
    }
    fn f1() -> Square {
        sq(5, 0)
    }
    fn f6() -> Square {
        sq(5, 5)
    }
    fn g1() -> Square {
        sq(6, 0)
    }
    fn g8() -> Square {
        sq(6, 7)
    }
    fn h1() -> Square {
        sq(7, 0)
    }
    fn h2() -> Square {
        sq(7, 1)
    }
    fn h8() -> Square {
        sq(7, 7)
    }

    fn a7() -> Square {
        sq(0, 6)
    }
    fn a8() -> Square {
        sq(0, 7)
    }
    fn b8() -> Square {
        sq(1, 7)
    }

    // If your Board::from_fen returns Result<Board, _>, change this to:
    //
    // Board::from_fen(fen).unwrap()
    //
    fn board_from_fen(fen: &str) -> Board {
        Board::from_fen(fen).unwrap()
    }

    #[derive(Clone)]
    struct BoardSnapshot {
        pieces: [[Bitboard; 6]; 2],
        occupancy: [Bitboard; 2],
        all_occupancy: Bitboard,
        side_to_move: Color,
        castling_rights: u8,
        en_passant: Option<Square>,
        halfmove_clock: u16,
        fullmove_number: u16,
        hash: u64,
    }

    fn snapshot(board: &Board) -> BoardSnapshot {
        BoardSnapshot {
            pieces: board.pieces,
            occupancy: board.occupancy,
            all_occupancy: board.all_occupancy,
            side_to_move: board.side_to_move,
            castling_rights: board.castling_rights,
            en_passant: board.en_passant,
            halfmove_clock: board.halfmove_clock,
            fullmove_number: board.fullmove_number,
            hash: board.hash,
        }
    }

    fn assert_restored(board: &Board, before: &BoardSnapshot) {
        assert_eq!(board.pieces, before.pieces);
        assert_eq!(board.occupancy, before.occupancy);
        assert_eq!(board.all_occupancy, before.all_occupancy);
        assert_eq!(board.side_to_move, before.side_to_move);
        assert_eq!(board.castling_rights, before.castling_rights);
        assert_eq!(board.en_passant, before.en_passant);
        assert_eq!(board.halfmove_clock, before.halfmove_clock);
        assert_eq!(board.fullmove_number, before.fullmove_number);
        assert_eq!(board.hash, before.hash);

        board.assert_hash();
    }

    fn assert_piece(board: &Board, sq: Square, expected: Option<Piece>) {
        let got = board.piece_at(sq);

        if got != expected {
            board.print_board();
            panic!(
                "Wrong piece on square {}. Expected {:?}, got {:?}",
                square_to_algebraic(sq),
                expected,
                got
            );
        }
    }

    fn make_check_undo<F>(mut board: Board, mv: Move, check_after_make: F)
    where
        F: FnOnce(&Board),
    {
        board.assert_hash();

        let before = snapshot(&board);
        let undo = board.make_move(mv);

        board.assert_hash();
        check_after_make(&board);

        board.undo_move(undo);

        assert_restored(&board, &before);
    }

    #[test]
    fn make_and_undo_double_pawn_push_sets_en_passant() {
        let board = board_from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");

        let mv = Move {
            from: e2(),
            to: e4(),
            kind: MoveType::Normal,
            promotion: None,
        };

        make_check_undo(board, mv, |board| {
            assert_piece(board, e2(), None);
            assert_piece(
                board,
                e4(),
                Some(Piece {
                    color: Color::White,
                    kind: PieceType::Pawn,
                }),
            );

            assert_eq!(board.side_to_move, Color::Black);
            assert_eq!(board.en_passant, Some(e3()));
            assert_eq!(board.halfmove_clock, 0);
            assert_eq!(board.fullmove_number, 1);
        });
    }

    #[test]
    fn make_and_undo_quiet_piece_move_clears_en_passant_and_increments_halfmove() {
        let board = board_from_fen("rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1");

        let mv = Move {
            from: g8(),
            to: f6(),
            kind: MoveType::Normal,
            promotion: None,
        };

        make_check_undo(board, mv, |board| {
            assert_piece(board, g8(), None);
            assert_piece(
                board,
                f6(),
                Some(Piece {
                    color: Color::Black,
                    kind: PieceType::Knight,
                }),
            );

            assert_eq!(board.side_to_move, Color::White);
            assert_eq!(board.en_passant, None);
            assert_eq!(board.halfmove_clock, 1);

            // Fullmove number increments after Black moves.
            assert_eq!(board.fullmove_number, 2);
        });
    }

    #[test]
    fn make_and_undo_capture_removes_captured_piece_and_resets_halfmove() {
        let board = board_from_fen("8/8/8/3p4/4P3/8/8/4K2k w - - 7 15");

        let mv = Move {
            from: e4(),
            to: d5(),
            kind: MoveType::Capture,
            promotion: None,
        };

        make_check_undo(board, mv, |board| {
            assert_piece(board, e4(), None);
            assert_piece(
                board,
                d5(),
                Some(Piece {
                    color: Color::White,
                    kind: PieceType::Pawn,
                }),
            );

            assert_eq!(board.side_to_move, Color::Black);
            assert_eq!(board.en_passant, None);
            assert_eq!(board.halfmove_clock, 0);
            assert_eq!(board.fullmove_number, 15);
        });
    }

    #[test]
    fn make_and_undo_en_passant_capture() {
        let board = board_from_fen("8/8/8/3pP3/8/8/8/4K2k w - d6 0 1");

        let mv = Move {
            from: e5(),
            to: d6(),
            kind: MoveType::EnPassant,
            promotion: None,
        };

        make_check_undo(board, mv, |board| {
            assert_piece(board, e5(), None);

            // White pawn lands on d6.
            assert_piece(
                board,
                d6(),
                Some(Piece {
                    color: Color::White,
                    kind: PieceType::Pawn,
                }),
            );

            // The captured black pawn was on d5, not d6.
            assert_piece(board, d5(), None);

            assert_eq!(board.side_to_move, Color::Black);
            assert_eq!(board.en_passant, None);
            assert_eq!(board.halfmove_clock, 0);
        });
    }

    #[test]
    fn make_and_undo_white_kingside_castle_moves_rook_too() {
        let board = board_from_fen("r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 3 1");

        let mv = Move {
            from: e1(),
            to: g1(),
            kind: MoveType::Castle,
            promotion: None,
        };

        make_check_undo(board, mv, |board| {
            assert_piece(board, e1(), None);
            assert_piece(board, h1(), None);

            assert_piece(
                board,
                g1(),
                Some(Piece {
                    color: Color::White,
                    kind: PieceType::King,
                }),
            );
            assert_piece(
                board,
                f1(),
                Some(Piece {
                    color: Color::White,
                    kind: PieceType::Rook,
                }),
            );

            assert_eq!(board.side_to_move, Color::Black);
            assert_eq!(board.en_passant, None);
            assert_eq!(board.halfmove_clock, 4);

            assert_eq!(
                board.castling_rights & (WHITE_KINGSIDE | WHITE_QUEENSIDE),
                0
            );

            assert_ne!(board.castling_rights & BLACK_KINGSIDE, 0);
            assert_ne!(board.castling_rights & BLACK_QUEENSIDE, 0);
        });
    }

    #[test]
    fn make_and_undo_rook_move_removes_only_that_rooks_castling_right() {
        let board = board_from_fen("r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1");

        let mv = Move {
            from: h1(),
            to: h2(),
            kind: MoveType::Normal,
            promotion: None,
        };

        make_check_undo(board, mv, |board| {
            assert_piece(board, h1(), None);
            assert_piece(
                board,
                h2(),
                Some(Piece {
                    color: Color::White,
                    kind: PieceType::Rook,
                }),
            );

            assert_eq!(board.castling_rights & WHITE_KINGSIDE, 0);
            assert_ne!(board.castling_rights & WHITE_QUEENSIDE, 0);

            assert_ne!(board.castling_rights & BLACK_KINGSIDE, 0);
            assert_ne!(board.castling_rights & BLACK_QUEENSIDE, 0);
        });
    }

    #[test]
    fn make_and_undo_rook_capture_removes_enemy_castling_right() {
        let board = board_from_fen("r3k2r/8/8/8/8/8/8/4K2R w Kkq - 0 1");

        let mv = Move {
            from: h1(),
            to: h8(),
            kind: MoveType::Capture,
            promotion: None,
        };

        make_check_undo(board, mv, |board| {
            assert_piece(board, h1(), None);
            assert_piece(
                board,
                h8(),
                Some(Piece {
                    color: Color::White,
                    kind: PieceType::Rook,
                }),
            );

            // White rook moved from h1, so white kingside castling is gone.
            assert_eq!(board.castling_rights & WHITE_KINGSIDE, 0);

            // Black h8 rook was captured, so black kingside castling is gone.
            assert_eq!(board.castling_rights & BLACK_KINGSIDE, 0);

            // Black a8 rook still exists, so black queenside castling remains.
            assert_ne!(board.castling_rights & BLACK_QUEENSIDE, 0);
        });
    }

    #[test]
    fn make_and_undo_quiet_promotion() {
        let board = board_from_fen("4k3/P7/8/8/8/8/8/4K3 w - - 0 1");

        let mv = Move {
            from: a7(),
            to: a8(),
            kind: MoveType::Normal,
            promotion: Some(PieceType::Queen),
        };

        make_check_undo(board, mv, |board| {
            assert_piece(board, a7(), None);
            assert_piece(
                board,
                a8(),
                Some(Piece {
                    color: Color::White,
                    kind: PieceType::Queen,
                }),
            );

            let white_pawns = board.pieces[Color::White as usize][PieceType::Pawn as usize];
            let white_queens = board.pieces[Color::White as usize][PieceType::Queen as usize];

            assert_eq!(white_pawns & (1u64 << a8()), 0);
            assert_ne!(white_queens & (1u64 << a8()), 0);

            assert_eq!(board.halfmove_clock, 0);
        });
    }

    #[test]
    fn make_and_undo_capture_promotion() {
        let board = board_from_fen("1n2k3/P7/8/8/8/8/8/4K3 w - - 0 1");

        let mv = Move {
            from: a7(),
            to: b8(),
            kind: MoveType::Capture,
            promotion: Some(PieceType::Queen),
        };

        make_check_undo(board, mv, |board| {
            assert_piece(board, a7(), None);
            assert_piece(
                board,
                b8(),
                Some(Piece {
                    color: Color::White,
                    kind: PieceType::Queen,
                }),
            );

            let black_knights = board.pieces[Color::Black as usize][PieceType::Knight as usize];
            assert_eq!(black_knights & (1u64 << b8()), 0);

            assert_eq!(board.halfmove_clock, 0);
        });
    }
    #[test]
    fn incremental_hash_matches_scratch_after_move_sequence_and_undos() {
        let mut board =
            Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();

        let start_hash = board.hash;

        board.assert_hash();

        let moves = [
            Move {
                from: sq(4, 1), // e2
                to: sq(4, 3),   // e4
                kind: MoveType::Normal,
                promotion: None,
            },
            Move {
                from: sq(4, 6), // e7
                to: sq(4, 4),   // e5
                kind: MoveType::Normal,
                promotion: None,
            },
            Move {
                from: sq(6, 0), // g1
                to: sq(5, 2),   // f3
                kind: MoveType::Normal,
                promotion: None,
            },
            Move {
                from: sq(1, 7), // b8
                to: sq(2, 5),   // c6
                kind: MoveType::Normal,
                promotion: None,
            },
        ];

        let mut undos = Vec::new();

        for mv in moves {
            let undo = board.make_move(mv);
            board.assert_hash();
            undos.push(undo);
        }

        while let Some(undo) = undos.pop() {
            board.undo_move(undo);
            board.assert_hash();
        }

        assert_eq!(board.hash, start_hash);
    }
}
