use crate::board::STARTPOS_FEN;
use crate::game::Game;
use crate::opening::book::find_legal_move_from_uci;

// move history is used for repetition detection although unlikelyt to occur
#[derive(Clone)]
pub struct OpeningPosition {
    pub name: String,
    pub game: Game,
}

impl OpeningPosition {
    pub fn new(name: String, game: Game) -> Self {
        Self { name, game }
    }

    // for now assumes starting fen is the standard starting position
    pub fn from_uci(name: impl Into<String>, starting_fen: String, uci_moves: &[&str]) -> Self {
        let name = name.into();

        // uses the Starting Position FEN as the starting position for the opening line
        let mut game = Game::from_fen(&starting_fen).expect("Invalid opening FEN");
        for &uci in uci_moves {
            let mv = find_legal_move_from_uci(&game.board, uci)
                .unwrap_or_else(|| panic!("Invalid UCI move `{uci}` in opening `{name}`"));
            game.play_move(mv).expect("Invalid move in game");
        }

        Self { name, game }
    }

    pub fn create_game(&self) -> Game {
        self.game.clone()
    }
}

#[derive(Clone)]
pub struct OpeningSuite {
    pub openings: Vec<OpeningPosition>,
}

impl OpeningSuite {
    pub fn new() -> Self {
        Self {
            openings: Vec::new(),
        }
    }
    pub fn add_line(
        &mut self,
        name: impl Into<String>,
        starting_fen: impl Into<String>,
        uci_moves: &[&str],
    ) {
        let opening_position = OpeningPosition::from_uci(name, starting_fen.into(), uci_moves);
        self.openings.push(opening_position);
    }
}

pub fn build_opening_suite() -> OpeningSuite {
    let mut suite = OpeningSuite::new();

    // Italian Game
    suite.add_line(
        "Italian Game",
        STARTPOS_FEN,
        &[
            "e2e4", "e7e5", "g1f3", "b8c6", "f1c4", "g8f6", "d2d3", "f8c5", "c2c3",
        ],
    );

    // Ruy Lopez
    suite.add_line(
        "Ruy Lopez",
        STARTPOS_FEN,
        &[
            "e2e4", "e7e5", "g1f3", "b8c6", "f1b5", "a7a6", "b5a4", "g8f6", "e1g1",
        ],
    );

    // Scotch
    suite.add_line(
        "Scotch",
        STARTPOS_FEN,
        &[
            "e2e4", "e7e5", "g1f3", "b8c6", "d2d4", "e5d4", "f3d4", "g8f6", "b1c3",
        ],
    );

    // Sicilian Defense
    suite.add_line(
        "Sicilian Defense",
        STARTPOS_FEN,
        &[
            "e2e4", "c7c5", "g1f3", "d7d6", "d2d4", "c5d4", "f3d4", "g8f6", "b1c3", "a7a6",
        ],
    );

    // Dragon Sicilian
    suite.add_line(
        "Dragon Sicilian",
        STARTPOS_FEN,
        &[
            "e2e4", "c7c5", "g1f3", "d7d6", "d2d4", "c5d4", "f3d4", "g8f6", "b1c3", "g7g6", "c1e3",
            "f8g7", "d1d2",
        ],
    );

    // French Defense
    suite.add_line(
        "French Defense",
        STARTPOS_FEN,
        &[
            "e2e4", "e7e6", "d2d4", "d7d5", "b1c3", "g8f6", "c1g5", "f8e7",
        ],
    );

    // Caro-Kann
    suite.add_line(
        "Caro-Kann",
        STARTPOS_FEN,
        &[
            "e2e4", "c7c6", "d2d4", "d7d5", "b1c3", "d5e4", "c3e4", "c8f5",
        ],
    );

    // Queen's Gambit Declined
    suite.add_line(
        "Queen's Gambit Declined",
        STARTPOS_FEN,
        &[
            "d2d4", "d7d5", "c2c4", "e7e6", "b1c3", "g8f6", "c1g5", "f8e7", "e2e3",
        ],
    );

    // King's Indian
    suite.add_line(
        "King's Indian",
        STARTPOS_FEN,
        &[
            "d2d4", "g8f6", "c2c4", "g7g6", "b1c3", "f8g7", "e2e4", "d7d6",
        ],
    );

    // English
    suite.add_line(
        "English",
        STARTPOS_FEN,
        &[
            "c2c4", "e7e5", "b1c3", "g8f6", "g2g3", "d7d5", "c4d5", "f6d5", "f1g2",
        ],
    );

    // Evans Gambit
    suite.add_line(
        "Evans Gambit",
        STARTPOS_FEN,
        &[
            "e2e4", "e7e5", "g1f3", "b8c6", "f1c4", "f8c5", "b2b4", "c5b4", "c2c3", "b4a5", "d2d4",
            "e5d4", "e1g1",
        ],
    );

    // Danish Gambit
    suite.add_line(
        "Danish Gambit",
        STARTPOS_FEN,
        &[
            "e2e4", "e7e5", "d2d4", "e5d4", "c2c3", "d4c3", "f1c4", "c3b2", "c1b2",
        ],
    );

    // King's Gambit Accepted
    suite.add_line(
        "King's Gambit Accepted",
        STARTPOS_FEN,
        &[
            "e2e4", "e7e5", "f2f4", "e5f4", "g1f3", "g7g5", "f1c4", "f8g7", "e1g1",
        ],
    );

    // Vienna Gambit
    suite.add_line(
        "Vienna Gambit",
        STARTPOS_FEN,
        &[
            "e2e4", "e7e5", "b1c3", "g8f6", "f2f4", "d7d5", "f4e5", "f6e4", "g1f3",
        ],
    );

    // Smith-Morra Gambit
    suite.add_line(
        "Smith-Morra Gambit",
        STARTPOS_FEN,
        &[
            "e2e4", "c7c5", "d2d4", "c5d4", "c2c3", "d4c3", "b1c3", "b8c6", "g1f3", "d7d6", "f1c4",
        ],
    );

    // -------------------------------------------------------------------------
    // Additional 1.e4 e5 openings
    // -------------------------------------------------------------------------

    // Petrov Defense
    suite.add_line(
        "Petrov Defense",
        STARTPOS_FEN,
        &[
            "e2e4", "e7e5", "g1f3", "g8f6", "f3e5", "d7d6", "e5f3", "f6e4", "d2d4", "d6d5", "f1d3",
            "f8e7", "e1g1", "e8g8",
        ],
    );

    // Four Knights Game
    suite.add_line(
        "Four Knights Game",
        STARTPOS_FEN,
        &[
            "e2e4", "e7e5", "g1f3", "b8c6", "b1c3", "g8f6", "f1b5", "f8b4", "e1g1", "e8g8", "d2d3",
            "d7d6",
        ],
    );

    // Philidor Defense
    suite.add_line(
        "Philidor Defense",
        STARTPOS_FEN,
        &[
            "e2e4", "e7e5", "g1f3", "d7d6", "d2d4", "e5d4", "f3d4", "g8f6", "b1c3", "f8e7", "f1e2",
            "e8g8", "e1g1",
        ],
    );

    // -------------------------------------------------------------------------
    // Hypermodern and unusual responses to 1.e4
    // -------------------------------------------------------------------------

    // Pirc Defense: Austrian Attack
    suite.add_line(
        "Pirc Defense: Austrian Attack",
        STARTPOS_FEN,
        &[
            "e2e4", "d7d6", "d2d4", "g8f6", "b1c3", "g7g6", "f2f4", "f8g7", "g1f3", "e8g8", "f1d3",
        ],
    );

    // Modern Defense
    suite.add_line(
        "Modern Defense",
        STARTPOS_FEN,
        &[
            "e2e4", "g7g6", "d2d4", "f8g7", "b1c3", "d7d6", "f2f4", "a7a6", "g1f3", "b7b5",
        ],
    );

    // Alekhine Defense
    suite.add_line(
        "Alekhine Defense",
        STARTPOS_FEN,
        &[
            "e2e4", "g8f6", "e4e5", "f6d5", "d2d4", "d7d6", "g1f3", "d6e5", "f3e5",
        ],
    );

    // Scandinavian Defense
    suite.add_line(
        "Scandinavian Defense",
        STARTPOS_FEN,
        &[
            "e2e4", "d7d5", "e4d5", "d8d5", "b1c3", "d5d8", "d2d4", "g8f6", "g1f3",
        ],
    );

    // -------------------------------------------------------------------------
    // Additional French and Caro-Kann structures
    // -------------------------------------------------------------------------

    // French Defense: Advance Variation
    suite.add_line(
        "French Defense: Advance Variation",
        STARTPOS_FEN,
        &[
            "e2e4", "e7e6", "d2d4", "d7d5", "e4e5", "c7c5", "c2c3", "b8c6", "g1f3", "d8b6", "f1d3",
        ],
    );

    // French Defense: Winawer Variation
    suite.add_line(
        "French Defense: Winawer Variation",
        STARTPOS_FEN,
        &[
            "e2e4", "e7e6", "d2d4", "d7d5", "b1c3", "f8b4", "e4e5", "c7c5", "a2a3", "b4c3", "b2c3",
            "g8e7",
        ],
    );

    // Caro-Kann: Advance Variation
    suite.add_line(
        "Caro-Kann: Advance Variation",
        STARTPOS_FEN,
        &[
            "e2e4", "c7c6", "d2d4", "d7d5", "e4e5", "c8f5", "g1f3", "e7e6", "f1e2", "c6c5", "e1g1",
            "b8c6",
        ],
    );

    // Caro-Kann: Panov Attack
    // Often produces an isolated queen pawn or hanging-pawn structure.
    suite.add_line(
        "Caro-Kann: Panov Attack",
        STARTPOS_FEN,
        &[
            "e2e4", "c7c6", "d2d4", "d7d5", "e4d5", "c6d5", "c2c4", "g8f6", "b1c3", "e7e6", "g1f3",
            "f8e7",
        ],
    );

    // -------------------------------------------------------------------------
    // Additional Sicilian structures
    // -------------------------------------------------------------------------

    // Sicilian Sveshnikov
    suite.add_line(
        "Sicilian Sveshnikov",
        STARTPOS_FEN,
        &[
            "e2e4", "c7c5", "g1f3", "b8c6", "d2d4", "c5d4", "f3d4", "g8f6", "b1c3", "e7e5", "d4b5",
            "d7d6", "c1g5", "a7a6", "b5a3", "b7b5",
        ],
    );

    // Classical Sicilian: Richter-Rauzer
    // Produces opposite-side castling and direct attacks.
    suite.add_line(
        "Classical Sicilian: Richter-Rauzer",
        STARTPOS_FEN,
        &[
            "e2e4", "c7c5", "g1f3", "d7d6", "d2d4", "c5d4", "f3d4", "g8f6", "b1c3", "b8c6", "c1g5",
            "e7e6", "d1d2", "f8e7", "e1c1",
        ],
    );

    // Sicilian Alapin
    suite.add_line(
        "Sicilian Alapin",
        STARTPOS_FEN,
        &[
            "e2e4", "c7c5", "c2c3", "g8f6", "e4e5", "f6d5", "d2d4", "c5d4", "g1f3", "b8c6", "c3d4",
            "d7d6",
        ],
    );

    // Closed Sicilian
    suite.add_line(
        "Closed Sicilian",
        STARTPOS_FEN,
        &[
            "e2e4", "c7c5", "b1c3", "b8c6", "g2g3", "g7g6", "f1g2", "f8g7", "d2d3", "d7d6", "f2f4",
            "e7e5",
        ],
    );

    // -------------------------------------------------------------------------
    // Queen's Gambit family
    // -------------------------------------------------------------------------

    // Queen's Gambit Accepted
    suite.add_line(
        "Queen's Gambit Accepted",
        STARTPOS_FEN,
        &[
            "d2d4", "d7d5", "c2c4", "d5c4", "g1f3", "g8f6", "e2e3", "e7e6", "f1c4", "c7c5", "e1g1",
            "a7a6",
        ],
    );

    // Slav Defense
    suite.add_line(
        "Slav Defense",
        STARTPOS_FEN,
        &[
            "d2d4", "d7d5", "c2c4", "c7c6", "g1f3", "g8f6", "b1c3", "d5c4", "a2a4", "c8f5", "e2e3",
            "e7e6", "f1c4", "f8b4",
        ],
    );

    // Semi-Slav Defense
    suite.add_line(
        "Semi-Slav Defense",
        STARTPOS_FEN,
        &[
            "d2d4", "d7d5", "c2c4", "e7e6", "b1c3", "g8f6", "g1f3", "c7c6", "e2e3", "b8d7", "f1d3",
            "d5c4", "d3c4", "b7b5",
        ],
    );

    // Queen's Gambit Declined: Exchange Variation
    suite.add_line(
        "Queen's Gambit Declined: Exchange Variation",
        STARTPOS_FEN,
        &[
            "d2d4", "d7d5", "c2c4", "e7e6", "b1c3", "g8f6", "c4d5", "e6d5", "c1g5", "c7c6", "e2e3",
            "c8f5",
        ],
    );

    // Tarrasch Defense
    // Frequently creates an isolated black queen pawn.
    suite.add_line(
        "Tarrasch Defense",
        STARTPOS_FEN,
        &[
            "d2d4", "d7d5", "c2c4", "e7e6", "b1c3", "c7c5", "c4d5", "e6d5", "g1f3", "b8c6", "g2g3",
            "g8f6",
        ],
    );

    // Catalan Opening
    suite.add_line(
        "Catalan Opening",
        STARTPOS_FEN,
        &[
            "d2d4", "g8f6", "c2c4", "e7e6", "g2g3", "d7d5", "f1g2", "f8e7", "g1f3", "e8g8", "e1g1",
            "d5c4", "d1c2", "a7a6",
        ],
    );

    // -------------------------------------------------------------------------
    // Indian defenses and asymmetrical 1.d4 positions
    // -------------------------------------------------------------------------

    // Nimzo-Indian Defense
    suite.add_line(
        "Nimzo-Indian Defense",
        STARTPOS_FEN,
        &[
            "d2d4", "g8f6", "c2c4", "e7e6", "b1c3", "f8b4", "e2e3", "e8g8", "f1d3", "d7d5", "g1f3",
            "c7c5", "e1g1",
        ],
    );

    // Queen's Indian Defense
    suite.add_line(
        "Queen's Indian Defense",
        STARTPOS_FEN,
        &[
            "d2d4", "g8f6", "c2c4", "e7e6", "g1f3", "b7b6", "g2g3", "c8a6", "b2b3", "f8b4", "c1d2",
            "b4e7", "f1g2",
        ],
    );

    // Grünfeld Defense: Exchange Variation
    suite.add_line(
        "Grünfeld Defense: Exchange Variation",
        STARTPOS_FEN,
        &[
            "d2d4", "g8f6", "c2c4", "g7g6", "b1c3", "d7d5", "c4d5", "f6d5", "e2e4", "d5c3", "b2c3",
            "f8g7",
        ],
    );

    // Modern Benoni
    suite.add_line(
        "Modern Benoni",
        STARTPOS_FEN,
        &[
            "d2d4", "g8f6", "c2c4", "c7c5", "d4d5", "e7e6", "b1c3", "e6d5", "c4d5", "d7d6", "e2e4",
            "g7g6", "f2f4", "f8g7",
        ],
    );

    // Benko Gambit
    // Gives Black long-term queenside activity for a pawn.
    suite.add_line(
        "Benko Gambit",
        STARTPOS_FEN,
        &[
            "d2d4", "g8f6", "c2c4", "c7c5", "d4d5", "b7b5", "c4b5", "a7a6", "b5a6", "g7g6", "b1c3",
            "c8a6",
        ],
    );

    // King's Indian: Sämisch Variation
    suite.add_line(
        "King's Indian: Sämisch Variation",
        STARTPOS_FEN,
        &[
            "d2d4", "g8f6", "c2c4", "g7g6", "b1c3", "f8g7", "e2e4", "d7d6", "f2f3", "e8g8", "c1e3",
            "e7e5", "d4d5",
        ],
    );

    // Budapest Gambit
    suite.add_line(
        "Budapest Gambit",
        STARTPOS_FEN,
        &[
            "d2d4", "g8f6", "c2c4", "e7e5", "d4e5", "f6g4", "g1f3", "b8c6", "c1f4", "f8b4", "b1d2",
            "d8e7",
        ],
    );

    // Dutch Defense: Leningrad
    suite.add_line(
        "Dutch Defense: Leningrad",
        STARTPOS_FEN,
        &[
            "d2d4", "f7f5", "g2g3", "g8f6", "f1g2", "g7g6", "g1f3", "f8g7", "e1g1", "e8g8", "c2c4",
            "d7d6", "b1c3",
        ],
    );

    // -------------------------------------------------------------------------
    // Independent 1.d4 systems
    // -------------------------------------------------------------------------

    // London System
    suite.add_line(
        "London System",
        STARTPOS_FEN,
        &[
            "d2d4", "d7d5", "g1f3", "g8f6", "c1f4", "e7e6", "e2e3", "f8d6", "f4g3", "e8g8", "f1d3",
            "c7c5",
        ],
    );

    // Trompowsky Attack
    suite.add_line(
        "Trompowsky Attack",
        STARTPOS_FEN,
        &[
            "d2d4", "g8f6", "c1g5", "f6e4", "g5f4", "d7d5", "e2e3", "c7c5", "f1d3", "b8c6",
        ],
    );

    // Jobava London
    suite.add_line(
        "Jobava London",
        STARTPOS_FEN,
        &[
            "d2d4", "d7d5", "b1c3", "g8f6", "c1f4", "c7c5", "e2e3", "b8c6", "c3b5", "e7e5", "d4e5",
        ],
    );

    // Blackmar-Diemer Gambit
    suite.add_line(
        "Blackmar-Diemer Gambit",
        STARTPOS_FEN,
        &[
            "d2d4", "d7d5", "e2e4", "d5e4", "b1c3", "g8f6", "f2f3", "e4f3", "g1f3",
        ],
    );

    // Albin Countergambit
    suite.add_line(
        "Albin Countergambit",
        STARTPOS_FEN,
        &[
            "d2d4", "d7d5", "c2c4", "e7e5", "d4e5", "d5d4", "g1f3", "b8c6", "a2a3", "c8e6",
        ],
    );

    // -------------------------------------------------------------------------
    // Flank openings
    // -------------------------------------------------------------------------

    // English Opening: Symmetrical Variation
    suite.add_line(
        "English Opening: Symmetrical Variation",
        STARTPOS_FEN,
        &[
            "c2c4", "c7c5", "b1c3", "b8c6", "g2g3", "g7g6", "f1g2", "f8g7", "g1f3", "e7e5", "e1g1",
            "g8e7",
        ],
    );

    // English Opening: Botvinnik Setup
    suite.add_line(
        "English Opening: Botvinnik Setup",
        STARTPOS_FEN,
        &[
            "c2c4", "e7e5", "b1c3", "b8c6", "g2g3", "g7g6", "f1g2", "f8g7", "e2e4", "d7d6", "g1e2",
        ],
    );

    // Réti Opening
    suite.add_line(
        "Réti Opening",
        STARTPOS_FEN,
        &[
            "g1f3", "d7d5", "c2c4", "e7e6", "g2g3", "g8f6", "f1g2", "f8e7", "e1g1", "e8g8", "d2d4",
        ],
    );

    // King's Indian Attack
    suite.add_line(
        "King's Indian Attack",
        STARTPOS_FEN,
        &[
            "g1f3", "d7d5", "g2g3", "g8f6", "f1g2", "e7e6", "e1g1", "f8e7", "d2d3", "e8g8", "b1d2",
        ],
    );

    // Bird Opening
    suite.add_line(
        "Bird Opening",
        STARTPOS_FEN,
        &[
            "f2f4", "d7d5", "g1f3", "g8f6", "e2e3", "g7g6", "b2b3", "f8g7", "c1b2", "e8g8",
        ],
    );

    // Polish Opening / Sokolsky
    suite.add_line(
        "Polish Opening / Sokolsky",
        STARTPOS_FEN,
        &[
            "b2b4", "e7e5", "c1b2", "f8b4", "b2e5", "g8f6", "g1f3", "e8g8", "e2e3", "d7d5",
        ],
    );

    suite
}

pub fn build_important_opening_suite() -> OpeningSuite {
    let mut suite = OpeningSuite::new();

    // Balanced open center; general development and king safety.
    suite.add_line(
        "Italian Game",
        STARTPOS_FEN,
        &[
            "e2e4", "e7e5", "g1f3", "b8c6", "f1c4", "g8f6", "d2d3", "f8c5", "c2c3",
        ],
    );

    // Symmetrical and relatively quiet; useful for detecting positional
    // weaknesses and unnecessary risk-taking.
    suite.add_line(
        "Petrov Defense",
        STARTPOS_FEN,
        &[
            "e2e4", "e7e5", "g1f3", "g8f6", "f3e5", "d7d6", "e5f3", "f6e4", "d2d4", "d6d5", "f1d3",
            "f8e7", "e1g1", "e8g8",
        ],
    );

    // Open tactical position with sacrificed material and rapid development.
    suite.add_line(
        "Evans Gambit",
        STARTPOS_FEN,
        &[
            "e2e4", "e7e5", "g1f3", "b8c6", "f1c4", "f8c5", "b2b4", "c5b4", "c2c3", "b4a5", "d2d4",
            "e5d4", "e1g1",
        ],
    );

    // Opposite-side castling and direct attacks.
    suite.add_line(
        "Classical Sicilian: Richter-Rauzer",
        STARTPOS_FEN,
        &[
            "e2e4", "c7c5", "g1f3", "d7d6", "d2d4", "c5d4", "f3d4", "g8f6", "b1c3", "b8c6", "c1g5",
            "e7e6", "d1d2", "f8e7", "e1c1",
        ],
    );

    // Closed center and pawn-chain play.
    suite.add_line(
        "French Defense: Advance Variation",
        STARTPOS_FEN,
        &[
            "e2e4", "e7e6", "d2d4", "d7d5", "e4e5", "c7c5", "c2c3", "b8c6", "g1f3", "d8b6", "f1d3",
        ],
    );

    // Isolated queen pawn or hanging-pawn structures.
    suite.add_line(
        "Caro-Kann: Panov Attack",
        STARTPOS_FEN,
        &[
            "e2e4", "c7c6", "d2d4", "d7d5", "e4d5", "c6d5", "c2c4", "g8f6", "b1c3", "e7e6", "g1f3",
            "f8e7",
        ],
    );

    // Carlsbad pawn structure and long-term positional plans.
    suite.add_line(
        "Queen's Gambit Declined: Exchange Variation",
        STARTPOS_FEN,
        &[
            "d2d4", "d7d5", "c2c4", "e7e6", "b1c3", "g8f6", "c4d5", "e6d5", "c1g5", "c7c6", "e2e3",
            "c8f5",
        ],
    );

    // Solid but tactically rich central tension.
    suite.add_line(
        "Semi-Slav Defense",
        STARTPOS_FEN,
        &[
            "d2d4", "d7d5", "c2c4", "e7e6", "b1c3", "g8f6", "g1f3", "c7c6", "e2e3", "b8d7", "f1d3",
            "d5c4", "d3c4", "b7b5",
        ],
    );

    // Bishop pair versus structural damage and central pressure.
    suite.add_line(
        "Nimzo-Indian Defense",
        STARTPOS_FEN,
        &[
            "d2d4", "g8f6", "c2c4", "e7e6", "b1c3", "f8b4", "e2e3", "e8g8", "f1d3", "d7d5", "g1f3",
            "c7c5", "e1g1",
        ],
    );

    // Large pawn center versus piece activity and pressure.
    suite.add_line(
        "Grünfeld Defense: Exchange Variation",
        STARTPOS_FEN,
        &[
            "d2d4", "g8f6", "c2c4", "g7g6", "b1c3", "d7d5", "c4d5", "f6d5", "e2e4", "d5c3", "b2c3",
            "f8g7",
        ],
    );

    // Closed center with kingside attacking chances.
    suite.add_line(
        "King's Indian: Sämisch Variation",
        STARTPOS_FEN,
        &[
            "d2d4", "g8f6", "c2c4", "g7g6", "b1c3", "f8g7", "e2e4", "d7d6", "f2f3", "e8g8", "c1e3",
            "e7e5", "d4d5",
        ],
    );

    // Asymmetrical structure: White space versus Black counterplay.
    suite.add_line(
        "Modern Benoni",
        STARTPOS_FEN,
        &[
            "d2d4", "g8f6", "c2c4", "c7c5", "d4d5", "e7e6", "b1c3", "e6d5", "c4d5", "d7d6", "e2e4",
            "g7g6", "f2f4", "f8g7",
        ],
    );

    // Slow flank play with a mostly symmetrical structure.
    suite.add_line(
        "English Opening: Symmetrical Variation",
        STARTPOS_FEN,
        &[
            "c2c4", "c7c5", "b1c3", "b8c6", "g2g3", "g7g6", "f1g2", "f8g7", "g1f3", "e7e5", "e1g1",
            "g8e7",
        ],
    );

    // Hypermodern development and transpositional play.
    suite.add_line(
        "Réti Opening",
        STARTPOS_FEN,
        &[
            "g1f3", "d7d5", "c2c4", "e7e6", "g2g3", "g8f6", "f1g2", "f8e7", "e1g1", "e8g8", "d2d4",
        ],
    );

    suite
}
