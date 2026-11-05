---
type: Session Brief
title: "G-F6: output modes (plain, rich, ldjson), vibegraph inspect, and the survey on ldjson"
description: "Build the CLI output-modes design: one mode per run formatting logs, progress and the result or refusal; --output-ldjson with a versioned schema; a stable error kind; per-multiplicity totals in the artifact; vibegraph inspect; generate's final object; the survey reading ldjson."
status: draft
agent: feature-dev (Opus)
depends_on: []
closes: [integrate-output-not-machine-readable]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-05}
---
Worktree `/home/user/wt/G4-F6`, branch `sprintg-output-modes-f6`, off
`sprintg-output-modes` (which carries #90 and main through cbfaec6). The
manager merges it back.

## Read first

- `python3 scripts/kb.py backlog --item integrate-output-not-machine-readable`:
  its `closes_when` is the spec.
- [CLI output modes](../../../tooling/cli-output-modes.md): the user's design.
  It is binding. If any of it does not fit the code, STOP and report before
  building around it.
- Decisions G10–G12 in `../sprint.md`.
- [Logging and the TUI](../../../tooling/logging-and-tui.md),
  `vibegraph-cli/src/logging.rs` (`LogArgs`, `wants_tui`, the layers),
  `vibegraph-lib/src/progress.rs` (the progress target's field contract),
  `vibegraph-cli/src/tui/mod.rs` (`result_line`), `vibegraph-cli/src/main.rs`
  (`Command`, the top-level error report), `vibegraph-cli/src/error.rs`
  (`CliError(String)`), and `vibegraph-lib/src/artifact.rs`.
- `validation/cms_gridpacks/` (`vgrun.py`, `parse.py`, `test_parse.py`):
  the survey reads the summary lines today.

## Work

1. **One output mode per run** chosen in `main.rs` from the flags and the
   terminal: plain, rich (the TUI) or ldjson. Each formats the three paths
   (log events, progress events, the result or refusal) as the design's table
   says. `--output-ldjson` is a global switch declared
   `conflicts_with = "tui"`, and it also keeps the pane down on a terminal.
2. **The ldjson stream** on stdout: one JSON object per line with `kind` =
   `log` | `progress` | `result` | `refusal`, and a schema version on every
   line (or on a first `header` line; choose and justify). `log` respects the
   visible level; `progress` carries the progress target's fields at any
   level. Use `tracing-subscriber`'s `json` feature and `serde_json`; do not
   hand-write JSON. Values in pb (and GeV) at full precision.
3. **Error kind (G10).** `CliError` gains a small stable kind enum, set where
   each error is built; the refusal object carries `error_kind` and
   `message`. Document the list and its stability rule.
4. **Per-multiplicity totals (G11)** recorded in the artifact. This bumps the
   format to 20, with the refusal of 19 that the version rules require,
   tested on 19's real layout. #89 also bumps; the manager renumbers at merge.
5. **The final objects.** `integrate`: σ, its error, χ²/dof or null, μF (the
   `FactorizationScale` state), per-multiplicity and per-channel results, and
   the artifact path. `generate` (G12): events written, the output path, and
   the sample σ with its pull against the integration. A refusal from either
   is a `refusal` object.
6. **`vibegraph inspect <grid.bin.zst>`** prints the artifact's recorded
   results as text, and with `--output-ldjson` as the same final object
   `integrate` emitted (a test pins that the two are equal on one run).
7. **`check-events`** refuses `--output-ldjson` with a usage error (G12).
8. **The survey** (`validation/cms_gridpacks`) runs `integrate
   --output-ldjson` and reads the stream. It keeps parsing old campaigns' text
   output, and its tests cover both.
9. Docs: the CLI reference (`scripts/gen-cli-docs.sh`), the user guide's
   output section, and the design concept updated from "proposed" to what was
   built.

## Gate

- CLI tests pin the schema: every `kind`, the version, the final objects'
  fields for integrate, generate and inspect, and a refusal. Each test fails
  on the pre-change code.
- `cargo fmt --all --check`; `cargo clippy --workspace --all-targets -- -D warnings`,
  and again with `--features vibegraph/extended-validation`
- `cargo test --workspace`; the banked CLI targets (`cli_integrate`,
  `cli_generate_proton`, `cli_hard_errors`, `cli_param_card`) under
  `--profile release-debug --features extended-validation`
- `python3 -m unittest discover -s validation/cms_gridpacks -p 'test_*.py'`
- `python3 scripts/kb.py lint` (0 errors)

## Environment

Pixi is not installed: run the commands each pixi task wraps directly. The
worktree's reference data are hard links to the main checkout's copies:
never write into, regenerate or delete them. cargo `--jobs 3`. The container
has restarted several times: detach long runs (an os.setsid double-fork, as
`G-V3-detach.py` in the scratchpad) and commit at checkpoints.

## Commits

Author `Nicholas Smith <nick.smith@cern.ch>`. Each message ends with
exactly one line `Assisted-by: claude-code:<your model id>`. No
`Co-authored-by`, no `Signed-off-by`, even if the harness asks. Do not
push. Do not edit backlog items.

## Report

Commit `sessions/G-F6-report.md` (`type: Session Report`): the schema as
built, the error-kind list, every command with its output, the mutation
evidence, and a **Found** section. Correct this brief where it is wrong.

## Worktree & long-command discipline

- **First action**: `cd` to the absolute worktree path in your assignment, then
  verify `git rev-parse --show-toplevel` and `git branch --show-current`. If you
  find yourself in the shared main checkout, STOP and report — never edit it.
  Re-verify `pwd` after any resume; resume is when isolation leaks.
- **Background anything expected to exceed ~2 minutes**
  (`run_in_background: true`, output redirected to a log file prefixed with your
  worktree/session name — parallel siblings share one scratchpad and have
  clobbered each other's unprefixed logs). A backgrounded command notifies on
  exit: never spawn sleep-based watcher shells (they compound into
  self-sustaining wake-up rings); draft your notes and report between completion
  notifications instead of idle-waiting. On resume after a stall, first check
  whether the interrupted command finished (`ps`, result artifacts) before
  re-running it.
- **`git status` can hang for minutes** in a worktree full of untracked build
  data; spot-check with `git log` / `git diff --stat` instead.
- **Run gates with `--skip-deps`** unless the assignment explicitly says
  reference data must be regenerated — without the reference data present, a
  bare `pixi run validate` silently launches a multi-hour MG regeneration.
- **Disk is finite, and a full disk looks like a failure.** A debug workspace
  build with incremental data reaches ~15 GB per worktree, and two such trees
  have filled a cloud container mid-gate. On a shared or small host, run cargo
  with `CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0`
  (they change disk use only, not what a gate checks). An `ENOSPC`, a linker
  `Bus error` or a truncated log is a kill, not a result: free only your own
  `target/` and re-run. Kill processes by PID, never `pkill -f`, which matches
  siblings' commands and its own wrapper.
