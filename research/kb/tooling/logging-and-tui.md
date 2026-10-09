---
type: Design
title: Logging and the terminal UI
description: "tracing events under vibegraph::* targets, the progress contract, plain/TUI/file layers, inline-viewport footer, stdout for results only, and graceful stop on q/Ctrl-C."
status: draft
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-09}]
tags: [cli, logging, tui, tracing, progress]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: n33-decisions, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/33-logging-tui-plan.md#L43-L69", title: "Note 33 §1, decisions (user, 2026-08-05)"}
  - {id: n33-arch, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/33-logging-tui-plan.md#L72-L197", title: "Note 33 §2–3, architecture and dependencies"}
  - {id: n33-nongoals, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/33-logging-tui-plan.md#L272-L292", title: "Note 33 §6–7, non-goals and open decisions"}
  - {id: n33-closeout, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/33-logging-tui-plan.md#L302-L383", title: "Note 33 §9, close-out: plan vs code, decisions, abort, verified vs asserted"}
  - {id: code-logging, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/vibegraph-cli/src/logging.rs", title: "vibegraph-cli/src/logging.rs"}
  - {id: code-progress, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/vibegraph-lib/src/progress.rs", title: "vibegraph-lib/src/progress.rs"}
---
# Logging and the terminal UI

## The output contract

**stdout carries results only**: the final σ line, `wrote <path>`, and the
`check-events` report. Everything a run says about itself is a `tracing`
event and goes to stderr (plain mode) or into the terminal scrollback through
the status pane (TUI mode). A pipeline reading stdout sees the same bytes at
every verbosity and in both modes; this was verified by raw-byte hashes of
stdout across `-q` … `-vv` and TUI vs piped.[^n33-closeout]

The contract is exhaustive. When it conflicted with an existing output line
(`generate`'s `scan:` summary, which a default-feature test read off stdout),
the line moved to stderr as an `info!` event and the test followed. The
extended-validation CLI test that parses stdout is `cli_generate_proton`.
This is the decision a user might revisit; the reason to keep it is that
machine consumers (`… | jq`, the CLI tests) must observe no change when
logging changes. Whether the stdout σ line adopts the SI formatter is left
open; it stays frozen for now.

## Emission (library side)

`vibegraph-lib` depends on `tracing` only (macros and spans, no subscriber);
all subscriber choice lives in the CLI. `tracing` rather than `log` because
structured fields drive the status pane without the library knowing a UI
exists, and `EnvFilter` gives per-module control.[^n33-decisions]

- **Targets** are the default module paths. The library crate's library name
  is `vibegraph`, so targets are `vibegraph::diagrams`, `vibegraph::helas::…`,
  not `vibegraph_lib::…`.
- **Progress contract** (`vibegraph-lib/src/progress.rs`): machine-consumed
  events on `progress::TARGET = "vibegraph::progress"`, at `TRACE`, each
  carrying `stage`, `done`, `total: Option<u64>`, plus stage-specific fields:
  VEGAS iterations `sigma`, `err`, `chi2` (`progress::vegas_iteration`);
  unweighting `accepted`, `requested` (`progress::unweighting`); evaluation
  timing `ns_per_eval` (`progress::eval_rate`). There is no `trials` field on
  unweighting, because `EventSource` does not expose its trial count. No API
  stability is promised for this contract on the 0.x line.
- **Spans**: the compile loop's span is `compile` (`helas/eval/compile.rs`,
  `info_span!("compile", process = …)`): it iterates subprocesses and flavour
  groups, and "channel" in this codebase means a per-diagram phase-space map.
  Span-close timings are **not** enabled (`FmtSpan::CLOSE` would change the
  shape of every plain-mode run); each heavy stage emits its own elapsed time
  at `debug` instead.
- **Where loops live**: the production VEGAS loop is
  `budget.rs::integrate_channels` (the hadronic and proton `adapt_grids_budget`
  paths delegate to it; `vegas.rs`'s `adapt_*` family is test-only).
  Requested-count unweighting loops are in `lhef/emit.rs`; `unweight.rs` carries
  the weight-scan instrumentation.
- **Determinism**: instrumentation must not touch RNG state, sampling order or
  artifact bytes. Artifacts are byte-identical across `-q`/`-vv`, TUI/piped,
  and with or without the abort plumbing; the `-j` byte-identical-artifact
  assertion and fixed-seed tests are the standing regression gates. Rayon
  workers emit freely; only the CLI's main thread touches the terminal.

Levels: `error`/`warn` are things gone wrong, `info` is MadGraph-style running
commentary, `debug` per-stage internals, `trace` per-item detail.

## Sinks (CLI side, `vibegraph-cli/src/logging.rs`)

**Mode**: the TUI is drawn iff stdout **and** stderr are terminals
(`LogArgs::wants_tui`); `--tui` forces it, `--no-tui` forbids it. A redirected
stdout means something downstream is reading the result, so nobody is
watching.

**Level**: default `info` in both modes (`DEFAULT_LEVEL`). `-v` is `info`
too (the default already shows run notices), `-vv` is `debug`, `-vvv` and
beyond `trace`; `-q` is `warn`; `--log-level off|error|warn|info|debug|trace`
sets it outright and conflicts with `-v`/`-q`. `RUST_LOG`, when set, replaces
whatever the flags ask for and can scope per module (`LogArgs::level`).

Three `tracing-subscriber` layers on one registry, each with its own filter:

1. **Line layer**: human-readable lines; compact form at `info`, target and
   uptime shown from `debug`. Plain mode writes stderr; TUI mode formats to a
   `String` and sends it to the display thread, which inserts it into the
   scrollback. Its filter sits in a `reload::Layer` so the level and scope can
   change at runtime. `vibegraph::progress=off` is part of this filter **at
   every level**, so even `-vvv` never prints progress events as lines.
2. **Progress layer** (TUI only): filtered on `progress::TARGET` alone,
   unaffected by the visible level; folds fields into the display's state.
3. **File layer** (`--log-file <path>`): every event at `trace`, whatever the
   screen shows, progress events included. On-screen filtering is
   prospective only (scrollback is immutable once written), and this file is
   the mitigation: it makes a run reviewable at a level nobody watched it at.

## The TUI

ratatui with an **inline viewport** (`Viewport::Inline(footer::HEIGHT)`,
`HEIGHT = 6` rows) rather than the alternate screen: log lines go into the
terminal's real scrollback with `insert_before`, so native scroll, search and
copy work and history survives exit. ratatui is built with
`scrolling-regions` so `insert_before` scrolls without redrawing the footer.
There is no log widget; the terminal is the log pane. crossterm is consumed
as ratatui's re-export to avoid a dual-version event-type mismatch.
`unicode-width` measures display width for line breaking, because
`insert_before` truncates if the height is undercounted.

One thread owns the terminal: it polls crossterm events, drains the line
channel, applies keys and redraws the footer from shared state. Footer widgets
are pure functions of that state, unit-tested against ratatui's
`TestBackend`.

- **Footer content**: logo, UFO brief (counts and model digest), process brief
  (process, coupling orders, channel count), stage line and progress gauge on
  the left; σ ± err and per-eval timing on the right. In `generate` the σ cell
  becomes accepted/requested and the scan's **predicted** unweighting
  efficiency (σ / Σⱼ w_maxⱼ), pushed in by the CLI because the progress stream
  has no trial count; it read 30.6% predicted against 30.03% achieved on the
  verification run.
- **Keys**: ↑/↓ walk a six-rung level ladder (off…trace) through the reload
  handle, inserting a marker line (`── log level → DEBUG ──`) so the history
  explains itself. ←/→ cycle module scope for the `debug`/`trace` tiers only,
  so narrowing can never swallow a warning.
- **SI formatter** (`vibegraph-cli/src/si.rs`): picks the prefix from the
  value, renders value and uncertainty in the same prefix at fixed width so
  live updates do not jitter.
- **Teardown**: on success, error or interrupt the footer is cleared and a
  plain summary line is left, so scrollback ends with the result. A panic hook
  and, on Unix, a `signal-hook` handler restore the terminal before anything
  else prints.
- **Consent prompts**: when the pane is up, a download question goes through
  the pane (`tui::ask_to_download`), not raw stdin; see
  [network consent](network-consent.md).

## Graceful stop

Raw mode delivers Ctrl-C as a key. The first `q` or Ctrl-C is a graceful
stop: finish the current VEGAS iteration, keep what has converged, print the
summary. The second exits immediately. The `StopSignal` is an explicit
parameter of `integrate_channels`, not a global, and its inertness on runs
that are not stopped is pinned by byte-identical artifacts.

A stop during warm-up has no kept iteration, so the combined σ would be NaN
and the grids would be exactly those the warm-up discard exists to throw away.
Such a run is refused: exit 1, no artifact, no σ line
(`ConvergenceReport::kept_iterations`, pinned by `budget.rs` tests and a PTY
run).

**`generate` does not poll the stop signal**: under `generate` the first
press does nothing visible. What an early stop should mean there (a truncated
sample with correct normalisation, or refusal) is undecided:
[generate-has-no-graceful-stop](../backlog/feature/generate-has-no-graceful-stop.md).

## Evidence and its limits

Verified by recorded capture or byte comparison: the stdout and artifact
invariances above, footer render, scrollback insertion, teardown, marker
lines, the level and scope filters actually moving, and both stop paths, all
under a DSR-answering PTY harness with a VT replayer.[^n33-closeout] Not
verified on a real terminal: everything "on screen" was read through that
harness; colour was read as SGR sequences, not pixels; terminal resize during
a run was never exercised (narrow layouts are unit-tested only); the
`generate` footer was captured on the fixed-energy path only.

Known rough edges, filed as
[tui-progress-and-consent-gaps](../backlog/feature/tui-progress-and-consent-gaps.md):
no `trials` on `progress::unweighting`; the line layer's format is fixed at
init from the starting level, so climbing to DEBUG at runtime keeps the
compact form; crossterm's cursor-position probe costs about 2 s before falling
back to plain lines where nothing answers DSR (bare `script(1)`). That item
also lists the consent prompt garbling the pane; `network::confirm` now routes
the question through the pane (`tui::ask_to_download`), so that part of the
item no longer describes the code.

## Non-goals

No JSON or structured log output, syslog or rotation; no TUI for
`check-events`; no retroactive re-filtering of scrollback; no async runtime and
no change to the rayon threading model; no Windows-terminal verification
beyond crossterm's defaults.

Profiling and per-stage timing are separate instruments:
[profiling](profiling.md).

[^n33-decisions]: Note 33 §1, decisions made by the user on 2026-08-05.
[^n33-closeout]: Note 33 §9, close-out, especially §9.1 (plan vs code) and §9.4 (verified vs asserted).
