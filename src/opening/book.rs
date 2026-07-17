use std::collections::HashMap;

use rand::RngExt;

use crate::bitboard::{Square, file_of, rank_of};
use crate::board::{Board, Move, STARTPOS_FEN};
use crate::engine::Engine;
use crate::types::PieceType;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BookMove {
    pub mv: Move,
    pub weight: i32,
}

impl Engine {
    pub fn get_book_move(&self, board: &Board) -> Option<Move> {
        let book_mv = self.opening_book.get_move(board)?;
        let mut board_clone = board.clone();

        // Safety check: only play it if it is legal in the current position.
        let legal_moves = board_clone.all_legal_moves();

        for mv in legal_moves.iter() {
            if *mv == book_mv {
                return Some(book_mv);
            }
        }

        println!("Book move was found but was illegal: {:?}", book_mv);
        None
    }
}

#[derive(Clone, Debug)]
pub struct OpeningBook {
    pub entries: HashMap<u64, Vec<BookMove>>,
}

impl OpeningBook {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    pub fn add_line(&mut self, start_board: &Board, moves: &[&str], weight: i32) {
        let mut board = start_board.clone();

        for mv_str in moves {
            let hash = board.hash();

            let Some(mv) = find_legal_move_from_uci(&board, mv_str) else {
                panic!("Opening book move {mv_str} is not legal in this position");
            };

            let entry = self.entries.entry(hash).or_default();

            if let Some(existing) = entry.iter_mut().find(|book_move| book_move.mv == mv) {
                existing.weight += weight;
            } else {
                entry.push(BookMove { mv, weight });
            }

            board.make_move(mv);
        }
    }

    pub fn get_move(&self, board: &Board) -> Option<Move> {
        let moves = self.entries.get(&board.hash())?;

        // Safer than summing raw weights: ignores negative/zero weights for random choice.
        let total_weight: i32 = moves.iter().map(|m| m.weight.max(0)).sum();

        if total_weight <= 0 {
            return Some(moves[0].mv);
        }

        let mut rng = rand::rng();
        let mut roll = rng.random_range(0..total_weight);

        for book_move in moves {
            let weight = book_move.weight.max(0);

            if weight == 0 {
                continue;
            }

            roll -= weight;

            if roll < 0 {
                return Some(book_move.mv);
            }
        }

        Some(moves[0].mv)
    }
}

impl Default for OpeningBook {
    fn default() -> Self {
        Self::new()
    }
}

pub fn build_opening_book() -> OpeningBook {
    let start = Board::from_fen(STARTPOS_FEN).expect("FEN not working in opening book!");
    let mut book = OpeningBook::new();

    // Italian Game
    book.add_line(
        &start,
        &[
            "e2e4", "e7e5", "g1f3", "b8c6", "f1c4", "g8f6", "d2d3", "f8c5", "c2c3",
        ],
        5,
    );

    // Ruy Lopez
    book.add_line(
        &start,
        &[
            "e2e4", "e7e5", "g1f3", "b8c6", "f1b5", "a7a6", "b5a4", "g8f6", "e1g1",
        ],
        4,
    );

    // Scotch
    book.add_line(
        &start,
        &[
            "e2e4", "e7e5", "g1f3", "b8c6", "d2d4", "e5d4", "f3d4", "g8f6", "b1c3",
        ],
        4,
    );

    // Sicilian Defense
    book.add_line(
        &start,
        &[
            "e2e4", "c7c5", "g1f3", "d7d6", "d2d4", "c5d4", "f3d4", "g8f6", "b1c3", "a7a6",
        ],
        5,
    );

    // Dragon Sicilian
    book.add_line(
        &start,
        &[
            "e2e4", "c7c5", "g1f3", "d7d6", "d2d4", "c5d4", "f3d4", "g8f6", "b1c3", "g7g6", "c1e3",
            "f8g7", "d1d2",
        ],
        5,
    );

    // French Defense
    book.add_line(
        &start,
        &[
            "e2e4", "e7e6", "d2d4", "d7d5", "b1c3", "g8f6", "c1g5", "f8e7",
        ],
        3,
    );

    // Caro-Kann
    book.add_line(
        &start,
        &[
            "e2e4", "c7c6", "d2d4", "d7d5", "b1c3", "d5e4", "c3e4", "c8f5",
        ],
        3,
    );

    // Queen's Gambit Declined
    book.add_line(
        &start,
        &[
            "d2d4", "d7d5", "c2c4", "e7e6", "b1c3", "g8f6", "c1g5", "f8e7", "e2e3",
        ],
        4,
    );

    // King's Indian
    book.add_line(
        &start,
        &[
            "d2d4", "g8f6", "c2c4", "g7g6", "b1c3", "f8g7", "e2e4", "d7d6",
        ],
        4,
    );

    // English
    book.add_line(
        &start,
        &[
            "c2c4", "e7e5", "b1c3", "g8f6", "g2g3", "d7d5", "c4d5", "f6d5", "f1g2",
        ],
        2,
    );

    // Evans Gambit
    book.add_line(
        &start,
        &[
            "e2e4", "e7e5", "g1f3", "b8c6", "f1c4", "f8c5", "b2b4", "c5b4", "c2c3", "b4a5", "d2d4",
            "e5d4", "e1g1",
        ],
        6,
    );

    // Danish Gambit
    book.add_line(
        &start,
        &[
            "e2e4", "e7e5", "d2d4", "e5d4", "c2c3", "d4c3", "f1c4", "c3b2", "c1b2",
        ],
        5,
    );

    // King's Gambit Accepted
    book.add_line(
        &start,
        &[
            "e2e4", "e7e5", "f2f4", "e5f4", "g1f3", "g7g5", "f1c4", "f8g7", "e1g1",
        ],
        5,
    );

    // Vienna Gambit
    book.add_line(
        &start,
        &[
            "e2e4", "e7e5", "b1c3", "g8f6", "f2f4", "d7d5", "f4e5", "f6e4", "g1f3",
        ],
        4,
    );

    // Smith-Morra Gambit
    book.add_line(
        &start,
        &[
            "e2e4", "c7c5", "d2d4", "c5d4", "c2c3", "d4c3", "b1c3", "b8c6", "g1f3", "d7d6", "f1c4",
        ],
        6,
    );

    book
}

fn find_legal_move_from_uci(board: &Board, s: &str) -> Option<Move> {
    let parsed = parse_uci_move(s)?;

    let mut board_clone = board.clone();
    let legal_moves = board_clone.all_legal_moves();

    for mv in legal_moves.iter() {
        if move_matches_uci(*mv, parsed) {
            return Some(*mv);
        }
    }

    None
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ParsedUciMove {
    from: Square,
    to: Square,
    promotion: Option<PieceType>,
}

fn parse_uci_move(s: &str) -> Option<ParsedUciMove> {
    let bytes = s.as_bytes();

    if bytes.len() != 4 && bytes.len() != 5 {
        return None;
    }

    let from = square_from_uci(bytes[0], bytes[1])?;
    let to = square_from_uci(bytes[2], bytes[3])?;

    let promotion = if bytes.len() == 5 {
        Some(match bytes[4].to_ascii_lowercase() {
            b'q' => PieceType::Queen,
            b'r' => PieceType::Rook,
            b'b' => PieceType::Bishop,
            b'n' => PieceType::Knight,
            _ => return None,
        })
    } else {
        None
    };

    Some(ParsedUciMove {
        from,
        to,
        promotion,
    })
}

fn square_from_uci(file_char: u8, rank_char: u8) -> Option<Square> {
    if !(b'a'..=b'h').contains(&file_char) {
        return None;
    }

    if !(b'1'..=b'8').contains(&rank_char) {
        return None;
    }

    let file = file_char - b'a';
    let rank = rank_char - b'1';

    // Bitboard mapping:
    // a1 = 0, b1 = 1, ..., h1 = 7,
    // a2 = 8, ..., h8 = 63.
    Some(rank * 8 + file)
}

fn move_matches_uci(mv: Move, parsed: ParsedUciMove) -> bool {
    mv.from == parsed.from && mv.to == parsed.to && mv.promotion == parsed.promotion
}

#[allow(dead_code)]
fn move_to_uci(mv: Move) -> String {
    let mut out = String::new();

    out.push(file_char(file_of(mv.from)));
    out.push(rank_char(rank_of(mv.from)));
    out.push(file_char(file_of(mv.to)));
    out.push(rank_char(rank_of(mv.to)));

    if let Some(promo) = mv.promotion {
        out.push(match promo {
            PieceType::Queen => 'q',
            PieceType::Rook => 'r',
            PieceType::Bishop => 'b',
            PieceType::Knight => 'n',
            _ => panic!("Invalid promotion piece"),
        });
    }

    out
}

fn file_char(file: u8) -> char {
    debug_assert!(file < 8);
    (b'a' + file) as char
}

fn rank_char(rank: u8) -> char {
    debug_assert!(rank < 8);

    // New bitboard mapping:
    // rank 0 = first rank, rank 7 = eighth rank.
    (b'1' + rank) as char
}
