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

Debug UCI always collects and reports detailed statistics. The `debug-uci`
feature enables `search-stats`, and searches use the regular instrumented
iterative-deepening and negamax implementations. Raw counters come directly
from `SearchResult.stats`; there is no `Debug.Statistics` option. Detailed
counters add execution cost, so compare timing using builds with the same features.

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
| `info string twilight.result ` | Final metrics, completion/termination, `detailed_statistics: true` and optional discovery result |

Final iteration snapshots and statistics are emitted after the measured search
and before the final ordinary `info` and `bestmove`. Full histogram arrays are
preserved. No per-node serialization or protocol output is added to timing.

## Search history and stopping

With `debug-uci` enabled, `SearchResult.iterations` contains a compact
`SearchIteration` record for each completed depth: move, evaluation, seldepth,
PV, cumulative main/quiescence nodes and elapsed time. Interrupted depths are
excluded. Final statistics can include work from an interrupted depth, so they
may exceed the last completed iteration's counts. History is absent from builds
without `debug-uci`.

Every caller creates a fresh `SearchContext` with its limits and repetition
history, then calls `Engine::search(&board, &mut context, opening_allowed,
can_print)`. Debug UCI attaches its cancellation flag and optional
`FindSolutionState` to that context before making the same call.
After accepting a completed depth, iterative deepening calls
`SearchContext::record_iteration`. That saves the snapshot and asks the concrete
finder to update its confirmation streak. A confirmed solution returns `true`,
and the loop breaks with `SearchTermination::IterationStop`. Before returning,
the loop moves the history into `SearchResult`. There are no iteration callbacks.
The adapter formats the returned history and the finder's final discovery report.

Normal UCI calls `Engine::search` regardless of build features. It emits the
final ordinary `info` line and `bestmove`, without intermediate depth updates.

There is one implementation each of `search`, `iterative_deepening`,
`search_root`, `negamax`, and `quiescence`, all under `src/engine/search/`.
Only `search` starts a search; it calls iterative deepening, which calls root
search. Root search and recursive negamax use the same negamax implementation,
and negamax enters quiescence at depth zero. There are no separate statistics
search entry points or mirrored search implementations.

The `search-stats` feature controls detailed counter updates, aspiration
statistics, per-depth histograms and detailed console reports. Without it, only
main/quiescence node counts are collected; search limits and elapsed timing
remain available. The existing regular search supplies the root move as its PV
prefix. Unused PV-table code from the removed mirror has been deleted.

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
