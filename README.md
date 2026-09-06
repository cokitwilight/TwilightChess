# Twilight Chess

Twilight Chess is a chess engine written entirely in Rust. 

The goal of this project is to build a solid chess engine with clean architecture. The project remains educational and focuses on learning engine design, move ordering heuristics, and low-level optimizations rather than competing with chess giants like Stockfish or Leela.

## Project Status

The current engine is playable but under active development. Currently has basic UCI support, a player vs player or player vs bot gui, and a tournament self play testing framework. 
Next steps are further improving optimizations, improving/tuning heuristics and hand made eval, continuing to implement pruning techniques, and validating an actual engine elo.

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

**Planned Pruning Additions**

- [ ] Late Move Pruning(LMP)
- [ ] ProbCut

**Other Search Infrastructure**

- Expanded History Tables with Capture, Main(Butterfly), Killers, and 1-,2-, and 4-ply Continuation History
- Switched To History Gravity rather than Aging
- Clustered Transposition Table
- Iterative Deepening + Aspiration Windows
- Detailed Statistics Tracking
- Singular Extensions 

**Planned Search Additions**

- [ ] Correction History
- [ ] Lazy SMP Search

**Current Optimizations**

- Magic Bitboards
- Pin Masks and Check Masks
- Iterative PST, Phase, and Material
- Reused Eval Info
- Staged Move Generation

**Planned Optimizations**

- [ ] Magic Pin and Check Masks
- [ ] Heavily Optimize Current Eval Function
- [ ] Implement Pawn Hashing
- [ ] Optimize CPU Cache Alignment and overall Memory Usage
- [ ] Pack Move into 16 bits


## Perft Performance Notes

NOTE: The following observations are merely observations on my machine(AMD Ryzen 7 9700x) and do not reflect actual tested results.
These numbers come from a rough 3 test average rather than full testing.

Many optimizations have caused perft NPS to grow steadily.
Before the bitboard rewrite, perft performance stood at only 14 million NPS.
After the basic bitboard rewrite perft performance jumped to 42 million NPS.
Eventually when Pin and Check masks were implemented perft performance grew to roughly 100 million NPS. 
When Magic Bitboards replaced my old ray scanning logic perft gained another 40 million NPS to reach approximately 142 million NPS.
The largest jump in perft performance was when I switched my old naive en passant and king move handling. 
Originally move generation would treat king and pawns moves separately by making the move before undoing it.
The goal was to handle en passant but I naively included all pawn moves as well further increasing to the cost of perft.
However after adding a special is_legal_move tuned for en passant and king moves using ray scanning instead, the resulting perft performance
jumped by more than 2x from 142 million NPS to ~305 million NPS.

## Self Play Tournament Notes

The following observations came from multiple tournaments where one engine would have the standard config(SEE, Delta, FUT, RFP, NMP, SING, LMR disabled by default)
and the other would contain the tested feature enabled(or similarly have the same features enabled but a different value or margin). Over roughly 800-1200 games per tournament(some features had multiple tournaments rather than 1). Each test limited time per move to 250 ms to average ~30 second games. 

**RFP ON vs RFP OFF: 880 Total Games:**
RFP On:  560.5/880 (63.69%) | 495 W - 254 L - 131 D
RFP Off: 319.5/880 (36.31%) | 254 W - 495 L - 131 D 

Estimated +97.6 Elo Difference
95% Elo Confidence: (+76.1, +119.9) 


**Notes**
RFP Demonstrated the strongest superiority over the standard engine. Due to this superiority another tournament should be done to confirm the elo difference.


**Futility On vs Futility Off: 880 Total Games:**
FUT On:  455.5 (51.76%) | 380 W - 349 L - 151 D 
FUT Off: 424.5 (48.14%) | 349 W - 380 L - 151 D

Estimated +12.2 Elo Difference
95% Elo Confidence: (-8.7, +33.2)

**Notes**
Due to the fact that the confidence interval falls in both negative and positive elo estimates another tournament should be conducted. Additionally during the tournament there was actually a small bug in the history table. Although this doesn't directly interact with Futility pruning it would still affect engine performance and therefore the tournament result should not be fully trusted.

## Search Performance Notes

NOTE: The following observations are merely observations on my machine(AMD Ryzen 7 9700x) and do not reflect actual tested results.
These numbers come from playing with the engine and pairing the engine through hand made matches(I played the moves myself) against Chess.com bots.
These do not reflect thoroughly tested results over 1000s of games since the test environment changed drastically and less overall games were played.
Additionally due to the varying nature of chess positions, observed NPS varied greatly. Specifically tactical positions could potentially lose NPS(by hundreds of thousands at some times) while
endgame and checkmate scenarios yielded huge jumps in NPS(sometimes millions of NPS higher). Additionally true NPS is likely higher as search statistics and other debug info can likely affect the performance.

Switching to bitboards yielded a rough 2x increase in NPS as the 2D Array board had roughly 500,000 NPS while the bitboard version has 1.1 million NPS
Since there was no previous PST/phase/material the iterative version does not have a NPS comparison.
Multiple Changes occured during the introduction of Pin and Check masks and NPS roughly stayed the same. This is likely due to changing the eval to be most sophisticated which took up more resources.
After Pin and Check masks, Magic Bitboards contributed a rough 200,000 NPS increase to roughly 1.3 million NPS.
Staged Move Ordering resulted in roughly 150,000 NPS increase before being rewritten to Staged Move Generation.
Full Staged Move Generation yielded roughly 300,000 NPS to land roughly at 1.6-1.7 million NPS.


