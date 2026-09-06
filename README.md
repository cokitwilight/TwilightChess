# Twilight Chess

Twilight Chess is a chess engine written entirely in Rust. 

The goal of this project is to build a solid chess engine with clean architecture. The project remains educational and focuses on learning engine design, move ordering heuristics, and low level optimizations rather than compete with chess giants like StockFish or Leela.

## Project Status

This project is currently being ported from a 2D array board representation to bitboards.

The move generator, board representation, GUI, Zobrist hashing, and game-end detection are currently working. Search and evaluation features are being ported next.

## Current Features

- Simple Bitboard Representation
- GUI with player vs player support
- Legal Move highlighting
- Promotion option slider
- Checkmate + Stalemate detection
- Perft Validated Move Generation
- Zobrist Hashing

## In Progress

- [ ] Insufficient Material Stalemate
- [x] Static Evaluation
- [x] Negamax Search with Alpha Beta pruning
- [x] Quiescence Search
- [x] Transposition Tables for regualar and quiescence search
- [x] Killer Moves Heuristic
- [x] History Heuristics
- [X] SEE(Static Exchange Evaluation)
- [x] Iterative Deepening
- [X] Aspiration Windows
- [X] Delta Pruning
- [X] Simple Opening Book
- [x] Gui with Player vs Bot

## Planned Features

- [X] Null Move Pruning
- [ ] Magic Bitboards
- [X] Pin Masks
- [x] Basic UCI Support
- [ ] SMP Search

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

Launch the interactive evaluator separately from the GUI:

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

Build and run the dedicated UCI executable:

```bash
cargo run --release --bin uci
```

The main executable can also enter UCI mode with `--uci`. The protocol supports
`uci`, `isready`, `ucinewgame`, `position startpos`, `position fen`, move lists,
`setoption` for `Hash`, `Clear Hash`, and `OwnBook`, and `go` limits for depth,
nodes, move time, and the standard clock/increment fields.

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

## Performance Notes

During the bitboard rewrite, the perft performance significantly improved compared to the previous 2D array representation.
On my machine(AMD 9700x) in release mode, the 2D array version reached roughly 16 million nodes/second while the bitboard representation reach roughly 42 million nodes/second.
These numbers are informal and are likely to change as the engine is optimized further.
