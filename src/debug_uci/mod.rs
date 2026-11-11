//! Optional debugging UCI target. Search algorithms remain in `engine`; this
//! adapter owns configuration, cancellation and versioned diagnostic messages.
mod conditions;
pub mod config;
mod discovery;
mod metrics;

use crate::board::{Board, STARTPOS_FEN};
use crate::engine::{Engine, SearchResult, SearchTermination, configs::EngineConfig};
use crate::uci::protocol::{GoParameters, UciPosition, write_search_info};
use conditions::ExpectedAnswer;
use discovery::{DiscoveryTracker, FindSolutionOptions};
use metrics::SearchMetrics;
use serde::Deserialize;
use serde_json::{Value, json};
use std::io::{self, BufRead, BufWriter, Write};
use std::ops::ControlFlow;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
    mpsc,
};
use std::thread::{self, JoinHandle};

pub const PROTOCOL_VERSION: u32 = 1;
type Output = Arc<Mutex<BufWriter<io::Stdout>>>;

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct FindRequest {
    options: FindSolutionOptions,
    position: ExpectedAnswer,
}

struct Input {
    line: String,
    cancellation: Option<Arc<AtomicBool>>,
}

fn emit(output: &Output, text: &str) -> io::Result<()> {
    let mut out = output
        .lock()
        .map_err(|_| io::Error::other("output lock poisoned"))?;
    writeln!(out, "{text}")?;
    out.flush()
}

fn diagnostic(output: &Output, name: &str, data: &impl serde::Serialize) -> io::Result<()> {
    emit(
        output,
        &format!(
            "info string twilight.{name} {}",
            serde_json::to_string(data)?
        ),
    )
}

pub fn run_stdio() -> io::Result<()> {
    let (sender, receiver) = mpsc::channel();
    // Read commands while search is running. Each go receives its own token,
    // so a stop sent immediately after go cannot be lost to a flag reset.
    thread::spawn(move || {
        let stdin = io::stdin();
        let mut active: Option<Arc<AtomicBool>> = None;
        for line in stdin.lock().lines() {
            let Ok(line) = line else { break };
            let command = line
                .split_whitespace()
                .next()
                .unwrap_or("")
                .to_ascii_lowercase();
            let cancellation = if command == "go" {
                let flag = Arc::new(AtomicBool::new(false));
                active = Some(flag.clone());
                Some(flag)
            } else {
                None
            };
            if matches!(command.as_str(), "stop" | "quit") {
                if let Some(flag) = &active {
                    flag.store(true, Ordering::Relaxed);
                }
            }
            if sender.send(Input { line, cancellation }).is_err() {
                return;
            }
            if command == "quit" {
                return;
            }
        }
        if let Some(flag) = active {
            flag.store(true, Ordering::Relaxed);
        }
        let _ = sender.send(Input {
            line: "quit".into(),
            cancellation: None,
        });
    });

    let output = Arc::new(Mutex::new(BufWriter::new(io::stdout())));
    let mut config = EngineConfig::default();
    let mut position = UciPosition::parse("startpos").map_err(io::Error::other)?;
    let mut detailed_stats = false;
    let mut find: Option<FindRequest> = None;
    let mut worker: Option<JoinHandle<()>> = None;
    let mut active_cancel: Option<Arc<AtomicBool>> = None;
    let mut finishing = Arc::new(AtomicBool::new(false));
    while let Ok(input) = receiver.recv() {
        if worker.as_ref().is_some_and(|w| w.is_finished())
            || worker.is_some() && finishing.load(Ordering::Acquire)
        {
            if worker.take().unwrap().join().is_err() {
                diagnostic(&output, "error", &"search worker panicked")?;
            }
        }
        let line = input.line.trim();
        let (command, args) = line.split_once(char::is_whitespace).unwrap_or((line, ""));
        let args = args.trim();
        let command = command.to_ascii_lowercase();
        if command == "quit" {
            if let Some(flag) = active_cancel.take() {
                flag.store(true, Ordering::Relaxed);
            }
            if let Some(worker) = worker.take() {
                let _ = worker.join();
            }
            break;
        }
        if command == "stop" {
            continue;
        }
        if command == "isready" {
            if let Err(error) = config::validate(&config) {
                diagnostic(&output, "error", &error)?;
            } else {
                diagnostic(&output, "config", &config)?;
            }
            emit(&output, "readyok")?;
            continue;
        }
        if command == "uci" {
            emit(
                &output,
                &format!("id name Twilight Debug {}", env!("CARGO_PKG_VERSION")),
            )?;
            emit(&output, "id author Twilight Chess contributors")?;
            for field in config::fields() {
                emit(&output, &config::option_line(&field))?;
            }
            emit(
                &output,
                "option name Hash type spin default 128 min 1 max 4096",
            )?;
            emit(&output, "option name Clear Hash type button")?;
            emit(
                &output,
                "option name Debug.Statistics type check default false",
            )?;
            emit(
                &output,
                "option name Debug.FindSolution type string default <none>",
            )?;
            diagnostic(
                &output,
                "schema",
                &json!({"version":PROTOCOL_VERSION,"engine":"Twilight","fields":config::fields(),"defaults":EngineConfig::default(),"capabilities":["statistics","find_solution","fresh_engine_per_position","cancellation"]}),
            )?;
            emit(&output, "uciok")?;
            continue;
        }
        if command == "debug" || command.is_empty() {
            continue;
        }
        if worker.is_some() {
            diagnostic(
                &output,
                "error",
                &"search is running; stop and wait for bestmove before changing settings",
            )?;
            continue;
        }
        let outcome: Result<(), String> = (|| {
            match command.as_str() {
                "setoption" => {
                    let option = args
                        .strip_prefix("name ")
                        .ok_or("expected setoption name <name> value <value>")?;
                    let (name, value) = option.split_once(" value ").unwrap_or((option, ""));
                    match name.trim().to_ascii_lowercase().as_str() {
                        "debug.statistics" => {
                            detailed_stats = match value.trim().to_ascii_lowercase().as_str() {
                                "true" => true,
                                "false" => false,
                                _ => return Err("Debug.Statistics expects true or false".into()),
                            }
                        }
                        "debug.findsolution" => {
                            find = if matches!(value.trim(), "<none>" | "none" | "<empty>" | "") {
                                None
                            } else {
                                let request: FindRequest =
                                    serde_json::from_str(value).map_err(|e| e.to_string())?;
                                request.options.validate()?;
                                request.position.validate()?;
                                Some(request)
                            };
                        }
                        "clear hash" => {} // Every go already creates a fresh engine.
                        "ownbook" if value.trim().eq_ignore_ascii_case("false") => {}
                        _ => config::set_option(&mut config, name.trim(), value.trim())?,
                    }
                }
                "ucinewgame" => {
                    position = UciPosition::parse("startpos")?;
                    find = None;
                }
                "position" => position = UciPosition::parse(args)?,
                "go" => {
                    config::validate(&config)?;
                    let mut limits = GoParameters::parse(args)?
                        .to_limits(position.board.side_to_move(), config.limits)?;
                    if let Some(request) = &find {
                        let annotated = Board::from_fen(&request.position.fen)?;
                        if annotated.board_hash() != position.board.board_hash()
                            || annotated.halfmove_clock() != position.board.halfmove_clock()
                        {
                            return Err("discovery annotations do not match the current FEN".into());
                        }
                        limits.max_depth = request.options.max_depth;
                    }
                    let flag = input
                        .cancellation
                        .unwrap_or_else(|| Arc::new(AtomicBool::new(false)));
                    active_cancel = Some(flag.clone());
                    let board = position.board.clone();
                    let history = position.repetition_history.clone();
                    let output = output.clone();
                    let find = find.clone();
                    finishing = Arc::new(AtomicBool::new(false));
                    let ready = finishing.clone();
                    worker = Some(
                        thread::Builder::new()
                            .name("twilight-search".into())
                            .stack_size(16 * 1024 * 1024)
                            .spawn(move || {
                                let result =
                                    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                                        search(
                                            &output,
                                            config,
                                            limits,
                                            &board,
                                            &history,
                                            detailed_stats,
                                            flag,
                                            find,
                                            &ready,
                                        )
                                    }));
                                match result {
                                    Ok(Ok(())) => {}
                                    Ok(Err(error)) => {
                                        ready.store(true, Ordering::Release);
                                        let _ = diagnostic(&output, "error", &error.to_string());
                                        let _ = emit(&output, "bestmove 0000");
                                    }
                                    Err(_) => {
                                        ready.store(true, Ordering::Release);
                                        let _ =
                                            diagnostic(&output, "error", &"search worker panicked");
                                        let _ = emit(&output, "bestmove 0000");
                                    }
                                }
                            })
                            .map_err(|e| e.to_string())?,
                    );
                }
                _ => return Err(format!("unsupported command `{command}`")),
            }
            Ok(())
        })();
        if let Err(error) = outcome {
            diagnostic(&output, "error", &error)?;
        }
    }
    Ok(())
}

fn search(
    output: &Output,
    mut config: EngineConfig,
    limits: crate::engine::SearchLimits,
    board: &Board,
    history: &Vec<u64>,
    detailed_stats: bool,
    flag: Arc<AtomicBool>,
    find: Option<FindRequest>,
    finishing: &AtomicBool,
) -> io::Result<()> {
    let scorable = find.as_ref().is_none_or(|r| r.position.scorable());
    if !scorable {
        let request = find.unwrap();
        let discovery = DiscoveryTracker::new(request.options).finish(0, false, false);
        let mut report = serde_json::to_value(SearchMetrics::unsearched())?;
        report["completed"] = json!(false);
        report["termination"] = json!("unscored");
        report["discovery"] = serde_json::to_value(discovery)?;
        diagnostic(output, "result", &report)?;
        finishing.store(true, Ordering::Release);
        return emit(output, "bestmove 0000");
    }
    // Exclude shared table initialization and engine construction from timing.
    crate::bitboard::attack_tables();
    crate::bitboard::magic::bishop_attack_table();
    crate::bitboard::magic::rook_attack_table();
    let mut warmup = Board::from_fen(STARTPOS_FEN).map_err(io::Error::other)?;
    warmup.all_legal_moves();
    crate::eval::evaluation_for_turn(&warmup);
    config.limits = limits;
    let mut engine = Engine::new(config);
    let mut tracker = find.as_ref().map(|r| DiscoveryTracker::new(r.options));
    let mut iterations = Vec::new();
    let result: SearchResult = engine.search_with_debug_control(
        board,
        limits,
        history,
        detailed_stats,
        flag,
        &mut |iteration| {
            let metrics = SearchMetrics::from_result(iteration);
            let stop = tracker.as_mut().is_some_and(|tracker| {
                let request = find.as_ref().unwrap();
                tracker.observe(
                    &metrics,
                    request
                        .position
                        .is_passing(metrics.best_move.as_deref(), metrics.completed_depth),
                )
            });
            iterations.push(metrics);
            if stop {
                ControlFlow::Break(())
            } else {
                ControlFlow::Continue(())
            }
        },
    );
    let cap = result.depth_reached == limits.max_depth
        && result.termination == SearchTermination::DepthLimit;
    let solved = result.termination == SearchTermination::IterationStop;
    let completed = if find.is_some() {
        cap || solved
    } else {
        cap || result.depth_reached > 0
            && matches!(
                result.termination,
                SearchTermination::NodeLimit | SearchTermination::TimeLimit
            )
    };
    let termination = match result.termination {
        SearchTermination::DepthLimit => "depth_limit",
        SearchTermination::NodeLimit => "node_limit",
        SearchTermination::TimeLimit => "time_limit",
        SearchTermination::Cancelled => "cancelled",
        SearchTermination::IterationStop => "solution_found",
        SearchTermination::BookMove => "book_move",
    };
    for metrics in iterations {
        diagnostic(output, "iteration", &metrics)?;
    }
    if detailed_stats {
        if let Value::Object(groups) = serde_json::to_value(result.stats)? {
            for (group, data) in groups {
                diagnostic(
                    output,
                    "stats",
                    &json!({"version": PROTOCOL_VERSION,"group":group,"data":data}),
                )?;
            }
        }
    }
    let mut report = serde_json::to_value(SearchMetrics::from_result(&result))?;
    report["version"] = json!(PROTOCOL_VERSION);
    report["completed"] = json!(completed);
    report["termination"] = json!(termination);
    report["detailed_statistics"] = json!(detailed_stats);
    if let Some(tracker) = tracker {
        report["discovery"] =
            serde_json::to_value(tracker.finish(result.depth_reached, cap, true))?;
    }
    diagnostic(output, "result", &report)?;
    {
        let mut out = output
            .lock()
            .map_err(|_| io::Error::other("output lock poisoned"))?;
        write_search_info(&mut *out, &result)?;
    }
    finishing.store(true, Ordering::Release);
    emit(
        output,
        &format!(
            "bestmove {}",
            result
                .best_move
                .map(|m| m.to_uci())
                .unwrap_or_else(|| "0000".into())
        ),
    )
}
