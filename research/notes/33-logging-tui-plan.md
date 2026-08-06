# 33 — Logging & terminal UI (plan)

**Status:** PLAN, drafted 2026-08-05. Design settled with the user in the
originating session; §7 lists the points still open. Baseline: `main` @
`319af7d` (post note-32 close-out; drafted before the channel-dedup census
close and the interned license notices merged — neither touches a surface this
plan names). This is a UX/plumbing sprint — no physics
surface moves, no tolerance changes, and the validation layer is a regression
gate here, not a target.

## 0. Motivation and scope

The CLI today communicates through ~40 ad-hoc `println!`/`eprintln!` sites in
`vibegraph-cli` (`integrate.rs`, `generate.rs`, `check.rs`, `assets.rs`,
`main.rs`); the library is silent (its only print sites are inside
`#[cfg(test)]` modules, and stay there). There is no level control, no way to
see pipeline internals without a debugger, and no progress indication during
multi-minute integrations.

This sprint delivers:

1. **Structured logging** via `tracing` throughout `vibegraph-lib`, with
   MadGraph-style notices at `info` and pipeline internals at `debug`/`trace`.
2. **A plain-lines mode** (stderr) when output is piped — the current stdout
   result contract is preserved byte-for-byte.
3. **A sticky-footer terminal UI** on a real terminal: log lines scroll into
   the terminal's own history above a 6-line ratatui inline-viewport status
   pane (σ ± err, progress bar, model/process brief, per-eval timing, animated
   logo).
4. **Runtime interactivity**: arrow keys raise/lower the visible log level and
   cycle a module-scope filter mid-run.

## 1. Decisions already made (user, 2026-08-05)

- **`tracing`, not `log`.** Structured fields drive the status pane without
  the library knowing a UI exists; spans give stage attribution and timing;
  `EnvFilter` gives `RUST_LOG`-style per-module control. `vibegraph-lib` gains
  a dependency on `tracing` only (macros + spans, no subscriber); all
  subscriber choice lives in the CLI.
- **Inline viewport, not alternate screen.** ratatui `Viewport::Inline(6)`
  with `Terminal::insert_before()` pushes log lines into the terminal's real
  scrollback — native scroll/search/copy work, history survives exit. Build
  ratatui with the `scrolling-regions` feature (v0.29+) so `insert_before`
  scrolls without redrawing the footer (flicker-free at high log rates).
  `tui-logger` is NOT used: there is no log widget, the terminal is the log
  pane.
- **stdout is for results only.** Final σ line, `wrote <path>`, and the
  `check-events` report are program output and stay on stdout, unchanged. All
  tracing output goes to stderr (plain mode) or through the footer-managed
  scrollback (TUI mode). Machine consumers (`… | jq`, the extended-validation
  CLI tests that parse stdout) must observe no change.
- **On-screen filtering is prospective-only** (scrollback is immutable once
  emitted). Mitigation: optional `--log-file <path>` sink recording at `trace`
  regardless of the on-screen level.
- **Graceful teardown**: on exit (success, error, or Ctrl-C) clear the footer
  and leave a final plain summary line so scrollback ends with the result, not
  a dead dashboard. Panic hook restores the terminal before the panic message
  prints.

## 2. Architecture

### 2.1 Emission (library side)

- Events use default module-path targets (`vibegraph_lib::diagrams`, …) so
  `EnvFilter` scoping falls out for free.
- **Progress taxonomy**: machine-consumed events use the dedicated target
  `"vibegraph::progress"` with fields `stage: &str`, `done: u64`,
  `total: Option<u64>`, plus stage-specific fields (VEGAS iterations carry
  `sigma`, `err`, `chi2`; unweighting carries `accepted`, `requested`;
  eval timing carries `ns_per_eval`). These are emitted at `TRACE` so plain
  mode never prints them as lines; the TUI installs a dedicated layer with its
  own filter that receives them regardless of the visible level.
- Stage spans (`ufo_load`, `enumerate`, `compile_channel`, `vegas`, …) wrap
  each pipeline stage; span close timings are the per-stage timing story.
- **Determinism guard**: instrumentation must not touch RNG state, sampling
  order, or artifact bytes. The `-j` byte-identical-artifact assertion and all
  fixed-seed tests are regression gates on this.
- Rayon workers emit freely — `tracing` is thread-safe; only the CLI's main
  thread touches the terminal.

### 2.2 Sinks (CLI side)

Mode selection: TUI iff stdout AND stderr are terminals
(`std::io::IsTerminal`), overridable with `--tui` / `--no-tui`. Level flags:
`-v`/`-vv` (info/debug), `-q` (warn), `--log-level <level>`; `RUST_LOG` is the
expert override via `EnvFilter`. Default level: `info` in both modes (§7.1).

Three `tracing-subscriber` layers on one registry, each with its own filter:

1. **Line layer** — formats human-readable events. Plain mode: writes stderr
   (compact `fmt`, no timestamp at `info`, target+uptime shown at `debug+`).
   TUI mode: formats to `String` and sends over a channel to the main loop,
   which calls `insert_before` then redraws. Filter is wrapped in
   `reload::Layer` so arrow keys can swap it at runtime.
2. **Progress layer** (TUI only) — matches target `"vibegraph::progress"`,
   folds fields into a shared `UiState` (mutex'd struct: stage, done/total,
   σ ± err, χ²/dof, ns/eval, model+process brief). Own filter, unaffected by
   the visible-level reload.
3. **File layer** (when `--log-file`) — full `fmt` at `trace` to the file.

### 2.3 TUI main loop

Single thread owns the terminal: poll crossterm events (~50 ms tick), drain
the line channel (`insert_before` each line), apply key presses, redraw the
footer from `UiState`. Raw mode is on (required for keys), so Ctrl-C arrives
as a key event: first press = graceful abort (finish current VEGAS iteration,
save the grid for what's converged, summary line), second press = immediate
exit. `q` behaves like the first Ctrl-C.

Keys, with current state always visible in the footer
(`level: INFO ▲▼  scope: all ◂▸`):

- **↑ / ↓** — visible level through warn ⇄ info ⇄ debug ⇄ trace, via the
  reload handle. On change, `insert_before` a marker line
  (`── log level → DEBUG ──`) so the history is self-explaining.
- **← / →** — cycle module scope for the debug/trace tiers
  (all → diagrams → helas → vegas/phasespace → pdf/hadronic), as `EnvFilter`
  directive swaps through the same handle.

### 2.4 Footer layout (6 rows, full width)

```
├────────────────────────────────────────────────────────────────────┤
│ ~vibegraph~     SM (d41f…)  17p 64v 12c │                          │
│ p p > e+ e-  QED=2 QCD=0    6 channels  │   σ = 802.94 ± 3.11 pb  │
│ integrating  iter 4/10   χ²/dof 1.02    │        (bold)            │
│ ██████████████████░░░░░░░░  1.2M/2.0M   │   eval  212 ns  4.7 M/s │
│ level: INFO ▲▼  scope: all ◂▸           │   q/^C graceful abort    │
└────────────────────────────────────────────────────────────────────┘
```

Left column: rainbow-wavey logo (per-character RGB, hue phase advanced each
tick), UFO brief (particle/vertex/coupling counts + model digest), process
card brief (process, coupling orders, channel count), stage line, `Gauge`
progress bar. Right column: σ ± err large/bold, per-eval timing. `generate`
phase swaps the σ cell for accepted/requested + unweighting efficiency and the
gauge tracks accepted events. Footer widgets are pure functions of `UiState` —
unit-testable against `ratatui::backend::TestBackend` buffers.

### 2.5 SI-prefix unitful formatter

One CLI-side utility (`fmt_si(value, unc: Option<f64>, unit, width)`):
chooses the prefix from the value's magnitude (fb…nb for pb-based cross
sections; ns/µs/ms for seconds), renders value and uncertainty in the **same**
prefix, fixed decimal width so live updates don't jitter. Unit tests pin
prefix boundaries (999.95 → 1.00 k), the unc-larger-than-value case, and
width stability across a decade sweep.

### 2.6 Per-stage instrumentation plan

| Stage | `info` | `debug` | `trace` |
|---|---|---|---|
| UFO load | model label + digest | particle/vertex/coupling/parameter counts, coupling-order table | per-vertex Lorentz structures |
| Proc card | resolved process list, coupling orders | alias expansion, selector decisions | — |
| Enumeration | "N diagrams for u d~ > e+ ve" per subprocess | topology counts, charge-conservation prefilter kill rate, per-process timing | per-diagram vertex assignments |
| Color | flows per channel, CF matrix size | flow-tag assignments | matrix entries |
| Eval compile (per channel) | "compiled evaluator: N ops" | per-pass stats from `lower`/`fold`/`schedule`/`layout` (nodes before/after CSE, arena size, roots) | per-pass op dumps |
| PDF | set name + member, cache hit vs fetch | grid ranges, interpolation setup | — |
| Phase space | channel list + allocation mode | per-channel maps, cut summary | — |
| VEGAS | per-iteration: iter k, σ ± err, χ²/dof | per-channel per-iteration results, grid-adaptation deltas, α rebalance | per-batch detail |
| Result | σ ± err, evals/s, artifact path | convergence record (stop reason, budget) | — |
| Generate | artifact + strategy summary, max-weight search result, unweighting efficiency | batch efficiency evolution | per-event weights |

`check-events` keeps its stdout report as-is (it *is* the output). The
network-consent prompts in `assets.rs`/`network.rs` are interactive UI, not
logging — untouched.

Progress totals: channel-compile loops know their channel count up front;
VEGAS knows `neval × niter` (or the `--target-rel` budget cap); unweighting
knows the requested event count. Enumeration emits `total: None` until the
assignment expansion is counted. Where a `Vec` return currently hides the
count (`generate_from_proc_card` → `Vec<DiagramSet>`), prefer emitting
progress events inside the loop over changing signatures; an
`ExactSizeIterator` refactor is welcome where it falls out naturally but is
not this sprint's obligation. (The `generate-stream` Part B backlog item —
lazy event iterator — is related in spirit but separate and stays in the
backlog.)

## 3. New dependencies

Per the AGENTS.md never-hand-write rule, all standard machinery comes from
crates: `tracing` (lib + CLI), `tracing-subscriber` with `env-filter` +
`registry` + `fmt` (CLI), `ratatui` with `scrolling-regions` (CLI),
`crossterm` (CLI; use the version ratatui re-exports to avoid a dual-version
event-type mismatch). No `log`, no `tui-logger`, no `indicatif`, no async
runtime.

## 4. Sessions

Scoping ground rules as note 28 §2: one session per dispatch, sprint branch
per session, AGENTS.md comment guidelines binding (no sprint/session names in
code), every session runs `cargo test` + clippy and pastes commands with
output.

### T1 — tracing plumbing + plain mode + notice migration

Add the dependencies; build the CLI subscriber skeleton (line layer with
reload-wrapped filter, optional file layer; no TUI yet); wire `-v`/`-vv`/
`-q`/`--log-level`/`--log-file`/`RUST_LOG`; migrate every existing notice
`println!`/`eprintln!` in `integrate.rs`/`generate.rs` to `tracing` events at
the §2.6 levels, keeping the stdout result lines (`σ = …`, `wrote …`, the
`check-events` report) exactly as they are.

**Gate**: full default `cargo test`; `cli_integrate` (extended-validation)
unchanged — it parses stdout, which must not move; a before/after diff of
`integrate` stdout on one partonic row showing only notice lines migrated to
stderr; clippy clean.

### T2 — pipeline instrumentation + progress taxonomy

Implement §2.6 through the library: stage spans, per-stage events, the
`vibegraph::progress` target with its field contract (write the contract as
rustdoc on a small `progress` module in the lib so T4 codes against a named
surface, not a convention). VEGAS per-iteration events in the adapt loops
(`vegas.rs`, `hadronic.rs` drivers); unweighting progress in `unweight.rs`;
compile-pass stats in `helas/eval/compile.rs`.

**Gate**: full default `cargo test`; artifact bytes identical with `-q` vs
`-vv` on the same seed (determinism guard, §2.1); `integrate` at `-v` and
`-vv` on a partonic quick row with output pasted into the report; the
`-j 16` byte-identical assertion re-run.

### T3 — SI-prefix formatter (light session)

§2.5 as specified, with its unit tests. No consumers changed yet beyond the
existing σ result line if trivially adoptable.

**Gate**: `cargo test` on the new tests; clippy.

### T4 — sticky-footer TUI

Mode detection + `--tui`/`--no-tui`; inline viewport, line channel,
`insert_before` loop, `UiState` + progress layer, footer per §2.4 (static
logo acceptable this session), graceful teardown + panic hook. No key
handling yet beyond Ctrl-C/q abort.

**Gate**: `cargo test` incl. TestBackend footer-render tests; manual
verification protocol pasted into the report — run `integrate` on a real
terminal (footer present, lines scroll into history, exit leaves summary) and
piped (`… 2>&1 | cat` — byte-identical to T2 plain mode); artifact
determinism spot-check TUI vs piped run, same seed.

### T5 — interactivity + polish + close-out

Arrow-key level/scope cycling via the reload handle with marker lines; the
two-stage Ctrl-C graceful abort (save converged grid); rainbow logo
animation; `generate`-phase footer variant; README section on logging flags
and the TUI; TODO.md close-out.

**Gate**: `cargo test`; manual key-interaction protocol pasted; one full
`integrate` → `generate` run under the TUI on a real terminal; `pixi run
--skip-deps validate` green as the sprint's exit check (nothing here should
touch it — a diff is a finding).

## 5. Sequencing

T1 → T2 → T4 → T5, strictly (each codes against the previous session's
surface). T3 is independent after T1 and must land before T4 (the footer
consumes it). Five dispatches, no parallel tracks needed; T3 may run
alongside T2 in a separate worktree.

## 6. Non-goals

- No JSON/structured log output, no syslog, no log rotation.
- No TUI for `check-events` or the fetch/consent flow.
- No retroactive scrollback re-filtering (accepted trade of inline mode).
- No async runtime; no change to the rayon threading model.
- No Windows-terminal verification beyond what crossterm provides by default.
- No library API stability promise for the progress-event contract (0.x line;
  the quality sprint owns API surface decisions).

## 7. Open decisions (user)

1. **Default level `info` in both modes** (MG-like notices on by default,
   piped consumers read stdout anyway) — recommended and assumed by T1;
   alternative is `warn` when piped. Flip is a one-line change if wanted.
2. **Footer aesthetics** (exact cell placement, logo styling) — T4/T5 have
   latitude within §2.4's content list; not gated.
3. **Whether the σ result line on stdout adopts `fmt_si`** (changes a string
   the extended-validation tests may parse) — default: leave stdout frozen
   this sprint, revisit at v0.2.

## 8. Agents

feature-dev throughout. T1, T2, T4 default to Opus (broad surface or
threading/terminal subtleties); T3 and T5 may run on Sonnet at the manager's
discretion. Manager owns worktrees per AGENTS.md Sprint & Subagent
Operations; no MG reference data is needed by any session except T1's
`cli_integrate` gate (COW-copy the refdata in, or the gate silently triggers
regeneration).

## 9. Close-out (manager, 2026-08-06)

**Status: CLOSED.** Five sessions T1–T5, five branches, all merged to `main`
in plan order (`logtui-t1` → `-t2`/`-t3` → `-t4` → `-t5`); default
`cargo test` run green on `main` after every merge. Exit check: T5's
`pixi run --skip-deps validate` exit 0, every gate cell passed, the measured
cells exactly the manifest's declared set — no diff, so no finding. The two
§7 defaults stood unexercised by anything the sprint found: **default level
stayed `info` in both modes** (§7.1) and **the stdout σ line stayed frozen**
(§7.3); both remain one-line changes.

### 9.1 Where the plan and the code disagreed, the code won

Recorded so the next reader patches the note's model of the codebase:

- **§2.1's target names are wrong**: the lib crate's library name is
  `vibegraph`, not `vibegraph_lib`, so targets are `vibegraph::diagrams`, …
  — and `vibegraph::progress` is literally the `progress` module's own path.
- **§4 T2's file list**: the production VEGAS iteration loop is
  `budget.rs::integrate_channels` (both `hadronic.rs` and `proton.rs`
  `adapt_grids_budget` delegate to it; `vegas.rs`'s `adapt_*` family is
  test-only). Requested-count unweighting loops live in `lhef/emit.rs`, not
  `unweight.rs` (which got the weight-scan instrumentation instead).
- **The compile stage span is `compile`, not `compile_channel`** — the
  compile loop iterates subprocesses/flavour groups; "channel" in this
  codebase means a per-diagram phase-space map.
- **§2.1's "TRACE keeps progress events off plain mode" was false as built**
  (`-vvv` printed them); T5 filters `vibegraph::progress=off` in the line
  layer at every level. `--log-file` still records them.
- **§2.4's unweighting-efficiency cell is not derivable from the progress
  contract** (no trial count on the stream); the footer shows the scan's
  predicted efficiency (`σ / Σⱼ w_maxⱼ`) pushed in by the CLI — it read
  30.6% predicted vs 30.03% achieved on the verification run. Follow-up
  filed: add `trials` to `progress::unweighting`.
- The level ladder is six rungs (off…trace), a superset of §2.3's four;
  scope narrowing applies only above INFO so it can never swallow a warning.
- T1 found `cli_integrate` never parsed stdout (the stdout-parsing extended
  test is `cli_generate_proton`); and the default-feature `cli_generate`
  test read `scan:` off stdout — §1's exhaustive stdout contract governs, so
  `scan:` moved to stderr and the test helper followed.

### 9.2 Decisions taken by the manager during the sprint

- **Span-close timings (§2.1) are not enabled** — `FmtSpan::CLOSE` would
  change plain-mode output shape for every run; each heavy stage instead
  emits its own elapsed at `debug`. Revisit if per-stage timing becomes a
  consumer surface.
- crossterm is consumed as ratatui's re-export (0.29) rather than a separate
  manifest entry — §3's own dual-version rationale.
- One extra dependency beyond §3's list: `unicode-width` (already in-tree
  transitively) for display-width line breaking, because `insert_before`
  truncates if the height undercounts.

### 9.3 What the abort work surfaced

A graceful stop during warm-up banks no kept iteration and the combined σ is
NaN; the first implementation would have printed `σ = NaN` and banked an
artifact holding exactly the grids the warm-up discard exists to throw away.
Fixed in `a791c87`: `ConvergenceReport` carries `kept_iterations`, and a run
stopped before any kept iteration is refused (exit 1, no artifact, no σ
line), pinned by `budget.rs` tests and a PTY run. The `StopSignal` is an
explicit parameter of `integrate_channels`, not a global, and its inertness
on non-aborted runs is pinned by byte-identical artifacts vs the pre-T5
binary on both a fixed-energy and a hadronic row.

### 9.4 Verified vs asserted

Verified by recorded capture or byte comparison: stdout frozen at every
verbosity (raw-byte hashes, T1/T4/T5); artifacts byte-identical across
`-q`/`-vv`, TUI/piped, and pre/post-abort-plumbing binaries; the thread-count
assertion re-run (T2); footer render, `insert_before` scrollback,
teardown, marker lines, level/scope filter actually moving, graceful and
immediate abort exits — all under a DSR-answering PTY harness with a VT
replayer (captures in the session records).

Asserted, NOT re-verifiable by the manager: everything "on a real terminal"
— the PTY harness stands in for one throughout; colour was read as SGR
sequences, not rendered pixels; terminal resize during a run was never
exercised (narrow-terminal layouts are unit-tested only); the generate
footer variant was captured on the fixed-energy path only (the hadronic path
shares `report_scan` but was not captured under the PTY).

### 9.5 Filed to the backlog

- `generate` has no graceful-stop poll: only `integrate_channels` reads the
  signal, so under `generate` the first `q`/`^C` does nothing visible.
  Stopping accept/reject early is a design decision (truncate the sample or
  refuse), not a wiring gap.
- `trials` field for `progress::unweighting` (needs `EventSource` to expose
  its trial count).
- The network-consent prompt is incompatible with the pane (raw mode +
  viewport); needs a suspend/resume around `network::confirm`. Only fires on
  an uncached PDF fetch, so on no gate path today.
- Cosmetic: the line layer's format is fixed at init from the starting
  level, so a runtime climb to DEBUG keeps the compact form; crossterm's
  cursor-position probe costs ~2 s before falling back to plain lines where
  nothing answers DSR (bare `script(1)`; real terminals answer immediately).
