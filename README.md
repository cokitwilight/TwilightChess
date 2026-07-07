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
- [ ] SEE(Static Exchange Evaluation)
- [x] Iterative Deepening
- [ ] Aspiration Windows
- [ ] Delta Pruning
- [ ] Simple Opening Book
- [x] Gui with Player vs Bot

## Planned Features

- [ ] Null Move Pruning
- [ ] Magic Bitboards
- [ ] Pin Masks
- [ ] UCI Support
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

## Testing

Run all tests:

```bash
cargo test --release
```

## Performance Notes

During the bitboard rewrite, the perft performance significantly improved compared to the previous 2D array representation.
On my machine(AMD 9700x) in release mode, the 2D array version reached roughly 16 million nodes/second while the bitboard representation reach roughly 42 million nodes/second.
These numbers are informal and are likely to change as the engine is optimized further.
