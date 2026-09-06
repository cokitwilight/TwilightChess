use std::io::{self, BufRead, BufWriter, Write};

use crate::board::{Board, STARTPOS_FEN};
use crate::engine::configs::EngineConfig;
use crate::engine::{CHECKMATE_SCORE, Engine, MATE_THRESHOLD, MAX_PLY, SearchLimits, SearchResult};
use crate::types::Color;
use crate::uci::parse_uci_move;

const ENGINE_NAME: &str = "Twilight Chess";
const ENGINE_AUTHOR: &str = "Twilight Chess contributors";
const MAX_HASH_MB: usize = 4096;
const DEFAULT_MOVES_TO_GO: u64 = 30;

/// Runs a UCI session using any buffered input and writable output.
///
/// This form is useful for tests and for embedding the engine. Most callers
/// should use [`run_uci_stdio`].
pub fn run_uci<R: BufRead, W: Write>(reader: R, writer: W) -> io::Result<()> {
    run_uci_with_config(reader, writer, EngineConfig::default())
}

/// Runs the UCI protocol over this process's standard input and output.
pub fn run_uci_stdio() -> io::Result<()> {
    let stdin = io::stdin();
    let stdout = io::stdout();

    run_uci(stdin.lock(), BufWriter::new(stdout.lock()))
}

fn run_uci_with_config<R: BufRead, W: Write>(
    reader: R,
    mut writer: W,
    config: EngineConfig,
) -> io::Result<()> {
    let mut session = UciSession::new(config);

    for line in reader.lines() {
        let line = line?;
        let should_quit = session.process_command(&line, &mut writer)?;
        writer.flush()?;

        if should_quit {
            break;
        }
    }

    Ok(())
}

struct UciSession {
    engine: Engine,
    position: UciPosition,
    own_book: bool,
    debug: bool,
}

impl UciSession {
    fn new(config: EngineConfig) -> Self {
        Self {
            engine: Engine::new(config),
            position: UciPosition::startpos(),
            own_book: false,
            debug: false,
        }
    }

    /// Returns true when the caller should terminate the protocol loop.
    fn process_command<W: Write>(&mut self, line: &str, output: &mut W) -> io::Result<bool> {
        let line = line.trim();
        if line.is_empty() {
            return Ok(false);
        }

        let mut parts = line.splitn(2, char::is_whitespace);
        let command = parts.next().unwrap_or_default().to_ascii_lowercase();
        let arguments = parts.next().unwrap_or_default().trim();

        match command.as_str() {
            "uci" => self.write_identification(output)?,
            "isready" => writeln!(output, "readyok")?,
            "debug" => self.set_debug(arguments, output)?,
            "setoption" => self.set_option(arguments, output)?,
            "ucinewgame" => self.new_game(),
            "position" => {
                if let Err(error) = self.set_position(arguments) {
                    writeln!(output, "info string position error: {error}")?;
                }
            }
            "go" => {
                if let Err(error) = self.go(arguments, output) {
                    writeln!(output, "info string go error: {error}")?;
                }
            }
            // Searches are currently synchronous, so a queued stop can only be
            // observed after bestmove has already been emitted.
            "stop" | "ponderhit" | "register" => {}
            "quit" => return Ok(true),
            _ if self.debug => {
                writeln!(output, "info string ignored unknown command: {command}")?;
            }
            _ => {}
        }

        Ok(false)
    }

    fn write_identification<W: Write>(&self, output: &mut W) -> io::Result<()> {
        writeln!(
            output,
            "id name {ENGINE_NAME} {}",
            env!("CARGO_PKG_VERSION")
        )?;
        writeln!(output, "id author {ENGINE_AUTHOR}")?;
        writeln!(
            output,
            "option name Hash type spin default {} min 1 max {MAX_HASH_MB}",
            self.engine.config.tt_size
        )?;
        writeln!(output, "option name Clear Hash type button")?;
        writeln!(output, "option name OwnBook type check default false")?;
        writeln!(output, "uciok")
    }

    fn set_debug<W: Write>(&mut self, arguments: &str, output: &mut W) -> io::Result<()> {
        match arguments.to_ascii_lowercase().as_str() {
            "on" => self.debug = true,
            "off" => self.debug = false,
            _ => writeln!(output, "info string debug expects `on` or `off`")?,
        }

        Ok(())
    }

    fn set_option<W: Write>(&mut self, arguments: &str, output: &mut W) -> io::Result<()> {
        let Some(option) = arguments.strip_prefix("name ") else {
            writeln!(output, "info string setoption expects `name <option>`")?;
            return Ok(());
        };

        let (name, value) = match option.split_once(" value ") {
            Some((name, value)) => (name.trim(), Some(value.trim())),
            None => (option.trim(), None),
        };

        match name.to_ascii_lowercase().as_str() {
            "hash" => {
                let Some(value) = value else {
                    writeln!(output, "info string Hash requires a value")?;
                    return Ok(());
                };

                match value.parse::<usize>() {
                    Ok(hash_mb) if (1..=MAX_HASH_MB).contains(&hash_mb) => {
                        let mut config = self.engine.config;
                        config.tt_size = hash_mb;
                        self.engine = Engine::new(config);
                    }
                    _ => writeln!(
                        output,
                        "info string Hash must be between 1 and {MAX_HASH_MB} MB"
                    )?,
                }
            }
            "clear hash" => self.engine.tt.clear(),
            "ownbook" => match value.map(str::to_ascii_lowercase).as_deref() {
                Some("true") => self.own_book = true,
                Some("false") => self.own_book = false,
                _ => writeln!(output, "info string OwnBook expects true or false")?,
            },
            _ if self.debug => writeln!(output, "info string ignored unknown option: {name}")?,
            _ => {}
        }

        Ok(())
    }

    fn new_game(&mut self) {
        self.position = UciPosition::startpos();
        self.engine.tt.clear();
    }

    fn set_position(&mut self, arguments: &str) -> Result<(), String> {
        let position = UciPosition::parse(arguments)?;
        self.position = position;
        Ok(())
    }

    fn go<W: Write>(&mut self, arguments: &str, output: &mut W) -> io::Result<()> {
        let limits = GoParameters::parse(arguments)
            .and_then(|parameters| {
                parameters.to_limits(
                    self.position.board.side_to_move(),
                    self.engine.config.limits,
                )
            })
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidInput, error))?;

        let result = self.engine.search(
            &self.position.board,
            limits,
            &self.position.repetition_history,
            self.own_book,
            false,
        );

        write_search_info(output, &result)?;

        match result.best_move {
            Some(best_move) => writeln!(output, "bestmove {best_move}"),
            None => writeln!(output, "bestmove 0000"),
        }
    }
}

struct UciPosition {
    board: Board,
    repetition_history: Vec<u64>,
}

impl UciPosition {
    fn startpos() -> Self {
        Self::from_fen(STARTPOS_FEN).expect("the built-in starting FEN must be valid")
    }

    fn from_fen(fen: &str) -> Result<Self, String> {
        let board = Board::from_fen(fen)?;
        let hash = board.hash();

        Ok(Self {
            board,
            repetition_history: vec![hash],
        })
    }

    fn parse(arguments: &str) -> Result<Self, String> {
        let mut tokens = arguments.split_whitespace();

        let mut position = match tokens.next() {
            Some("startpos") => Self::startpos(),
            Some("fen") => {
                let fen_fields = (0..6)
                    .map(|_| {
                        tokens
                            .next()
                            .ok_or_else(|| "FEN requires 6 fields".to_string())
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                Self::from_fen(&fen_fields.join(" "))?
            }
            Some(other) => return Err(format!("expected `startpos` or `fen`, found `{other}`")),
            None => return Err("missing position".to_string()),
        };

        match tokens.next() {
            None => return Ok(position),
            Some("moves") => {}
            Some(other) => return Err(format!("expected `moves`, found `{other}`")),
        }

        for move_text in tokens {
            let mv = parse_uci_move(&position.board, move_text)?;
            position.board.make_move(mv);
            position.repetition_history.push(position.board.hash());
        }

        Ok(position)
    }
}

#[derive(Default)]
struct GoParameters {
    wtime: Option<u64>,
    btime: Option<u64>,
    winc: Option<u64>,
    binc: Option<u64>,
    moves_to_go: Option<u64>,
    depth: Option<u16>,
    nodes: Option<u64>,
    move_time: Option<u64>,
}

impl GoParameters {
    fn parse(arguments: &str) -> Result<Self, String> {
        let mut parameters = Self::default();
        let mut tokens = arguments.split_whitespace();

        while let Some(name) = tokens.next() {
            let lower_name = name.to_ascii_lowercase();

            match lower_name.as_str() {
                "wtime" => parameters.wtime = Some(parse_u64_value(&mut tokens, name)?),
                "btime" => parameters.btime = Some(parse_u64_value(&mut tokens, name)?),
                "winc" => parameters.winc = Some(parse_u64_value(&mut tokens, name)?),
                "binc" => parameters.binc = Some(parse_u64_value(&mut tokens, name)?),
                "movestogo" => parameters.moves_to_go = Some(parse_u64_value(&mut tokens, name)?),
                "depth" => {
                    let depth = parse_u64_value(&mut tokens, name)?;
                    parameters.depth = Some(depth.min((MAX_PLY - 1) as u64) as u16);
                }
                "nodes" => parameters.nodes = Some(parse_u64_value(&mut tokens, name)?),
                "movetime" => parameters.move_time = Some(parse_u64_value(&mut tokens, name)?),
                "infinite" | "ponder" | "searchmoves" | "mate" => {
                    return Err(format!("unsupported go option `{name}`"));
                }
                _ => return Err(format!("unknown go option `{name}`")),
            }
        }

        Ok(parameters)
    }

    fn to_limits(
        &self,
        side_to_move: Color,
        defaults: SearchLimits,
    ) -> Result<SearchLimits, String> {
        if self.moves_to_go == Some(0) {
            return Err("movestogo must be greater than zero".to_string());
        }

        let time_limit = self.move_time.or_else(|| {
            let (remaining, increment) = match side_to_move {
                Color::White => (self.wtime, self.winc.unwrap_or(0)),
                Color::Black => (self.btime, self.binc.unwrap_or(0)),
            };

            remaining.map(|remaining| {
                allocate_time(
                    remaining,
                    increment,
                    self.moves_to_go.unwrap_or(DEFAULT_MOVES_TO_GO),
                )
            })
        });

        let has_open_ended_limit = self.nodes.is_some() || time_limit.is_some();
        let default_depth = if has_open_ended_limit {
            (MAX_PLY - 1) as u16
        } else {
            defaults.max_depth
        };
        let max_depth = self.depth.unwrap_or(default_depth);

        Ok(SearchLimits {
            max_depth,
            max_q_depth: defaults.max_q_depth,
            max_nodes: self.nodes,
            soft_time_limit_ms: time_limit,
            hard_time_limit_ms: time_limit,
        })
    }
}

fn parse_u64_value<'a>(
    tokens: &mut impl Iterator<Item = &'a str>,
    name: &str,
) -> Result<u64, String> {
    let value = tokens
        .next()
        .ok_or_else(|| format!("missing value after `{name}`"))?;

    value
        .parse::<u64>()
        .map_err(|_| format!("invalid value `{value}` after `{name}`"))
}

fn allocate_time(remaining_ms: u64, increment_ms: u64, moves_to_go: u64) -> u64 {
    if remaining_ms == 0 {
        return 0;
    }

    // Keep a small reserve for protocol and scheduling overhead, then spend an
    // even share of the remaining moves plus most of the next increment.
    let reserve_ms = (remaining_ms / 20).clamp(1, 100);
    let spendable_ms = remaining_ms.saturating_sub(reserve_ms);
    let base_ms = spendable_ms / moves_to_go.max(1);
    let increment_share_ms = increment_ms.saturating_mul(3) / 4;

    base_ms
        .saturating_add(increment_share_ms)
        .min(spendable_ms)
        .max(1)
}

fn write_search_info<W: Write>(output: &mut W, result: &SearchResult) -> io::Result<()> {
    let nodes = result.stats.total_nodes();
    let time_ms = result.elapsed.as_millis().min(u64::MAX as u128) as u64;
    let nps = nodes.saturating_mul(1000).checked_div(time_ms).unwrap_or(0);

    write!(
        output,
        "info depth {} score {} nodes {nodes} time {time_ms} nps {nps}",
        result.depth_reached,
        format_uci_score(result.eval)
    )?;

    if let Some(pv_move) = result.pv[0].or(result.best_move) {
        write!(output, " pv {pv_move}")?;
    }

    writeln!(output)
}

fn format_uci_score(score: i32) -> String {
    if score.abs() < MATE_THRESHOLD {
        return format!("cp {score}");
    }

    let plies = (CHECKMATE_SCORE - score.abs()).max(0);
    let moves = (plies + 1) / 2;

    if moves == 0 {
        "mate 0".to_string()
    } else if score >= 0 {
        format!("mate {moves}")
    } else {
        format!("mate -{moves}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn test_config() -> EngineConfig {
        EngineConfig {
            tt_size: 1,
            limits: SearchLimits::depth(2, 2),
            ..EngineConfig::default()
        }
    }

    #[test]
    fn handshake_and_readiness_are_reported() {
        let input = Cursor::new("uci\nisready\nquit\n");
        let mut output = Vec::new();

        run_uci_with_config(input, &mut output, test_config()).unwrap();

        let output = String::from_utf8(output).unwrap();
        assert!(output.contains("id name Twilight Chess"));
        assert!(output.contains("option name Hash type spin default 1"));
        assert!(output.contains("uciok\nreadyok\n"));
    }

    #[test]
    fn position_applies_startpos_and_fen_move_lists() {
        let mut session = UciSession::new(test_config());
        let mut output = Vec::new();

        session
            .process_command("position startpos moves e2e4 e7e5 g1f3", &mut output)
            .unwrap();
        assert_eq!(session.position.repetition_history.len(), 4);
        assert_eq!(session.position.board.side_to_move(), Color::Black);

        session
            .process_command(
                "position fen 4k3/8/8/3pP3/8/8/8/4K3 w - d6 0 1 moves e5d6",
                &mut output,
            )
            .unwrap();
        assert_eq!(session.position.repetition_history.len(), 2);
        assert_eq!(session.position.board.side_to_move(), Color::Black);
    }

    #[test]
    fn invalid_position_does_not_replace_current_position() {
        let mut session = UciSession::new(test_config());
        let original_hash = session.position.board.hash();
        let mut output = Vec::new();

        session
            .process_command("position startpos moves e2e5", &mut output)
            .unwrap();

        assert_eq!(session.position.board.hash(), original_hash);
        assert!(
            String::from_utf8(output)
                .unwrap()
                .contains("position error")
        );
    }

    #[test]
    fn go_parameters_build_combined_limits() {
        let parameters = GoParameters::parse(
            "wtime 60000 btime 50000 winc 1000 binc 500 movestogo 20 depth 12 nodes 100000",
        )
        .unwrap();
        let limits = parameters
            .to_limits(Color::White, SearchLimits::depth(8, 6))
            .unwrap();

        assert_eq!(limits.max_depth, 12);
        assert_eq!(limits.max_q_depth, 6);
        assert_eq!(limits.max_nodes, Some(100000));
        assert!(limits.soft_time_limit_ms.is_some_and(|time| time < 60000));
    }

    #[test]
    fn depth_one_search_emits_info_and_bestmove() {
        let input = Cursor::new("position startpos\ngo depth 1\nquit\n");
        let mut output = Vec::new();

        run_uci_with_config(input, &mut output, test_config()).unwrap();

        let output = String::from_utf8(output).unwrap();
        assert!(output.contains("info depth 1 score "));
        assert!(output.contains("bestmove "));
        assert!(!output.contains("bestmove 0000"));
    }

    #[test]
    fn mate_scores_use_uci_mate_notation() {
        assert_eq!(format_uci_score(CHECKMATE_SCORE - 1), "mate 1");
        assert_eq!(format_uci_score(-CHECKMATE_SCORE + 2), "mate -1");
        assert_eq!(format_uci_score(-CHECKMATE_SCORE), "mate 0");
        assert_eq!(format_uci_score(42), "cp 42");
    }
}
