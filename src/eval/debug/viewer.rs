use std::io::{self, Write};

use crate::board::{Board, STARTPOS_FEN};
use crate::eval::debug::breakdown::{
    KingBreakdown, ScoreLine, endgame_pst_lines, evaluation_breakdown, king_breakdown,
    knight_lines, material_lines, middlegame_pst_lines, mobility_lines, pawn_lines, slider_lines,
    summary_breakdown, tempo_lines,
};
use crate::eval::{EvalInfo, MAX_PHASE};
use crate::types::{Color, PieceType};

pub fn run_interactive() -> io::Result<()> {
    EvalViewer::default().run()
}

#[derive(Default)]
struct EvalViewer {
    position: Option<LoadedPosition>,
    comparison: Option<Comparison>,
}

struct LoadedPosition {
    fen: String,
    board: Board,
}

struct Comparison {
    first: LoadedPosition,
    second: LoadedPosition,
}

#[derive(Clone, Copy)]
enum Section {
    Summary,
    Material,
    Pst,
    Mobility,
    Pawns,
    Knights,
    Sliders,
    King,
    Tempo,
    All,
}

impl EvalViewer {
    fn run(mut self) -> io::Result<()> {
        println!();
        println!("============================================================");
        println!("                STATIC EVALUATION DEBUGGER");
        println!("============================================================");
        println!("Paste a six-field FEN to load and summarize a position.");
        println!("Type 'help' to show the available commands.");

        let stdin = io::stdin();
        let mut input = String::new();

        loop {
            print!("\neval> ");
            io::stdout().flush()?;

            input.clear();
            if stdin.read_line(&mut input)? == 0 {
                println!();
                break;
            }

            let input = input.trim();
            if input.is_empty() {
                continue;
            }

            if !self.handle_input(input, &stdin)? {
                break;
            }
        }

        Ok(())
    }

    fn handle_input(&mut self, input: &str, stdin: &io::Stdin) -> io::Result<bool> {
        let mut parts = input.split_whitespace();
        let first = parts.next().expect("non-empty input");
        let command = first.to_ascii_lowercase();

        match command.as_str() {
            "help" | "?" => Self::print_help(),
            "quit" | "exit" => return Ok(false),
            "fen" => {
                let fen = input[first.len()..].trim();
                if fen.is_empty() {
                    println!("Usage: fen <six-field FEN>");
                } else {
                    self.load_fen(fen);
                }
            }
            "startpos" | "start" => self.load_fen(STARTPOS_FEN),
            "summary" => self.print_section(Section::Summary),
            "current" => self.print_current(),
            "compare" => {
                let arguments = input[first.len()..].trim();
                self.handle_compare(arguments, stdin)?;
            }
            "show" => match (parts.next(), parts.next()) {
                (Some(section), None) => match Section::parse(section) {
                    Some(section) => self.print_section(section),
                    None => println!(
                        "Unknown evaluation section '{section}'. Type 'help' for sections."
                    ),
                },
                (None, _) => println!("Usage: show <section>"),
                (_, Some(_)) => println!("Usage: show <section>"),
            },
            _ => {
                if input.split_whitespace().count() == 6 {
                    self.load_fen(input);
                } else if let Some(section) = Section::parse(&command) {
                    self.print_section(section);
                } else {
                    println!("Unknown command '{first}'. Type 'help' for commands.");
                }
            }
        }

        Ok(true)
    }

    fn load_fen(&mut self, fen: &str) {
        let position = match parse_loaded_position(fen) {
            Ok(position) => position,
            Err(message) => {
                println!("Cannot evaluate FEN: {message}");
                return;
            }
        };

        self.position = Some(position);

        println!("\nPosition loaded.");
        self.print_section(Section::Summary);
    }

    fn handle_compare(&mut self, arguments: &str, stdin: &io::Stdin) -> io::Result<()> {
        if arguments.is_empty() {
            return self.prompt_for_comparison(stdin);
        }

        match split_comparison_input(arguments) {
            Ok(Some((first, second))) => self.load_comparison(&first, &second),
            Ok(None) => self.print_comparison_query(arguments),
            Err(message) => println!("{message}"),
        }

        Ok(())
    }

    fn prompt_for_comparison(&mut self, stdin: &io::Stdin) -> io::Result<()> {
        println!("Enter two FENs to compare. Submit an empty line to cancel.");

        let Some(first) = read_prompted_fen(stdin, "first FEN> ")? else {
            println!("Comparison cancelled.");
            return Ok(());
        };
        let Some(second) = read_prompted_fen(stdin, "second FEN> ")? else {
            println!("Comparison cancelled.");
            return Ok(());
        };

        self.load_comparison(&first, &second);
        Ok(())
    }

    fn load_comparison(&mut self, first_fen: &str, second_fen: &str) {
        let first = match parse_loaded_position(first_fen) {
            Ok(position) => position,
            Err(message) => {
                println!("Cannot evaluate Position 1: {message}");
                return;
            }
        };
        let second = match parse_loaded_position(second_fen) {
            Ok(position) => position,
            Err(message) => {
                println!("Cannot evaluate Position 2: {message}");
                return;
            }
        };

        self.comparison = Some(Comparison { first, second });
        println!("\nComparison loaded. Delta values are Position 2 - Position 1.");
        self.print_comparison_section(Section::Summary);
    }

    fn print_current(&self) {
        let Some(position) = &self.position else {
            Self::print_no_position();
            return;
        };

        println!("\nFEN: {}", position.fen);
        self.print_section(Section::Summary);
    }

    fn print_section(&self, section: Section) {
        let Some(position) = &self.position else {
            Self::print_no_position();
            return;
        };

        let board = &position.board;
        let info = EvalInfo::calculate(board);

        match section {
            Section::Summary => print_summary(board, &info),
            Section::Material => print_score_table("MATERIAL", &material_lines(board)),
            Section::Pst => print_pst(board, &info),
            Section::Mobility => print_score_table("MOBILITY", &mobility_lines(board, &info)),
            Section::Pawns => print_score_table("PAWNS", &pawn_lines(board, &info)),
            Section::Knights => print_score_table("KNIGHTS", &knight_lines(board, &info)),
            Section::Sliders => print_score_table("SLIDERS", &slider_lines(board, &info)),
            Section::King => print_king(board, &info),
            Section::Tempo => print_score_table("TEMPO", &tempo_lines(board)),
            Section::All => {
                print_summary(board, &info);
                print_score_table("MATERIAL", &material_lines(board));
                print_pst(board, &info);
                print_score_table("MOBILITY", &mobility_lines(board, &info));
                print_score_table("PAWNS", &pawn_lines(board, &info));
                print_score_table("KNIGHTS", &knight_lines(board, &info));
                print_score_table("SLIDERS", &slider_lines(board, &info));
                print_king(board, &info);
                print_score_table("TEMPO", &tempo_lines(board));
            }
        }
    }

    fn print_comparison_query(&self, query: &str) {
        if let Some(section) = Section::parse(query) {
            self.print_comparison_section(section);
            return;
        }

        let Some(comparison) = &self.comparison else {
            Self::print_no_comparison();
            return;
        };

        let first_info = EvalInfo::calculate(&comparison.first.board);
        let second_info = EvalInfo::calculate(&comparison.second.board);
        let first_lines = named_bonus_lines(&comparison.first.board, &first_info);
        let second_lines = named_bonus_lines(&comparison.second.board, &second_info);
        let matches = matching_bonus_indices(&first_lines, query);

        match matches.as_slice() {
            [] => println!(
                "Unknown bonus '{query}'. Use 'compare <section>' or a named row from a section."
            ),
            [index] => print_comparison_table(
                "NAMED BONUS COMPARISON",
                &first_lines[*index..=*index],
                &second_lines[*index..=*index],
                false,
            ),
            _ => {
                let names = matches
                    .iter()
                    .map(|index| first_lines[*index].name)
                    .collect::<Vec<_>>()
                    .join(", ");
                println!("Bonus name '{query}' is ambiguous. Matches: {names}");
            }
        }
    }

    fn print_comparison_section(&self, section: Section) {
        let Some(comparison) = &self.comparison else {
            Self::print_no_comparison();
            return;
        };

        let first = &comparison.first.board;
        let second = &comparison.second.board;
        let first_info = EvalInfo::calculate(first);
        let second_info = EvalInfo::calculate(second);

        println!();
        println!("Position 1: {}", comparison.first.fen);
        println!("Position 2: {}", comparison.second.fen);
        println!("Delta: Position 2 - Position 1");

        match section {
            Section::Summary => print_comparison_summary(first, &first_info, second, &second_info),
            Section::Material => print_comparison_table(
                "MATERIAL COMPARISON",
                &material_lines(first),
                &material_lines(second),
                true,
            ),
            Section::Pst => print_comparison_pst(first, &first_info, second, &second_info),
            Section::Mobility => print_comparison_table(
                "MOBILITY COMPARISON",
                &mobility_lines(first, &first_info),
                &mobility_lines(second, &second_info),
                true,
            ),
            Section::Pawns => print_comparison_table(
                "PAWN COMPARISON",
                &pawn_lines(first, &first_info),
                &pawn_lines(second, &second_info),
                true,
            ),
            Section::Knights => print_comparison_table(
                "KNIGHT COMPARISON",
                &knight_lines(first, &first_info),
                &knight_lines(second, &second_info),
                true,
            ),
            Section::Sliders => print_comparison_table(
                "SLIDER COMPARISON",
                &slider_lines(first, &first_info),
                &slider_lines(second, &second_info),
                true,
            ),
            Section::King => print_comparison_king(first, &first_info, second, &second_info),
            Section::Tempo => print_comparison_table(
                "TEMPO COMPARISON",
                &tempo_lines(first),
                &tempo_lines(second),
                true,
            ),
            Section::All => {
                print_comparison_summary(first, &first_info, second, &second_info);
                print_comparison_table(
                    "MATERIAL COMPARISON",
                    &material_lines(first),
                    &material_lines(second),
                    true,
                );
                print_comparison_pst(first, &first_info, second, &second_info);
                print_comparison_table(
                    "MOBILITY COMPARISON",
                    &mobility_lines(first, &first_info),
                    &mobility_lines(second, &second_info),
                    true,
                );
                print_comparison_table(
                    "PAWN COMPARISON",
                    &pawn_lines(first, &first_info),
                    &pawn_lines(second, &second_info),
                    true,
                );
                print_comparison_table(
                    "KNIGHT COMPARISON",
                    &knight_lines(first, &first_info),
                    &knight_lines(second, &second_info),
                    true,
                );
                print_comparison_table(
                    "SLIDER COMPARISON",
                    &slider_lines(first, &first_info),
                    &slider_lines(second, &second_info),
                    true,
                );
                print_comparison_king(first, &first_info, second, &second_info);
                print_comparison_table(
                    "TEMPO COMPARISON",
                    &tempo_lines(first),
                    &tempo_lines(second),
                    true,
                );
            }
        }
    }

    fn print_no_position() {
        println!("No position is loaded. Paste a FEN or type 'startpos'.");
    }

    fn print_no_comparison() {
        println!("No comparison is loaded. Type 'compare' and enter two FENs.");
    }

    fn print_help() {
        println!();
        println!("Commands");
        println!("  <FEN>            Load a pasted six-field FEN and print its summary");
        println!("  fen <FEN>        Load a FEN explicitly");
        println!("  startpos         Load the standard starting position");
        println!("  summary          Print the compact evaluation summary");
        println!("  current          Print the current FEN and summary");
        println!("  show <section>   Print every bonus in one evaluation section");
        println!("  compare          Prompt for two FENs and compare their summaries");
        println!("  compare <name>   Compare a section or named bonus for the loaded pair");
        println!("  compare FEN | FEN  Load two FENs without the guided prompts");
        println!("  help             Print this command list");
        println!("  quit             Close the debugger");
        println!();
        println!("Sections");
        println!("  material, pst, mobility, pawns, knights, sliders, king, tempo, all");
        println!();
        println!("Section names can also be entered directly, for example: king");
        println!("Comparison deltas are always Position 2 - Position 1.");
    }
}

impl Section {
    fn parse(value: &str) -> Option<Self> {
        match normalize_name(value).as_str() {
            "summary" => Some(Self::Summary),
            "material" => Some(Self::Material),
            "pst" | "psts" | "piecesquare" | "piecesquaretables" => Some(Self::Pst),
            "mobility" => Some(Self::Mobility),
            "pawn" | "pawns" => Some(Self::Pawns),
            "knight" | "knights" => Some(Self::Knights),
            "slider" | "sliders" | "bishop" | "bishops" | "rook" | "rooks" | "queen" | "queens" => {
                Some(Self::Sliders)
            }
            "king" | "kings" | "kingsafety" | "safety" => Some(Self::King),
            "tempo" => Some(Self::Tempo),
            "all" => Some(Self::All),
            _ => None,
        }
    }
}

fn print_summary(board: &Board, info: &EvalInfo) {
    let summary = summary_breakdown(board, info);

    println!();
    println!("COMPACT SUMMARY");
    println!("Side to move: {:?}", summary.side_to_move);
    println!("Phase: {}/{}", summary.phase, MAX_PHASE);
    println!("White/Black are each side's scores; Net is White - Black.");
    print_table_header();

    for line in &summary.components {
        print_score_line(line);
    }

    print_separator();
    print_score_line(&summary.total);
    println!(
        "Side-to-move evaluation: {:+} ({:?})",
        summary.side_to_move_total, summary.side_to_move
    );
}

fn print_comparison_summary(
    first: &Board,
    first_info: &EvalInfo,
    second: &Board,
    second_info: &EvalInfo,
) {
    let first_summary = summary_breakdown(first, first_info);
    let second_summary = summary_breakdown(second, second_info);
    let first_phase = first_summary.phase;
    let second_phase = second_summary.phase;
    let first_side = first_summary.side_to_move;
    let second_side = second_summary.side_to_move;
    let first_stm = first_summary.side_to_move_total;
    let second_stm = second_summary.side_to_move_total;

    let mut first_lines = first_summary.components;
    first_lines.push(first_summary.total);
    let mut second_lines = second_summary.components;
    second_lines.push(second_summary.total);

    print_comparison_table(
        "OVERALL EVALUATION COMPARISON",
        &first_lines,
        &second_lines,
        false,
    );
    println!(
        "Phase: Position 1 {first_phase}/{MAX_PHASE}, Position 2 {second_phase}/{MAX_PHASE}, delta {:+}",
        second_phase - first_phase
    );
    println!(
        "Side-to-move eval: Position 1 {first_stm:+} ({first_side:?}), Position 2 {second_stm:+} ({second_side:?}), delta {:+}",
        second_stm - first_stm
    );
}

fn print_pst(board: &Board, info: &EvalInfo) {
    print_score_table(
        "MIDDLEGAME PIECE-SQUARE TABLES",
        &middlegame_pst_lines(board),
    );
    print_score_table("ENDGAME PIECE-SQUARE TABLES", &endgame_pst_lines(board));

    let summary = summary_breakdown(board, info);
    let tapered = &summary.components[1];
    println!();
    println!("TAPERED PST (phase {}/{})", summary.phase, MAX_PHASE);
    print_table_header();
    print_score_line(tapered);
}

fn print_comparison_pst(
    first: &Board,
    first_info: &EvalInfo,
    second: &Board,
    second_info: &EvalInfo,
) {
    print_comparison_table(
        "MIDDLEGAME PST COMPARISON",
        &middlegame_pst_lines(first),
        &middlegame_pst_lines(second),
        true,
    );
    print_comparison_table(
        "ENDGAME PST COMPARISON",
        &endgame_pst_lines(first),
        &endgame_pst_lines(second),
        true,
    );

    let first_summary = summary_breakdown(first, first_info);
    let second_summary = summary_breakdown(second, second_info);
    print_comparison_table(
        "TAPERED PST COMPARISON",
        std::slice::from_ref(&first_summary.components[1]),
        std::slice::from_ref(&second_summary.components[1]),
        false,
    );
}

fn print_king(board: &Board, info: &EvalInfo) {
    let breakdown = king_breakdown(board, info);

    println!();
    println!("KING SAFETY - DANGER INPUTS");
    println!("Higher values mean greater danger to that king.");
    println!("{:<30} {:>12} {:>12}", "Factor", "White king", "Black king");
    println!("{:-<56}", "");
    print_king_danger_line(
        "Attacker weight",
        breakdown.white.attacker_weight,
        breakdown.black.attacker_weight,
    );
    print_king_danger_line(
        "Pawn shield",
        breakdown.white.pawn_shield,
        breakdown.black.pawn_shield,
    );
    print_king_danger_line(
        "Open files",
        breakdown.white.open_files,
        breakdown.black.open_files,
    );
    print_king_danger_line(
        "Open diagonals",
        breakdown.white.open_diagonals,
        breakdown.black.open_diagonals,
    );
    print_king_danger_line(
        "Escape squares",
        breakdown.white.escape_squares,
        breakdown.black.escape_squares,
    );
    print_king_danger_line(
        "Defenders",
        breakdown.white.defenders,
        breakdown.black.defenders,
    );
    println!("{:-<56}", "");
    print_king_danger_line(
        "Raw danger",
        breakdown.white.raw_danger,
        breakdown.black.raw_danger,
    );
    print_king_danger_line(
        "Clamped danger index",
        breakdown.white.clamped_danger,
        breakdown.black.clamped_danger,
    );

    println!();
    println!("KING SAFETY - FINAL SCORE");
    println!(
        "{:<30} {:>12} {:>12} {:>12}",
        "Stage", "White", "Black", "Net W-B"
    );
    println!("{:-<69}", "");
    print_king_score_line(
        "Danger table penalty",
        breakdown.white.table_penalty,
        breakdown.black.table_penalty,
    );
    print_king_score_line(
        "Phase-scaled king score",
        breakdown.white.final_score,
        breakdown.black.final_score,
    );

    let compact = evaluation_breakdown(board);
    debug_assert_eq!(
        breakdown.white.final_score - breakdown.black.final_score,
        compact.king
    );
}

fn print_comparison_king(
    first: &Board,
    first_info: &EvalInfo,
    second: &Board,
    second_info: &EvalInfo,
) {
    let first_breakdown = king_breakdown(first, first_info);
    let second_breakdown = king_breakdown(second, second_info);

    println!("\nKing danger rows use White king danger - Black king danger as Net.");
    print_comparison_table(
        "KING DANGER INPUT COMPARISON",
        &king_danger_lines(first_breakdown),
        &king_danger_lines(second_breakdown),
        false,
    );
    print_comparison_table(
        "KING FINAL SCORE COMPARISON",
        &king_score_lines(first_breakdown),
        &king_score_lines(second_breakdown),
        false,
    );
}

fn print_score_table(title: &str, lines: &[ScoreLine]) {
    println!();
    println!("{title}");
    println!("White/Black are each side's scores; Net is White - Black.");
    print_table_header();

    for line in lines {
        print_score_line(line);
    }

    let total = ScoreLine {
        name: "Section total",
        white: lines.iter().map(|line| line.white).sum(),
        black: lines.iter().map(|line| line.black).sum(),
        net: lines.iter().map(|line| line.net).sum(),
    };
    print_separator();
    print_score_line(&total);
}

fn print_comparison_table(
    title: &str,
    first: &[ScoreLine],
    second: &[ScoreLine],
    include_total: bool,
) {
    debug_assert_eq!(first.len(), second.len());
    debug_assert!(
        first
            .iter()
            .zip(second)
            .all(|(first, second)| first.name == second.name)
    );

    println!();
    println!("{title}");
    println!(
        "{:<28} {:^26} {:^26} {:>10}",
        "", "Position 1", "Position 2", "Delta Net"
    );
    println!(
        "{:<28} {:>8} {:>8} {:>8} {:>8} {:>8} {:>8} {:>10}",
        "Feature", "White", "Black", "Net", "White", "Black", "Net", "P2 - P1"
    );
    print_comparison_separator();

    for (first, second) in first.iter().zip(second) {
        print_comparison_line(first, second);
    }

    if include_total {
        let first_total = section_total(first);
        let second_total = section_total(second);
        print_comparison_separator();
        print_comparison_line(&first_total, &second_total);
    }
}

fn print_comparison_line(first: &ScoreLine, second: &ScoreLine) {
    println!(
        "{:<28} {:>+8} {:>+8} {:>+8} {:>+8} {:>+8} {:>+8} {:>+10}",
        first.name,
        first.white,
        first.black,
        first.net,
        second.white,
        second.black,
        second.net,
        second.net - first.net,
    );
}

fn print_comparison_separator() {
    println!("{:-<95}", "");
}

fn section_total(lines: &[ScoreLine]) -> ScoreLine {
    ScoreLine {
        name: "Section total",
        white: lines.iter().map(|line| line.white).sum(),
        black: lines.iter().map(|line| line.black).sum(),
        net: lines.iter().map(|line| line.net).sum(),
    }
}

fn print_table_header() {
    println!(
        "{:<30} {:>12} {:>12} {:>12}",
        "Feature", "White", "Black", "Net W-B"
    );
    print_separator();
}

fn print_separator() {
    println!("{:-<69}", "");
}

fn print_score_line(line: &ScoreLine) {
    println!(
        "{:<30} {:>+12} {:>+12} {:>+12}",
        line.name, line.white, line.black, line.net
    );
}

fn print_king_danger_line(name: &str, white: i32, black: i32) {
    println!("{name:<30} {white:>+12} {black:>+12}");
}

fn print_king_score_line(name: &str, white: i32, black: i32) {
    println!(
        "{name:<30} {white:>+12} {black:>+12} {:>+12}",
        white - black
    );
}

fn king_danger_lines(breakdown: KingBreakdown) -> Vec<ScoreLine> {
    vec![
        score_line(
            "Attacker weight",
            breakdown.white.attacker_weight,
            breakdown.black.attacker_weight,
        ),
        score_line(
            "Pawn shield",
            breakdown.white.pawn_shield,
            breakdown.black.pawn_shield,
        ),
        score_line(
            "Open files",
            breakdown.white.open_files,
            breakdown.black.open_files,
        ),
        score_line(
            "Open diagonals",
            breakdown.white.open_diagonals,
            breakdown.black.open_diagonals,
        ),
        score_line(
            "Escape squares",
            breakdown.white.escape_squares,
            breakdown.black.escape_squares,
        ),
        score_line(
            "Defenders",
            breakdown.white.defenders,
            breakdown.black.defenders,
        ),
        score_line(
            "Raw danger",
            breakdown.white.raw_danger,
            breakdown.black.raw_danger,
        ),
        score_line(
            "Clamped danger index",
            breakdown.white.clamped_danger,
            breakdown.black.clamped_danger,
        ),
    ]
}

fn king_score_lines(breakdown: KingBreakdown) -> Vec<ScoreLine> {
    vec![
        score_line(
            "Danger table penalty",
            breakdown.white.table_penalty,
            breakdown.black.table_penalty,
        ),
        score_line(
            "Phase-scaled king score",
            breakdown.white.final_score,
            breakdown.black.final_score,
        ),
    ]
}

fn score_line(name: &'static str, white: i32, black: i32) -> ScoreLine {
    ScoreLine {
        name,
        white,
        black,
        net: white - black,
    }
}

fn named_bonus_lines(board: &Board, info: &EvalInfo) -> Vec<ScoreLine> {
    let mut lines = mobility_lines(board, info);
    lines.extend(pawn_lines(board, info));
    lines.extend(knight_lines(board, info));
    lines.extend(slider_lines(board, info));

    let king = king_breakdown(board, info);
    lines.extend(king_danger_lines(king));
    lines.extend(king_score_lines(king));
    lines.extend(tempo_lines(board));
    lines
}

fn matching_bonus_indices(lines: &[ScoreLine], query: &str) -> Vec<usize> {
    let query = normalize_name(query);
    let exact = lines
        .iter()
        .enumerate()
        .filter_map(|(index, line)| (normalize_name(line.name) == query).then_some(index))
        .collect::<Vec<_>>();

    if !exact.is_empty() {
        return exact;
    }

    lines
        .iter()
        .enumerate()
        .filter_map(|(index, line)| normalize_name(line.name).contains(&query).then_some(index))
        .collect()
}

fn normalize_name(value: &str) -> String {
    value
        .chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

fn split_comparison_input(input: &str) -> Result<Option<(String, String)>, &'static str> {
    if input.contains('|') {
        let mut fens = input.split('|');
        let first = fens.next().unwrap_or_default().trim();
        let second = fens.next().unwrap_or_default().trim();

        if first.is_empty() || second.is_empty() || fens.next().is_some() {
            return Err("Usage: compare <FEN 1> | <FEN 2>");
        }

        return Ok(Some((first.to_owned(), second.to_owned())));
    }

    let fields = input.split_whitespace().collect::<Vec<_>>();
    if fields.len() == 12 {
        return Ok(Some((fields[..6].join(" "), fields[6..].join(" "))));
    }

    Ok(None)
}

fn read_prompted_fen(stdin: &io::Stdin, prompt: &str) -> io::Result<Option<String>> {
    print!("{prompt}");
    io::stdout().flush()?;

    let mut fen = String::new();
    if stdin.read_line(&mut fen)? == 0 {
        return Ok(None);
    }

    let fen = fen.trim();
    if fen.is_empty() {
        Ok(None)
    } else {
        Ok(Some(fen.to_owned()))
    }
}

fn parse_loaded_position(fen: &str) -> Result<LoadedPosition, String> {
    let board = Board::from_fen(fen).map_err(|message| format!("invalid FEN: {message}"))?;
    validate_for_evaluation(&board)?;

    Ok(LoadedPosition {
        fen: fen.to_owned(),
        board,
    })
}

fn validate_for_evaluation(board: &Board) -> Result<(), String> {
    for color in [Color::White, Color::Black] {
        let king_count = board.pieces(color, PieceType::King).count_ones();
        if king_count != 1 {
            return Err(format!(
                "expected exactly one {color:?} king, found {king_count}"
            ));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn section_aliases_are_supported() {
        assert!(matches!(Section::parse("pawns"), Some(Section::Pawns)));
        assert!(matches!(Section::parse("king-safety"), Some(Section::King)));
        assert!(matches!(Section::parse("bishop"), Some(Section::Sliders)));
        assert!(Section::parse("unknown").is_none());
    }

    #[test]
    fn positions_without_exactly_one_king_per_side_are_rejected() {
        let missing_black = Board::from_fen("8/8/8/8/8/8/8/4K3 w - - 0 1").unwrap();
        assert!(validate_for_evaluation(&missing_black).is_err());

        let start = Board::from_fen(STARTPOS_FEN).unwrap();
        assert!(validate_for_evaluation(&start).is_ok());
    }

    #[test]
    fn comparison_input_accepts_pipe_or_twelve_fen_fields() {
        let other = "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq - 0 1";
        let piped = format!("{STARTPOS_FEN} | {other}");
        let unseparated = format!("{STARTPOS_FEN} {other}");

        assert_eq!(
            split_comparison_input(&piped).unwrap(),
            Some((STARTPOS_FEN.to_owned(), other.to_owned()))
        );
        assert_eq!(
            split_comparison_input(&unseparated).unwrap(),
            Some((STARTPOS_FEN.to_owned(), other.to_owned()))
        );
        assert_eq!(split_comparison_input("pawn shield").unwrap(), None);
        assert!(split_comparison_input("fen | ").is_err());
    }

    #[test]
    fn named_bonus_matching_supports_full_and_partial_names() {
        let board = Board::from_fen(STARTPOS_FEN).unwrap();
        let info = EvalInfo::calculate(&board);
        let lines = named_bonus_lines(&board, &info);

        let pawn_shield = matching_bonus_indices(&lines, "pawn shield");
        assert_eq!(pawn_shield.len(), 1);
        assert_eq!(lines[pawn_shield[0]].name, "Pawn shield");

        let connected = matching_bonus_indices(&lines, "connected diagonals");
        assert_eq!(connected.len(), 1);
        assert_eq!(lines[connected[0]].name, "Connected diagonals (B/Q)");
    }
}
