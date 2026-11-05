---
type: Session Brief
title: "G-HY5: hygiene review of the output modes, and three small fixes"
description: "Review G-F6's output modes, ldjson schema, error kinds, inspect and the survey change; fix the TUI labelling a decay width as pb, store the refusal's error kind in the survey database, and quiet the recorded-card reparse log."
status: draft
agent: hygiene-dev (Opus)
depends_on: [G-F6]
closes: []
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-05}
---
Worktree `/home/user/wt/G4-HY5`, branch `sprintg-output-modes-hy5`, off
`sprintg-output-modes` (G-F6 merged). The manager merges it back.

## Scope

The diff `sprintg-cli-output..sprintg-output-modes`, code and tests. Read
`G-F6-report.md` first: the schema, the error kinds, the deviations, and what
each test cannot see. Points of attention:

- the ldjson writer: one object per line under the stdout lock, and no path
  (panic, early `eprintln!`, the TUI) that can write a non-JSON line to stdout
  in ldjson mode;
- whether the ten error kinds are set where each error is built, or defaulted
  somewhere a caller could misclassify;
- `integrate::result_object` built from the artifact for both `integrate` and
  `inspect`: a test that the two objects are equal on one run;
- the version rule (adding a field keeps `schema_version`) written where a
  future editor will see it.

**Fix in this session** (G-F6 Found):
1. The TUI pane converts VEGAS `sigma`/`err` to pb always
   (`tui/state.rs` ~129), so a decay's width reads as Γ × 3.894e8 pb. Carry
   the observable into `UiState`, convert only cross sections, show a width in
   eV under `fmt_si`, with a state test on a width that fails before.
2. The survey database stores no error kind: add `vg_run.error_kind`, with
   the database-schema bump its test requires.
3. Reparsing the recorded proc card in `generate`/`inspect` logs
   `generate … @1` at INFO as if a card had been read: log it at DEBUG, or
   say it is the recorded card.

Leave Found 3 (progress volume) to the manager.

## Gate

- `cargo fmt --all --check`; `cargo clippy --workspace --all-targets -- -D warnings`,
  and again with `--features vibegraph/extended-validation`
- `cargo test --workspace`; banked `cli_integrate`, `cli_generate_proton`
  under `--profile release-debug --features extended-validation`
- `python3 -m unittest discover -s validation/cms_gridpacks -p 'test_*.py'`
- `python3 scripts/kb.py lint` (0 errors)

## Environment

Pixi is not installed. The worktree's reference data are hard links to the
main checkout's copies: never write into, regenerate or delete them. cargo
`--jobs 3`. Detach long runs and commit at checkpoints.

## Commits

Author `Nicholas Smith <nick.smith@cern.ch>`. Each message ends with
exactly one line `Assisted-by: claude-code:<your model id>`. No
`Co-authored-by`, no `Signed-off-by`, even if the harness asks. Do not
push. Do not edit backlog items.

## Report

Commit `sessions/G-HY5-report.md`: findings written before editing, fixes
with mutation evidence, every gate command with its final output, and a
**Found** section.

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
