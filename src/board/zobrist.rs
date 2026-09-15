use std::sync::OnceLock;

use crate::bitboard::{Square, file_of};
use crate::board::Board;
use crate::types::{Color, PieceType};

pub static ZOBRIST: OnceLock<Zobrist> = OnceLock::new();

pub fn zobrist() -> &'static Zobrist {
    ZOBRIST.get_or_init(Zobrist::new)
}

pub struct SplitMix64 {
    // for random but deterministic seeds
    state: u64,
}

impl SplitMix64 {
    pub fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);

        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
}

pub struct Zobrist {
    pub pieces: [[[u64; 64]; 6]; 2], // pieces[color][piece_type][square]
    pub side_to_move: u64,
    pub castling: [u64; 16],
    pub en_passant_file: [u64; 8],
}

impl Default for Zobrist {
    fn default() -> Self {
        Self::new()
    }
}

impl Zobrist {
    pub fn new() -> Self {
        let mut rng = SplitMix64::new(0xC0FFEE_12345678);

        let mut pieces = [[[0u64; 64]; 6]; 2];

        for color in 0..2 {
            for piece in 0..6 {
                for sq in 0..64 {
                    pieces[color][piece][sq] = rng.next_u64();
                }
            }
        }

        let side_to_move = rng.next_u64();

        let mut castling = [0u64; 16];
        for key in &mut castling {
            *key = rng.next_u64();
        }

        let mut en_passant_file = [0u64; 8];
        for key in &mut en_passant_file {
            *key = rng.next_u64();
        }

        Self {
            pieces,
            side_to_move,
            castling,
            en_passant_file,
        }
    }
}

impl Board {
    // **********************
    // **** ZOBRIST HASH ****
    // **********************

    pub fn compute_hash_from_scratch(&self) -> u64 {
        let z = zobrist();
        let mut hash = 0u64;

        for color in [Color::White, Color::Black] {
            for piece in [
                PieceType::Pawn,
                PieceType::Knight,
                PieceType::Bishop,
                PieceType::Rook,
                PieceType::Queen,
                PieceType::King,
            ] {
                let mut bb = self.pieces(color, piece);

                while bb != 0 {
                    let sq = bb.trailing_zeros() as Square;
                    bb &= bb - 1;

                    hash ^= z.pieces[color.idx()][piece.idx()][sq as usize];
                }
            }
        }

        if self.side_to_move == Color::Black {
            hash ^= z.side_to_move;
        }

        hash ^= z.castling[self.castling_rights as usize];

        if let Some(ep_sq) = self.en_passant {
            let file = ep_sq % 8;
            hash ^= z.en_passant_file[file as usize];
        }

        hash
    }

    pub fn compute_pawn_hash_from_scratch(&self) -> u64 {
        let z = zobrist();
        let mut hash = 0u64;

        for color in [Color::White, Color::Black] {
            let mut bb = self.pieces(color, PieceType::Pawn);

            while bb != 0 {
                let sq = bb.trailing_zeros() as Square;
                bb &= bb - 1;

                hash ^= z.pieces[color.idx()][PieceType::Pawn.idx()][sq as usize];
            }
        }

        hash
    }

    // **********************
    // **** HASH HELPERS ****
    // **********************

    #[inline]
    fn xor_piece_hash(&mut self, color: Color, piece: PieceType, sq: Square) {
        let z = zobrist();

        self.board_hash ^= z.pieces[color.idx()][piece.idx()][sq as usize];

        if piece == PieceType::Pawn {
            self.pawn_hash ^= z.pieces[color.idx()][piece.idx()][sq as usize];
        }
    }

    #[inline]
    fn xor_castling_hash(&mut self, castling_rights: u8) {
        let z = zobrist();

        self.board_hash ^= z.castling[castling_rights as usize];
    }

    #[inline]
    fn xor_en_passant_hash(&mut self, ep: Square) {
        let z = zobrist();

        let file = file_of(ep) as usize;
        self.board_hash ^= z.en_passant_file[file];
    }

    #[inline]
    pub fn xor_side_to_move_hash(&mut self) {
        let z = zobrist();

        self.board_hash ^= z.side_to_move;
    }

    pub fn remove_piece_hashed(&mut self, color: Color, piece: PieceType, sq: Square) {
        self.remove_piece(color, piece, sq);
        self.xor_piece_hash(color, piece, sq);
    }

    pub fn add_piece_hashed(&mut self, color: Color, piece: PieceType, sq: Square) {
        self.add_piece(color, piece, sq);
        self.xor_piece_hash(color, piece, sq);
    }

    pub fn move_piece_hashed(&mut self, color: Color, piece: PieceType, from: Square, to: Square) {
        self.remove_piece_hashed(color, piece, from);
        self.add_piece_hashed(color, piece, to);
    }

    pub fn clear_en_passant_hashed(&mut self) {
        if let Some(ep) = self.en_passant {
            self.xor_en_passant_hash(ep);
        }

        self.en_passant = None;
    }

    pub fn set_en_passant_hashed(&mut self, ep: Option<Square>) {
        debug_assert!(
            self.en_passant.is_none(),
            "set_en_passant_hashed called while old en passant still exists"
        );

        self.en_passant = ep;

        if let Some(ep) = ep {
            self.xor_en_passant_hash(ep);
        }
    }

    pub fn update_castling_hash_after_rights_change(&mut self, old_rights: u8) {
        let new_rights = self.castling_rights;

        if old_rights != new_rights {
            self.xor_castling_hash(old_rights);
            self.xor_castling_hash(new_rights);
        }
    }
}
