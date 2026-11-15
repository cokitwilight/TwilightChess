# Twilight Chess

Twilight Chess is a chess engine written entirely in Rust. 

The goal of this project is to build a solid chess engine with clean architecture. The project remains educational and focuses on learning engine design, move ordering heuristics, and low-level optimizations rather than competing with chess giants like Stockfish or Leela.

## Project Status

The current engine is playable but under active development. Currently has basic UCI support, a player vs player or player vs bot gui, and a tournament self play testing framework. 
Next steps are further improving optimizations, improving/tuning heuristics and hand made eval, continuing to implement pruning techniques, and validating an actual engine elo.

## Estimated Elo: 2316-2486

Twilight is estimated to perform at an elo range of 2,316 to 2,486. The lower bound was determined by a 400 game match against Blunder 7.1.0, rated 2388 CCRL, using blitz time controls(normalized to my machine to 60s + 0.5s increment). The upper bound was determined based on a 2,000-game match against Stockfish 19 configured with UCI_LimitStrength=true and UCI_Elo=2500, at a time control of 10 seconds + 0.1-second increment.

Testing used randomly selected openings from 8move_v3.pgn, with 16 plies of predetermined moves and each opening played once per color to reduce opening and color bias. Playing strength may vary across time controls, hardware, and opponents; additional testing at longer time controls and against different opponents is needed to establish a more representative rating.

## Current Features

- Fully Working Basic GUI
- Player vs Player
- Player vs Bot(2 second search limit)
- Perft-validated Move Generation
- Self Play Bot Tournament
- Static Evaluation Debugger
- Sophisticated Bot Opening Book

## Roadmap

- [ ] Full Elo validation with CuteChess Testing
- [ ] Stylized/Improved GUI

## Requirements

- Rust
- Cargo

## Build

```bash
cargo build --release
```

## Run

```bash
cargo run --release
```

## Static Evaluation Debugger

Note: The static evaluation tool is currently used solely for debugging by printing to the terminal.
Launch the interactive evaluator separately from the terminal:

```bash
cargo run --release --bin eval-debug
```

Paste a six-field FEN at the prompt for a compact summary. Use commands such as
`show pawns`, `show mobility`, `show sliders`, `show king`, or `show all` to print
the named bonuses for the current position. Type `compare` to enter two FENs and
print their overall values side by side. Once loaded, `compare pawns`,
`compare king`, or a named row such as `compare pawn shield` prints its change.
Comparison deltas are Position 2 minus Position 1. Type `help` for the full
command list.

## UCI

Note: Current UCI is a starting implementation and only necessary UCI functions currently work.
Build and run the dedicated UCI executable:

```bash
cargo run --release --bin uci
```

The main executable can also enter UCI mode with `--uci`. The protocol supports
`uci`, `isready`, `ucinewgame`, `position startpos`, `position fen`, move lists,
`setoption` for `Hash`, `Clear Hash`, and `OwnBook`, and `go` limits for depth,
nodes, move time, and the standard clock/increment fields.

After each completed search iteration, UCI reports `depth` and `seldepth` for
GUI displays such as Cute Chess's depth pair. `seldepth` is the deepest ply
visited during that iteration, including quiescence search.

Search is currently synchronous because the engine does not expose external
cancellation. As a result, interruptible `stop`, `ponder`, `infinite`,
`searchmoves`, and mate-limited searches are not implemented yet.

## Testing

Run all tests:

```bash
cargo test --release
```

Tournament matches are run through the dedicated binary. Pass the former test
name as the tournament selector:

```bash
cargo run --release --bin tournament -- test_tournament_lmr
```

Run `cargo run --release --bin tournament -- --help` to list every configured
tournament.

## Engine Architecture

**Search Pruning:**

- Alpha Beta Pruning
- Null Move Pruning
- Reverse Futility Pruning
- Futility Pruning
- Delta Pruning
- SEE pruning
- Late Move Pruning(LMP)

**Planned Pruning Additions**

- [ ] Main SEE pruning
- [ ] ProbCut
- [ ] Add dynamic decisions to pruning(NMP, LMR, SEE, etc)

**Other Search Infrastructure**

- Expanded History Tables with Capture, Main(Butterfly), Killers, and 1-,2-, and 4-ply Continuation History
- Switched To History Gravity rather than Aging
- Clustered Transposition Table
- Iterative Deepening + Aspiration Windows
- Detailed Statistics Tracking
- Singular Extensions 
- Pawn Hash based Correction History

**Planned Search Additions**

- [ ] Advanced Correction History(Minor, major, continuation, etc)
- [ ] Allow larger and/or chained singular extensions
- [ ] Implement NNUE
- [ ] Lazy SMP Search

**Current Optimizations**

- Magic Bitboards
- Pin Masks and Check Masks
- Iterative PST, Phase, and Material
- Reused Eval Info
- Staged Move Generation
- Packed TT Entries into 32 byte clusters

**Planned Optimizations**

- [ ] Magic Pin and Check Masks
- [ ] Heavily Optimize Current Eval Function
- [ ] Implement Pawn Hashing to store eval

