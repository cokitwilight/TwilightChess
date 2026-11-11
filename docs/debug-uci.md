# Twilight debugging UCI

Build the optional, optimized debugging executable:

```powershell
cargo build --release --features debug-uci --bin twilight-debug
```

Run `target\release\twilight-debug.exe` directly. It speaks UCI over standard
input/output; it does not require `--uci`. Normal builds and the existing UCI
entry point keep their existing option list. This is a release build with
debugging capabilities, not Cargo's unoptimized debug profile.

## Configuration

The `uci` handshake advertises the actual serialized `EngineConfig::default()`
as dotted option names. Boolean fields use `check`, integer fields use `spin`,
and decimal/optional integer fields use `string`. `<none>` clears optional
budgets. `Hash` is an alias for `tt_size`. New scalar fields are discovered from
serialization; add semantic validation when a new field has special invariants.

```text
uci
setoption name tt_size value 128
setoption name search.lmr.divisor value 2.25
setoption name search.null_move.enabled value false
setoption name limits.max_q_depth value 6
setoption name Debug.Statistics value true
isready
position startpos
go depth 8
```

Changes are accepted while idle, validated as a complete configuration on
`isready`/`go`, and applied by creating a fresh `Engine::new(config)` for each
`go`. TT, history, LMR tables and all construction-time settings are therefore
isolated per EPD position. Opening books are disabled. All options for a run
should be sent before starting its first search. `go` explicitly controls the
node/time budgets; constructor time limits are not silently inherited.

`Debug.Statistics=true` uses the same instrumented iterative-deepening and
negamax search as `Engine::search`. False uses the lighter benchmark path. Raw
counters come directly from `SearchResult.stats`. Detailed counters change
execution cost, so compare timing with the same statistics setting.

The input loop remains responsive during search. `stop`/`quit` set an atomic
cancellation token; it is checked by the search's existing stop checks, rather
than waiting for an entire iterative depth. `isready` can be answered during
search. Search-affecting commands are rejected until `bestmove`.

## Version 1 diagnostic messages

All extensions are newline-delimited compact JSON inside standard `info string`
messages. A client must buffer partial reads and continuously drain stdout.
Never assume that one pipe read contains exactly one line. Flushes occur after
messages; ordinary UCI engines need not understand these diagnostic payloads.

| Prefix | Payload |
|---|---|
| `info string twilight.schema ` | Version, capabilities, defaults and typed field list; emitted before `uciok` |
| `info string twilight.config ` | Accepted configuration; emitted before `readyok` |
| `info string twilight.error ` | JSON string describing a rejected request |
| `info string twilight.iteration ` | Completed-iteration metrics; buffered during search and emitted afterwards |
| `info string twilight.stats ` | `{ "version":1, "group":"lmr_stats", "data":{...} }`; one message per statistics group |
| `info string twilight.result ` | Final metrics, completion/termination, statistics setting and optional discovery result |

Final iteration snapshots and statistics are emitted after the measured search
and before the final ordinary `info` and `bestmove`. Full histogram arrays are
preserved. No per-node serialization or protocol output is added to timing.

## Exact solution discovery

Set the string option `Debug.FindSolution` to a compact JSON object containing
`options` and `position`. The GUI loads EPD files and normalizes annotated moves
to legal UCI strings before sending these conditions:

```json
{"options":{"start_depth":1,"max_depth":12,"confirm_depths":2},"position":{"id":"sample","fen":"<six-field FEN>","acceptable_moves":["e2e4"]}}
```

Send this after `ucinewgame`, then send the matching `position fen ...` and
`go depth 12`. The engine checks the supplied conditions between completed
iterations. Passing depths below `start_depth` do not count; a miss resets the
confirmation streak. Weighted move scores require full credit, and confirmation
is capped at `max_depth`. Node/time metrics are cumulative from search start.
Unscored records are skipped. `<none>` clears this setting; `ucinewgame` also
clears position-specific annotations.

The GUI treats exhaustion of a dedicated node/time budget as normal completion
when a valid completed iteration exists. Depth mode still requires the requested
depth; manual cancellation is incomplete. Standard UCI alone does not describe
these completion/discovery details, so exact discovery requires this extension.

Validation: `cargo test --features debug-uci --test debug_uci` checks normal
search/statistics equivalence, fresh states, configuration application,
cancellation and discovery. Normal builds do not include the debug adapter or
its serialization dependencies. EPD loading, suite management and reports live
in the separate GUI application; local `suites/` files are ignored by Git.
