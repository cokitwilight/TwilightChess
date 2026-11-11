#![cfg(feature = "debug-uci")]
use chess_final::board::{Board, STARTPOS_FEN};
use chess_final::engine::{Engine, SearchLimits, configs::EngineConfig};
use serde_json::{Value, json};
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::mpsc::{self, Receiver};
use std::time::Duration;

struct Client {
    child: Child,
    input: ChildStdin,
    lines: Receiver<String>,
}
impl Drop for Client {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
impl Client {
    fn new() -> Self {
        let mut child = Command::new(env!("CARGO_BIN_EXE_twilight-debug"))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .unwrap();
        let input = child.stdin.take().unwrap();
        let output = child.stdout.take().unwrap();
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            for line in BufReader::new(output).lines() {
                let Ok(line) = line else { break };
                if tx.send(line).is_err() {
                    break;
                }
            }
        });
        Self {
            child,
            input,
            lines: rx,
        }
    }
    fn send(&mut self, line: &str) {
        writeln!(self.input, "{line}").unwrap();
        self.input.flush().unwrap();
    }
    fn until(&self, token: &str) -> Vec<String> {
        let mut lines = Vec::new();
        loop {
            let line = self
                .lines
                .recv_timeout(Duration::from_secs(30))
                .expect("engine reply timeout");
            let done = line.starts_with(token);
            lines.push(line);
            if done {
                return lines;
            }
        }
    }
    fn ready(&mut self) {
        self.send("isready");
        let lines = self.until("readyok");
        assert!(
            !lines
                .iter()
                .any(|l| l.starts_with("info string twilight.error ")),
            "{lines:?}"
        );
    }
    fn setup(&mut self, stats: bool) {
        self.send("uci");
        self.until("uciok");
        self.send("setoption name Hash value 1");
        self.send("setoption name limits.max_q_depth value 3");
        self.send(&format!("setoption name Debug.Statistics value {stats}"));
        self.ready();
    }
    fn search(&mut self, go: &str) -> Vec<String> {
        self.send(&format!("position fen {STARTPOS_FEN}"));
        self.send(go);
        self.until("bestmove ")
    }
}
fn report(lines: &[String]) -> Value {
    lines
        .iter()
        .find_map(|l| l.strip_prefix("info string twilight.result "))
        .map(|s| serde_json::from_str(s).unwrap())
        .expect("structured result")
}

#[test]
fn advertises_actual_defaults_and_runs_the_normal_instrumented_search() {
    let mut client = Client::new();
    client.send("uci");
    let lines = client.until("uciok");
    assert!(
        lines
            .iter()
            .any(|l| l.starts_with("option name search.lmr.divisor type string default 2.0"))
    );
    let schema: Value = serde_json::from_str(
        lines
            .iter()
            .find_map(|l| l.strip_prefix("info string twilight.schema "))
            .unwrap(),
    )
    .unwrap();
    assert_eq!(schema["defaults"]["search"]["lmr"]["history_scale"], 94);
    client.send("setoption name Hash value 1");
    client.send("setoption name limits.max_q_depth value 3");
    client.send("setoption name Debug.Statistics value true");
    client.ready();
    let lines = client.search("go depth 2");
    let result = report(&lines);
    assert_eq!(result["completed"], true);
    assert_eq!(result["completed_depth"], 2);
    let board = Board::from_fen(STARTPOS_FEN).unwrap();
    let cfg = EngineConfig {
        tt_size: 1,
        limits: SearchLimits::depth(2, 3),
        ..EngineConfig::default()
    };
    let normal =
        Engine::new(cfg).search(&board, cfg.limits, &vec![board.board_hash()], false, false);
    assert_eq!(result["nodes"], normal.stats.total_nodes());
    assert_eq!(result["raw_eval"], normal.eval);
    assert_eq!(result["best_move"], normal.best_move.unwrap().to_uci());
    let mut stats = json!({});
    for line in &lines {
        if let Some(s) = line.strip_prefix("info string twilight.stats ") {
            let v: Value = serde_json::from_str(s).unwrap();
            stats[v["group"].as_str().unwrap()] = v["data"].clone();
        }
    }
    assert_eq!(stats, serde_json::to_value(normal.stats).unwrap());
}

#[test]
fn repeated_positions_are_fresh_and_config_changes_apply_between_searches() {
    let mut client = Client::new();
    client.setup(false);
    let first = report(&client.search("go depth 2"));
    client.send("ucinewgame");
    client.ready();
    let second = report(&client.search("go depth 2"));
    assert_eq!(first["nodes"], second["nodes"]);
    assert_eq!(first["best_move"], second["best_move"]);
    client.send("setoption name search.lmr.divisor value 3.25");
    client.send("isready");
    let lines = client.until("readyok");
    let c: Value = serde_json::from_str(
        lines
            .iter()
            .find_map(|l| l.strip_prefix("info string twilight.config "))
            .unwrap(),
    )
    .unwrap();
    assert_eq!(c["search"]["lmr"]["divisor"], 3.25);
    client.send("setoption name search.lmr.history_scale value 0");
    client.send("isready");
    assert!(
        client
            .until("readyok")
            .iter()
            .any(|l| l.starts_with("info string twilight.error "))
    );
}

#[test]
fn cancellation_is_prompt_and_budget_modes_have_valid_completion() {
    let mut client = Client::new();
    client.setup(false);
    let nodes = report(&client.search("go nodes 10000"));
    assert_eq!(nodes["termination"], "node_limit");
    assert_eq!(nodes["completed"], true);
    let zero = report(&client.search("go movetime 0"));
    assert_eq!(zero["completed"], false);
    assert_eq!(zero["completed_depth"], 0);
    client.send(&format!("position fen {STARTPOS_FEN}"));
    client.send("go depth 255");
    client.send("isready");
    client.until("readyok");
    let start = std::time::Instant::now();
    client.send("stop");
    let stopped = report(&client.until("bestmove "));
    assert!(start.elapsed() < Duration::from_secs(2));
    assert_eq!(stopped["termination"], "cancelled");
    assert_eq!(stopped["completed"], false);
}

#[test]
fn discovery_confirms_a_solution_inside_the_engine() {
    let mut client = Client::new();
    client.setup(false);
    let first = report(&client.search("go depth 1"));
    let best = first["best_move"].as_str().unwrap();
    let request = json!({"options":{"start_depth":1,"max_depth":1,"confirm_depths":2},"position":{"id":"test","fen":STARTPOS_FEN,"acceptable_moves":[best]}});
    client.send("ucinewgame");
    client.ready();
    client.send(&format!(
        "setoption name Debug.FindSolution value {request}"
    ));
    let result = report(&client.search("go depth 1"));
    assert_eq!(result["termination"], "solution_found");
    assert_eq!(result["discovery"]["status"], "solved");
    assert_eq!(result["discovery"]["confirmation_capped"], true);
}
