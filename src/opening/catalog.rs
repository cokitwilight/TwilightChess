use crate::board::STARTPOS_FEN;

use crate::opening::OpeningBook;

pub fn build_opening_book() -> OpeningBook {
    let mut book = OpeningBook::new();

    // Italian Game
    book.add_line(
        "Italian Game",
        STARTPOS_FEN,
        &[
            "e2e4", "e7e5", "g1f3", "b8c6", "f1c4", "g8f6", "d2d3", "f8c5", "c2c3",
        ],
        5,
    );

    // Ruy Lopez
    book.add_line(
        "Ruy Lopez",
        STARTPOS_FEN,
        &[
            "e2e4", "e7e5", "g1f3", "b8c6", "f1b5", "a7a6", "b5a4", "g8f6", "e1g1",
        ],
        4,
    );

    // Scotch
    book.add_line(
        "Scotch",
        STARTPOS_FEN,
        &[
            "e2e4", "e7e5", "g1f3", "b8c6", "d2d4", "e5d4", "f3d4", "g8f6", "b1c3",
        ],
        4,
    );

    // Sicilian Defense
    book.add_line(
        "Sicilian Defense",
        STARTPOS_FEN,
        &[
            "e2e4", "c7c5", "g1f3", "d7d6", "d2d4", "c5d4", "f3d4", "g8f6", "b1c3", "a7a6",
        ],
        5,
    );

    // Dragon Sicilian
    book.add_line(
        "Dragon Sicilian",
        STARTPOS_FEN,
        &[
            "e2e4", "c7c5", "g1f3", "d7d6", "d2d4", "c5d4", "f3d4", "g8f6", "b1c3", "g7g6", "c1e3",
            "f8g7", "d1d2",
        ],
        5,
    );

    // French Defense
    book.add_line(
        "French Defense",
        STARTPOS_FEN,
        &[
            "e2e4", "e7e6", "d2d4", "d7d5", "b1c3", "g8f6", "c1g5", "f8e7",
        ],
        3,
    );

    // Caro-Kann
    book.add_line(
        "Caro-Kann",
        STARTPOS_FEN,
        &[
            "e2e4", "c7c6", "d2d4", "d7d5", "b1c3", "d5e4", "c3e4", "c8f5",
        ],
        3,
    );

    // Queen's Gambit Declined
    book.add_line(
        "Queen's Gambit Declined",
        STARTPOS_FEN,
        &[
            "d2d4", "d7d5", "c2c4", "e7e6", "b1c3", "g8f6", "c1g5", "f8e7", "e2e3",
        ],
        4,
    );

    // King's Indian
    book.add_line(
        "King's Indian",
        STARTPOS_FEN,
        &[
            "d2d4", "g8f6", "c2c4", "g7g6", "b1c3", "f8g7", "e2e4", "d7d6",
        ],
        4,
    );

    // English
    book.add_line(
        "English",
        STARTPOS_FEN,
        &[
            "c2c4", "e7e5", "b1c3", "g8f6", "g2g3", "d7d5", "c4d5", "f6d5", "f1g2",
        ],
        2,
    );

    // Evans Gambit
    book.add_line(
        "Evans Gambit",
        STARTPOS_FEN,
        &[
            "e2e4", "e7e5", "g1f3", "b8c6", "f1c4", "f8c5", "b2b4", "c5b4", "c2c3", "b4a5", "d2d4",
            "e5d4", "e1g1",
        ],
        6,
    );

    // Danish Gambit
    book.add_line(
        "Danish Gambit",
        STARTPOS_FEN,
        &[
            "e2e4", "e7e5", "d2d4", "e5d4", "c2c3", "d4c3", "f1c4", "c3b2", "c1b2",
        ],
        5,
    );

    // King's Gambit Accepted
    book.add_line(
        "King's Gambit Accepted",
        STARTPOS_FEN,
        &[
            "e2e4", "e7e5", "f2f4", "e5f4", "g1f3", "g7g5", "f1c4", "f8g7", "e1g1",
        ],
        5,
    );

    // Vienna Gambit
    book.add_line(
        "Vienna Gambit",
        STARTPOS_FEN,
        &[
            "e2e4", "e7e5", "b1c3", "g8f6", "f2f4", "d7d5", "f4e5", "f6e4", "g1f3",
        ],
        4,
    );

    // Smith-Morra Gambit
    book.add_line(
        "Smith-Morra Gambit",
        STARTPOS_FEN,
        &[
            "e2e4", "c7c5", "d2d4", "c5d4", "c2c3", "d4c3", "b1c3", "b8c6", "g1f3", "d7d6", "f1c4",
        ],
        6,
    );

    // -------------------------------------------------------------------------
    // Additional 1.e4 e5 openings
    // -------------------------------------------------------------------------

    // Petrov Defense
    book.add_line(
        "Petrov Defense",
        STARTPOS_FEN,
        &[
            "e2e4", "e7e5", "g1f3", "g8f6", "f3e5", "d7d6", "e5f3", "f6e4", "d2d4", "d6d5", "f1d3",
            "f8e7", "e1g1", "e8g8",
        ],
        3,
    );

    // Four Knights Game
    book.add_line(
        "Four Knights Game",
        STARTPOS_FEN,
        &[
            "e2e4", "e7e5", "g1f3", "b8c6", "b1c3", "g8f6", "f1b5", "f8b4", "e1g1", "e8g8", "d2d3",
            "d7d6",
        ],
        3,
    );

    // Philidor Defense
    book.add_line(
        "Philidor Defense",
        STARTPOS_FEN,
        &[
            "e2e4", "e7e5", "g1f3", "d7d6", "d2d4", "e5d4", "f3d4", "g8f6", "b1c3", "f8e7", "f1e2",
            "e8g8", "e1g1",
        ],
        2,
    );

    // -------------------------------------------------------------------------
    // Hypermodern and unusual responses to 1.e4
    // -------------------------------------------------------------------------

    // Pirc Defense: Austrian Attack
    book.add_line(
        "Pirc Defense: Austrian Attack",
        STARTPOS_FEN,
        &[
            "e2e4", "d7d6", "d2d4", "g8f6", "b1c3", "g7g6", "f2f4", "f8g7", "g1f3", "e8g8", "f1d3",
        ],
        4,
    );

    // Modern Defense
    book.add_line(
        "Modern Defense",
        STARTPOS_FEN,
        &[
            "e2e4", "g7g6", "d2d4", "f8g7", "b1c3", "d7d6", "f2f4", "a7a6", "g1f3", "b7b5",
        ],
        3,
    );

    // Alekhine Defense
    book.add_line(
        "Alekhine Defense",
        STARTPOS_FEN,
        &[
            "e2e4", "g8f6", "e4e5", "f6d5", "d2d4", "d7d6", "g1f3", "d6e5", "f3e5",
        ],
        3,
    );

    // Scandinavian Defense
    book.add_line(
        "Scandinavian Defense",
        STARTPOS_FEN,
        &[
            "e2e4", "d7d5", "e4d5", "d8d5", "b1c3", "d5d8", "d2d4", "g8f6", "g1f3",
        ],
        3,
    );

    // -------------------------------------------------------------------------
    // Additional French and Caro-Kann structures
    // -------------------------------------------------------------------------

    // French Defense: Advance Variation
    book.add_line(
        "French Defense: Advance Variation",
        STARTPOS_FEN,
        &[
            "e2e4", "e7e6", "d2d4", "d7d5", "e4e5", "c7c5", "c2c3", "b8c6", "g1f3", "d8b6", "f1d3",
        ],
        4,
    );

    // French Defense: Winawer Variation
    book.add_line(
        "French Defense: Winawer Variation",
        STARTPOS_FEN,
        &[
            "e2e4", "e7e6", "d2d4", "d7d5", "b1c3", "f8b4", "e4e5", "c7c5", "a2a3", "b4c3", "b2c3",
            "g8e7",
        ],
        4,
    );

    // Caro-Kann: Advance Variation
    book.add_line(
        "Caro-Kann: Advance Variation",
        STARTPOS_FEN,
        &[
            "e2e4", "c7c6", "d2d4", "d7d5", "e4e5", "c8f5", "g1f3", "e7e6", "f1e2", "c6c5", "e1g1",
            "b8c6",
        ],
        4,
    );

    // Caro-Kann: Panov Attack
    // Often produces an isolated queen pawn or hanging-pawn structure.
    book.add_line(
        "Caro-Kann: Panov Attack",
        STARTPOS_FEN,
        &[
            "e2e4", "c7c6", "d2d4", "d7d5", "e4d5", "c6d5", "c2c4", "g8f6", "b1c3", "e7e6", "g1f3",
            "f8e7",
        ],
        4,
    );

    // -------------------------------------------------------------------------
    // Additional Sicilian structures
    // -------------------------------------------------------------------------

    // Sicilian Sveshnikov
    book.add_line(
        "Sicilian Sveshnikov",
        STARTPOS_FEN,
        &[
            "e2e4", "c7c5", "g1f3", "b8c6", "d2d4", "c5d4", "f3d4", "g8f6", "b1c3", "e7e5", "d4b5",
            "d7d6", "c1g5", "a7a6", "b5a3", "b7b5",
        ],
        5,
    );

    // Classical Sicilian: Richter-Rauzer
    // Produces opposite-side castling and direct attacks.
    book.add_line(
        "Classical Sicilian: Richter-Rauzer",
        STARTPOS_FEN,
        &[
            "e2e4", "c7c5", "g1f3", "d7d6", "d2d4", "c5d4", "f3d4", "g8f6", "b1c3", "b8c6", "c1g5",
            "e7e6", "d1d2", "f8e7", "e1c1",
        ],
        5,
    );

    // Sicilian Alapin
    book.add_line(
        "Sicilian Alapin",
        STARTPOS_FEN,
        &[
            "e2e4", "c7c5", "c2c3", "g8f6", "e4e5", "f6d5", "d2d4", "c5d4", "g1f3", "b8c6", "c3d4",
            "d7d6",
        ],
        3,
    );

    // Closed Sicilian
    book.add_line(
        "Closed Sicilian",
        STARTPOS_FEN,
        &[
            "e2e4", "c7c5", "b1c3", "b8c6", "g2g3", "g7g6", "f1g2", "f8g7", "d2d3", "d7d6", "f2f4",
            "e7e5",
        ],
        4,
    );

    // -------------------------------------------------------------------------
    // Queen's Gambit family
    // -------------------------------------------------------------------------

    // Queen's Gambit Accepted
    book.add_line(
        "Queen's Gambit Accepted",
        STARTPOS_FEN,
        &[
            "d2d4", "d7d5", "c2c4", "d5c4", "g1f3", "g8f6", "e2e3", "e7e6", "f1c4", "c7c5", "e1g1",
            "a7a6",
        ],
        4,
    );

    // Slav Defense
    book.add_line(
        "Slav Defense",
        STARTPOS_FEN,
        &[
            "d2d4", "d7d5", "c2c4", "c7c6", "g1f3", "g8f6", "b1c3", "d5c4", "a2a4", "c8f5", "e2e3",
            "e7e6", "f1c4", "f8b4",
        ],
        4,
    );

    // Semi-Slav Defense
    book.add_line(
        "Semi-Slav Defense",
        STARTPOS_FEN,
        &[
            "d2d4", "d7d5", "c2c4", "e7e6", "b1c3", "g8f6", "g1f3", "c7c6", "e2e3", "b8d7", "f1d3",
            "d5c4", "d3c4", "b7b5",
        ],
        4,
    );

    // Queen's Gambit Declined: Exchange Variation
    book.add_line(
        "Queen's Gambit Declined: Exchange Variation",
        STARTPOS_FEN,
        &[
            "d2d4", "d7d5", "c2c4", "e7e6", "b1c3", "g8f6", "c4d5", "e6d5", "c1g5", "c7c6", "e2e3",
            "c8f5",
        ],
        3,
    );

    // Tarrasch Defense
    // Frequently creates an isolated black queen pawn.
    book.add_line(
        "Tarrasch Defense",
        STARTPOS_FEN,
        &[
            "d2d4", "d7d5", "c2c4", "e7e6", "b1c3", "c7c5", "c4d5", "e6d5", "g1f3", "b8c6", "g2g3",
            "g8f6",
        ],
        3,
    );

    // Catalan Opening
    book.add_line(
        "Catalan Opening",
        STARTPOS_FEN,
        &[
            "d2d4", "g8f6", "c2c4", "e7e6", "g2g3", "d7d5", "f1g2", "f8e7", "g1f3", "e8g8", "e1g1",
            "d5c4", "d1c2", "a7a6",
        ],
        4,
    );

    // -------------------------------------------------------------------------
    // Indian defenses and asymmetrical 1.d4 positions
    // -------------------------------------------------------------------------

    // Nimzo-Indian Defense
    book.add_line(
        "Nimzo-Indian Defense",
        STARTPOS_FEN,
        &[
            "d2d4", "g8f6", "c2c4", "e7e6", "b1c3", "f8b4", "e2e3", "e8g8", "f1d3", "d7d5", "g1f3",
            "c7c5", "e1g1",
        ],
        5,
    );

    // Queen's Indian Defense
    book.add_line(
        "Queen's Indian Defense",
        STARTPOS_FEN,
        &[
            "d2d4", "g8f6", "c2c4", "e7e6", "g1f3", "b7b6", "g2g3", "c8a6", "b2b3", "f8b4", "c1d2",
            "b4e7", "f1g2",
        ],
        4,
    );

    // Grünfeld Defense: Exchange Variation
    book.add_line(
        "Grünfeld Defense: Exchange Variation",
        STARTPOS_FEN,
        &[
            "d2d4", "g8f6", "c2c4", "g7g6", "b1c3", "d7d5", "c4d5", "f6d5", "e2e4", "d5c3", "b2c3",
            "f8g7",
        ],
        5,
    );

    // Modern Benoni
    book.add_line(
        "Modern Benoni",
        STARTPOS_FEN,
        &[
            "d2d4", "g8f6", "c2c4", "c7c5", "d4d5", "e7e6", "b1c3", "e6d5", "c4d5", "d7d6", "e2e4",
            "g7g6", "f2f4", "f8g7",
        ],
        5,
    );

    // Benko Gambit
    // Gives Black long-term queenside activity for a pawn.
    book.add_line(
        "Benko Gambit",
        STARTPOS_FEN,
        &[
            "d2d4", "g8f6", "c2c4", "c7c5", "d4d5", "b7b5", "c4b5", "a7a6", "b5a6", "g7g6", "b1c3",
            "c8a6",
        ],
        5,
    );

    // King's Indian: Sämisch Variation
    book.add_line(
        "King's Indian: Sämisch Variation",
        STARTPOS_FEN,
        &[
            "d2d4", "g8f6", "c2c4", "g7g6", "b1c3", "f8g7", "e2e4", "d7d6", "f2f3", "e8g8", "c1e3",
            "e7e5", "d4d5",
        ],
        4,
    );

    // Budapest Gambit
    book.add_line(
        "Budapest Gambit",
        STARTPOS_FEN,
        &[
            "d2d4", "g8f6", "c2c4", "e7e5", "d4e5", "f6g4", "g1f3", "b8c6", "c1f4", "f8b4", "b1d2",
            "d8e7",
        ],
        4,
    );

    // Dutch Defense: Leningrad
    book.add_line(
        "Dutch Defense: Leningrad",
        STARTPOS_FEN,
        &[
            "d2d4", "f7f5", "g2g3", "g8f6", "f1g2", "g7g6", "g1f3", "f8g7", "e1g1", "e8g8", "c2c4",
            "d7d6", "b1c3",
        ],
        4,
    );

    // -------------------------------------------------------------------------
    // Independent 1.d4 systems
    // -------------------------------------------------------------------------

    // London System
    book.add_line(
        "London System",
        STARTPOS_FEN,
        &[
            "d2d4", "d7d5", "g1f3", "g8f6", "c1f4", "e7e6", "e2e3", "f8d6", "f4g3", "e8g8", "f1d3",
            "c7c5",
        ],
        3,
    );

    // Trompowsky Attack
    book.add_line(
        "Trompowsky Attack",
        STARTPOS_FEN,
        &[
            "d2d4", "g8f6", "c1g5", "f6e4", "g5f4", "d7d5", "e2e3", "c7c5", "f1d3", "b8c6",
        ],
        3,
    );

    // Jobava London
    book.add_line(
        "Jobava London",
        STARTPOS_FEN,
        &[
            "d2d4", "d7d5", "b1c3", "g8f6", "c1f4", "c7c5", "e2e3", "b8c6", "c3b5", "e7e5", "d4e5",
        ],
        3,
    );

    // Blackmar-Diemer Gambit
    book.add_line(
        "Blackmar-Diemer Gambit",
        STARTPOS_FEN,
        &[
            "d2d4", "d7d5", "e2e4", "d5e4", "b1c3", "g8f6", "f2f3", "e4f3", "g1f3",
        ],
        3,
    );

    // Albin Countergambit
    book.add_line(
        "Albin Countergambit",
        STARTPOS_FEN,
        &[
            "d2d4", "d7d5", "c2c4", "e7e5", "d4e5", "d5d4", "g1f3", "b8c6", "a2a3", "c8e6",
        ],
        3,
    );

    // -------------------------------------------------------------------------
    // Flank openings
    // -------------------------------------------------------------------------

    // English Opening: Symmetrical Variation
    book.add_line(
        "English Opening: Symmetrical Variation",
        STARTPOS_FEN,
        &[
            "c2c4", "c7c5", "b1c3", "b8c6", "g2g3", "g7g6", "f1g2", "f8g7", "g1f3", "e7e5", "e1g1",
            "g8e7",
        ],
        4,
    );

    // English Opening: Botvinnik Setup
    book.add_line(
        "English Opening: Botvinnik Setup",
        STARTPOS_FEN,
        &[
            "c2c4", "e7e5", "b1c3", "b8c6", "g2g3", "g7g6", "f1g2", "f8g7", "e2e4", "d7d6", "g1e2",
        ],
        3,
    );

    // Réti Opening
    book.add_line(
        "Réti Opening",
        STARTPOS_FEN,
        &[
            "g1f3", "d7d5", "c2c4", "e7e6", "g2g3", "g8f6", "f1g2", "f8e7", "e1g1", "e8g8", "d2d4",
        ],
        4,
    );

    // King's Indian Attack
    book.add_line(
        "King's Indian Attack",
        STARTPOS_FEN,
        &[
            "g1f3", "d7d5", "g2g3", "g8f6", "f1g2", "e7e6", "e1g1", "f8e7", "d2d3", "e8g8", "b1d2",
        ],
        3,
    );

    // Bird Opening
    book.add_line(
        "Bird Opening",
        STARTPOS_FEN,
        &[
            "f2f4", "d7d5", "g1f3", "g8f6", "e2e3", "g7g6", "b2b3", "f8g7", "c1b2", "e8g8",
        ],
        2,
    );

    // Polish Opening / Sokolsky
    book.add_line(
        "Polish Opening / Sokolsky",
        STARTPOS_FEN,
        &[
            "b2b4", "e7e5", "c1b2", "f8b4", "b2e5", "g8f6", "g1f3", "e8g8", "e2e3", "d7d5",
        ],
        2,
    );

    // -------------------------------------------------------------------------
    // Short suggestion openings
    //
    // These intentionally stop after one to three plies. They seed a starting
    // idea, then let the engine decide how to continue without a long book line.
    // -------------------------------------------------------------------------

    book.add_line("Grob Opening", STARTPOS_FEN, &["g2g4"], 1);
    book.add_line("Barnes Opening", STARTPOS_FEN, &["f2f3"], 1);
    book.add_line("Van't Kruijs Opening", STARTPOS_FEN, &["e2e3"], 1);
    book.add_line("Saragossa Opening", STARTPOS_FEN, &["c2c3"], 1);
    book.add_line("Mieses Opening", STARTPOS_FEN, &["d2d3"], 1);
    book.add_line("Ware Opening", STARTPOS_FEN, &["a2a4"], 1);
    book.add_line("Anderssen Opening", STARTPOS_FEN, &["a2a3"], 1);
    book.add_line("Clemenz Opening", STARTPOS_FEN, &["h2h3"], 1);
    book.add_line("Desprez Opening", STARTPOS_FEN, &["h2h4"], 1);
    book.add_line("Durkin Opening", STARTPOS_FEN, &["b1a3"], 1);
    book.add_line("Amar Opening", STARTPOS_FEN, &["g1h3"], 1);
    book.add_line("Nimzowitsch-Larsen Attack", STARTPOS_FEN, &["b2b3"], 1);
    book.add_line("Owen's Defense", STARTPOS_FEN, &["e2e4", "b7b6"], 1);
    book.add_line("Nimzowitsch Defense", STARTPOS_FEN, &["e2e4", "b8c6"], 1);
    book.add_line("St. George Defense", STARTPOS_FEN, &["e2e4", "a7a6"], 1);
    book.add_line("Borg Defense", STARTPOS_FEN, &["e2e4", "g7g5"], 1);
    book.add_line("Goldsmith Defense", STARTPOS_FEN, &["e2e4", "h7h5"], 1);
    book.add_line("Carr Defense", STARTPOS_FEN, &["e2e4", "h7h6"], 1);
    book.add_line(
        "French Defense: Knight Suggestion",
        STARTPOS_FEN,
        &["e2e4", "e7e6", "g1f3"],
        1,
    );
    book.add_line(
        "Caro-Kann: Knight Suggestion",
        STARTPOS_FEN,
        &["e2e4", "c7c6", "g1f3"],
        1,
    );
    book.add_line(
        "Sicilian Defense: Wing Suggestion",
        STARTPOS_FEN,
        &["e2e4", "c7c5", "a2a3"],
        1,
    );
    book.add_line(
        "Scandinavian Defense: Exchange Suggestion",
        STARTPOS_FEN,
        &["e2e4", "d7d5", "e4d5"],
        1,
    );
    book.add_line(
        "Alekhine Defense: Advance Suggestion",
        STARTPOS_FEN,
        &["e2e4", "g8f6", "e4e5"],
        1,
    );
    book.add_line("Polish Defense", STARTPOS_FEN, &["d2d4", "b7b5"], 1);
    book.add_line("Old Benoni Defense", STARTPOS_FEN, &["d2d4", "c7c5"], 1);

    // -------------------------------------------------------------------------
    // Additional developed variations
    // -------------------------------------------------------------------------

    book.add_line(
        "Ruy Lopez: Berlin Defense",
        STARTPOS_FEN,
        &[
            "e2e4", "e7e5", "g1f3", "b8c6", "f1b5", "g8f6", "e1g1", "f6e4", "d2d4", "e4d6", "b5c6",
            "d7c6", "d4e5", "d6f5", "d1d8", "e8d8",
        ],
        4,
    );

    book.add_line(
        "Ruy Lopez: Exchange Variation",
        STARTPOS_FEN,
        &[
            "e2e4", "e7e5", "g1f3", "b8c6", "f1b5", "a7a6", "b5c6", "d7c6", "e1g1", "f7f6",
        ],
        3,
    );

    book.add_line(
        "Italian Game: Giuoco Pianissimo",
        STARTPOS_FEN,
        &[
            "e2e4", "e7e5", "g1f3", "b8c6", "f1c4", "f8c5", "d2d3", "g8f6", "e1g1", "d7d6", "c2c3",
            "e8g8", "f1e1", "a7a6",
        ],
        4,
    );

    book.add_line(
        "Italian Game: Two Knights Defense",
        STARTPOS_FEN,
        &[
            "e2e4", "e7e5", "g1f3", "b8c6", "f1c4", "g8f6", "f3g5", "d7d5", "e4d5", "c6a5", "c4b5",
            "c7c6", "d5c6", "b7c6", "b5e2", "h7h6", "g5f3", "e5e4", "f3e5", "f8d6", "d2d4", "e8g8",
        ],
        5,
    );

    book.add_line(
        "Center Game: Paulsen Attack",
        STARTPOS_FEN,
        &[
            "e2e4", "e7e5", "d2d4", "e5d4", "d1d4", "b8c6", "d4e3", "g8f6", "b1c3", "f8b4", "c1d2",
            "e8g8", "e1c1", "f8e8",
        ],
        3,
    );

    book.add_line(
        "Bishop's Opening: Berlin Defense",
        STARTPOS_FEN,
        &[
            "e2e4", "e7e5", "f1c4", "g8f6", "d2d3", "b8c6", "g1f3", "f8c5", "e1g1", "d7d6", "c2c3",
            "e8g8",
        ],
        3,
    );

    book.add_line(
        "Ponziani Opening: Jaenisch Counterattack",
        STARTPOS_FEN,
        &[
            "e2e4", "e7e5", "g1f3", "b8c6", "c2c3", "g8f6", "d2d4", "f6e4", "d4d5", "c6e7", "f3e5",
            "e7g6", "f1d3", "g6e5",
        ],
        3,
    );

    book.add_line(
        "Vienna Game: Max Lange Defense",
        STARTPOS_FEN,
        &[
            "e2e4", "e7e5", "b1c3", "g8f6", "f1c4", "b8c6", "d2d3", "f8b4", "g1e2", "d7d5", "e4d5",
            "f6d5", "e1g1", "c8e6",
        ],
        3,
    );

    book.add_line(
        "King's Gambit Declined: Classical Variation",
        STARTPOS_FEN,
        &[
            "e2e4", "e7e5", "f2f4", "f8c5", "g1f3", "d7d6", "c2c3", "g8f6", "d2d4", "e5d4", "c3d4",
            "c5b4", "c1d2", "b4d2", "b1d2", "e8g8",
        ],
        4,
    );

    book.add_line(
        "Sicilian Defense: Najdorf English Attack",
        STARTPOS_FEN,
        &[
            "e2e4", "c7c5", "g1f3", "d7d6", "d2d4", "c5d4", "f3d4", "g8f6", "b1c3", "a7a6", "c1e3",
            "e7e5", "d4b3", "c8e6", "f2f3", "f8e7", "d1d2", "e8g8",
        ],
        5,
    );

    book.add_line(
        "Sicilian Defense: Accelerated Dragon",
        STARTPOS_FEN,
        &[
            "e2e4", "c7c5", "g1f3", "b8c6", "d2d4", "c5d4", "f3d4", "g7g6", "c2c4", "f8g7", "c1e3",
            "g8f6", "b1c3", "f6g4",
        ],
        5,
    );

    book.add_line(
        "French Defense: Tarrasch Variation",
        STARTPOS_FEN,
        &[
            "e2e4", "e7e6", "d2d4", "d7d5", "b1d2", "g8f6", "e4e5", "f6d7", "f1d3", "c7c5", "c2c3",
            "b8c6", "g1e2", "c5d4", "c3d4",
        ],
        4,
    );

    book.add_line(
        "Caro-Kann Defense: Classical Variation",
        STARTPOS_FEN,
        &[
            "e2e4", "c7c6", "d2d4", "d7d5", "b1c3", "d5e4", "c3e4", "c8f5", "e4g3", "f5g6", "h2h4",
            "h7h6", "g1f3", "b8d7",
        ],
        4,
    );

    book.add_line(
        "Queen's Gambit Declined: Orthodox Defense",
        STARTPOS_FEN,
        &[
            "d2d4", "d7d5", "c2c4", "e7e6", "b1c3", "g8f6", "c1g5", "f8e7", "e2e3", "e8g8", "g1f3",
            "b8d7", "a1c1", "c7c6",
        ],
        4,
    );

    book.add_line(
        "Slav Defense: Exchange Variation",
        STARTPOS_FEN,
        &[
            "d2d4", "d7d5", "c2c4", "c7c6", "c4d5", "c6d5", "b1c3", "g8f6", "c1f4", "b8c6", "e2e3",
            "c8f5",
        ],
        3,
    );

    book.add_line(
        "Semi-Slav Defense: Meran Variation",
        STARTPOS_FEN,
        &[
            "d2d4", "d7d5", "c2c4", "e7e6", "b1c3", "g8f6", "g1f3", "c7c6", "e2e3", "b8d7", "f1d3",
            "d5c4", "d3c4", "b7b5", "c4d3", "a7a6", "e3e4", "c6c5",
        ],
        5,
    );

    book.add_line(
        "Nimzo-Indian Defense: Rubinstein Variation",
        STARTPOS_FEN,
        &[
            "d2d4", "g8f6", "c2c4", "e7e6", "b1c3", "f8b4", "e2e3", "e8g8", "f1d3", "d7d5", "g1f3",
            "c7c5", "e1g1", "b8c6",
        ],
        5,
    );

    book.add_line(
        "King's Indian Defense: Classical Variation",
        STARTPOS_FEN,
        &[
            "d2d4", "g8f6", "c2c4", "g7g6", "b1c3", "f8g7", "e2e4", "d7d6", "g1f3", "e8g8", "f1e2",
            "e7e5", "e1g1", "b8c6", "d4d5", "c6e7",
        ],
        5,
    );

    book.add_line(
        "English Opening: Four Knights Variation",
        STARTPOS_FEN,
        &[
            "c2c4", "e7e5", "b1c3", "g8f6", "g1f3", "b8c6", "g2g3", "f8b4", "f1g2", "e8g8", "e1g1",
            "d7d6",
        ],
        4,
    );

    book.add_line(
        "Dutch Defense: Stonewall Variation",
        STARTPOS_FEN,
        &[
            "d2d4", "f7f5", "g2g3", "g8f6", "f1g2", "e7e6", "g1f3", "d7d5", "e1g1", "f8d6", "c2c4",
            "c7c6", "b1c3", "e8g8",
        ],
        4,
    );

    book
}

pub const IMPORTANT_OPENINGS: &[&str] = &[
    "Italian Game",
    "Petrov Defense",
    "Evans Gambit",
    "Classical Sicilian: Richter-Rauzer",
    "French Defense: Advance Variation",
    "Caro-Kann: Panov Attack",
    "Queen's Gambit Declined: Exchange Variation",
    "Semi-Slav Defense",
    "Nimzo-Indian Defense",
    "Grünfeld Defense: Exchange Variation",
    "King's Indian: Sämisch Variation",
    "Modern Benoni",
    "English Opening: Symmetrical Variation",
    "Réti Opening",
];

pub fn build_important_opening_book() -> OpeningBook {
    build_opening_book().only_named(IMPORTANT_OPENINGS)
}

pub const SUGGESTION_OPENINGS: &[&str] = &[
    "Grob Opening",
    "Barnes Opening",
    "Van't Kruijs Opening",
    "Saragossa Opening",
    "Mieses Opening",
    "Ware Opening",
    "Anderssen Opening",
    "Clemenz Opening",
    "Desprez Opening",
    "Durkin Opening",
    "Amar Opening",
    "Nimzowitsch-Larsen Attack",
    "Owen's Defense",
    "Nimzowitsch Defense",
    "St. George Defense",
    "Borg Defense",
    "Goldsmith Defense",
    "Carr Defense",
    "French Defense: Knight Suggestion",
    "Caro-Kann: Knight Suggestion",
    "Sicilian Defense: Wing Suggestion",
    "Scandinavian Defense: Exchange Suggestion",
    "Alekhine Defense: Advance Suggestion",
    "Polish Defense",
    "Old Benoni Defense",
];

pub fn build_suggestion_opening_book() -> OpeningBook {
    build_opening_book().only_named(SUGGESTION_OPENINGS)
}
