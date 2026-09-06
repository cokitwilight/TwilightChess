use std::{collections::HashMap, fmt};

use rand::RngExt;

use crate::board::{Board, Move};
use crate::engine::Engine;
use crate::game::Game;
use crate::uci::find_legal_move_from_uci;

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

#[derive(Clone)]
pub struct OpeningLine {
    pub name: String,
    pub game: Game,
    pub weight: i32,
}
impl OpeningLine {
    pub fn from_uci(
        name: impl Into<String>,
        starting_fen: impl Into<String>,
        uci_moves: &[&str],
        weight: i32,
    ) -> Self {
        let name = name.into();
        let starting_fen = starting_fen.into();
        let mut game = Game::from_fen(&starting_fen).expect("Invalid opening FEN");

        for &uci in uci_moves {
            let mv = find_legal_move_from_uci(&game.board, uci)
                .unwrap_or_else(|| panic!("Invalid UCI move `{uci}` in opening `{name}`"));
            game.play_move(mv)
                .unwrap_or_else(|_| panic!("Invalid UCI move `{uci}` in opening `{name}`"));
        }

        Self { name, game, weight }
    }

    pub fn create_game(&self) -> Game {
        self.game.clone()
    }
}

impl fmt::Debug for OpeningLine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OpeningLine")
            .field("name", &self.name)
            .field("starting_fen", &self.game.starting_fen)
            .field("move_count", &self.game.move_history.len())
            .field("weight", &self.weight)
            .finish()
    }
}

#[derive(Clone)]
pub struct OpeningBook {
    pub entries: HashMap<u64, Vec<BookMove>>,
    pub openings: Vec<OpeningLine>,
}

impl OpeningBook {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
            openings: Vec::new(),
        }
    }

    pub fn add_line(
        &mut self,
        name: impl Into<String>,
        starting_fen: impl Into<String>,
        uci_moves: &[&str],
        weight: i32,
    ) {
        let line = OpeningLine::from_uci(name, starting_fen, uci_moves, weight);
        self.index_line(&line);
        self.openings.push(line);
    }

    fn index_line(&mut self, line: &OpeningLine) {
        let mut board = Board::from_fen(&line.game.starting_fen)
            .unwrap_or_else(|_| panic!("Invalid opening FEN for `{}`", line.name));

        for &mv in &line.game.move_history {
            let entry = self.entries.entry(board.hash()).or_default();

            if let Some(existing) = entry.iter_mut().find(|book_move| book_move.mv == mv) {
                existing.weight += line.weight;
            } else {
                entry.push(BookMove {
                    mv,
                    weight: line.weight,
                });
            }

            board.make_move(mv);
        }
    }

    pub fn random_line(&self) -> Option<OpeningLine> {
        if self.openings.is_empty() {
            return None;
        }

        let mut rng = rand::rng();
        let index = rng.random_range(0..self.openings.len());
        Some(self.openings[index].clone())
    }

    pub fn line_from_name(&self, name: &str) -> Option<OpeningLine> {
        self.openings
            .iter()
            .find(|opening| opening.name == name)
            .cloned()
    }

    pub fn only_named(&self, names: &[&str]) -> Self {
        let mut filtered = Self::new();

        for &name in names {
            let line = self
                .line_from_name(name)
                .unwrap_or_else(|| panic!("Unknown opening `{name}`"));
            filtered.index_line(&line);
            filtered.openings.push(line);
        }

        filtered
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

impl fmt::Debug for OpeningBook {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OpeningBook")
            .field("entries", &self.entries)
            .field("openings", &self.openings)
            .finish()
    }
}

impl Default for OpeningBook {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use crate::board::STARTPOS_FEN;
    use crate::opening::{
        IMPORTANT_OPENINGS, SUGGESTION_OPENINGS, build_important_opening_book, build_opening_book,
        build_suggestion_opening_book,
    };

    use super::*;

    #[test]
    fn adding_a_line_updates_search_and_tournament_views() {
        let mut book = OpeningBook::new();
        book.add_line("King's Pawn", STARTPOS_FEN, &["e2e4"], 7);

        let line = book.line_from_name("King's Pawn").unwrap();
        assert_eq!(line.game.move_history.len(), 1);
        assert_eq!(line.weight, 7);

        let board = Board::from_fen(STARTPOS_FEN).unwrap();
        assert_eq!(book.get_move(&board), Some(line.game.move_history[0]));
    }

    #[test]
    fn catalog_builds_both_views_from_all_openings() {
        let book = build_opening_book();

        assert_eq!(book.openings.len(), 100);
        assert!(!book.entries.is_empty());

        let mut names = HashSet::new();
        for line in &book.openings {
            assert!(names.insert(line.name.as_str()), "duplicate opening name");
        }

        for (index, line) in book.openings.iter().enumerate() {
            for other in book.openings.iter().skip(index + 1) {
                assert_ne!(
                    line.game.move_history, other.game.move_history,
                    "duplicate opening lines: `{}` and `{}`",
                    line.name, other.name
                );
            }
        }

        let italian = book.line_from_name("Italian Game").unwrap();
        assert_eq!(italian.game.move_history.len(), 9);
        assert_eq!(italian.weight, 5);
    }

    #[test]
    fn important_book_filters_the_shared_catalog() {
        let full_book = build_opening_book();
        let important_book = build_important_opening_book();

        assert_eq!(important_book.openings.len(), IMPORTANT_OPENINGS.len());

        for &name in IMPORTANT_OPENINGS {
            let full_line = full_book.line_from_name(name).unwrap();
            let important_line = important_book.line_from_name(name).unwrap();
            assert_eq!(
                important_line.game.move_history,
                full_line.game.move_history
            );
            assert_eq!(important_line.weight, full_line.weight);
        }
    }

    #[test]
    fn suggestion_book_contains_25_short_lines() {
        let full_book = build_opening_book();
        let suggestion_book = build_suggestion_opening_book();

        assert_eq!(SUGGESTION_OPENINGS.len(), 25);
        assert_eq!(suggestion_book.openings.len(), 25);

        let mut suggestion_names = HashSet::new();
        for &name in SUGGESTION_OPENINGS {
            assert!(
                suggestion_names.insert(name),
                "duplicate suggestion opening `{name}`"
            );

            let full_line = full_book.line_from_name(name).unwrap();
            let suggestion_line = suggestion_book.line_from_name(name).unwrap();
            let move_count = suggestion_line.game.move_history.len();

            assert!(
                (1..=3).contains(&move_count),
                "suggestion `{name}` has {move_count} moves"
            );
            assert_eq!(
                suggestion_line.game.move_history,
                full_line.game.move_history
            );
            assert_eq!(suggestion_line.weight, full_line.weight);
        }
    }
}
