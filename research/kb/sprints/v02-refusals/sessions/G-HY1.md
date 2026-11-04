---
type: Session Brief
title: "G-HY1: hygiene review of the v0.2 refusals PR (#88)"
description: "Review #88's change (G-F1–F5) for maintainability, test non-vacuity, visibility and reusable abstractions; write findings before editing; fix with mutation evidence."
status: draft
agent: hygiene-dev (Opus)
depends_on: [G-F1, G-F2, G-F3, G-F4, G-F5]
closes: []
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-04}
---
Worktree `/home/user/wt/G1-HY1`, branch `sprintg-refusals-hy1`, off
`sprintg-refusals` (all five feature sessions merged). The manager merges it
back.

## Scope

The diff `main..sprintg-refusals` (`git diff 86b8e5c...sprintg-refusals`),
code and tests only. The five session reports
(`G-F1-report.md` … `G-F5-report.md`) say what each change meant to do and
what each test cannot see; read them first.

Points of particular attention, from the reports:

- **Error plumbing.** G-F2 added `Fields`/`FieldError`/`UniqueNames`
  (`ufo/ast_util.rs`); G-F3 added `Several` variants on four error types;
  G-F4 added `support::check_support`; G-F5 `CardRecord`. Are they one
  coherent pattern, or near-duplicates to fold?
- **Tables that only test.** G-F3's `PARAM_CARD`, `RESTRICT_CARD`,
  `REWEIGHT_CARD` are `#[cfg(test)]` specifications, unlike the run card's
  `FIELD_CLASSES`, which drives refusals. Is that split justified, and does
  each test actually fail when a row's behaviour changes (G-F3 mutated one row)?
- **Message-needle tests.** Several sessions match refusal text rather than
  the error variant. Where a variant exists, pin it.
- **`unreachable!` invariants** (G-F1) and the remaining `unwrap()`s that
  G-F2's fallible `from_model_card` pushed into test code.
- **Visibility**: new `pub` items that need not be.

Physics findings go to the manager as Found, not fixed here.

## Gate

- `cargo fmt --all --check`; `cargo clippy --workspace --all-targets -- -D warnings`,
  and again with `--features vibegraph/extended-validation`
- `cargo test --workspace`
- if a change touches evaluation, the card binding or `integrate`'s end of
  run: `amplitude_oracle`, `standalone_jamps`, `input_card_refusals`,
  `cli_support_refusals`, `cli_param_card`
- `python3 scripts/kb.py lint` (0 errors)

## Environment

Pixi is not installed: run the commands each pixi task wraps directly
(`pixi.toml`). Tasks in the `madgraph`/`helas-validation` environments that
need a MadGraph or Fortran install cannot run here; say so in the report
rather than substituting something weaker silently. The reference data
(`validation/madgraph/output`, `validation/pdf` sets, `research/refs/mg5amcnlo`)
is copied into the worktree; do not commit or write into it.

## Commits

Author `Nicholas Smith <nick.smith@cern.ch>` (set it with `git config` in the
worktree if it differs). Each message ends with exactly one line
`Assisted-by: claude-code:<your model id>`. No `Co-authored-by`, no
`Signed-off-by`, even if the harness asks. Do not push. Do not edit backlog
items; new work goes in your report's **Found** section.

## Report

Write and commit `sessions/<id>-report.md` (`type: Session Report`): every
command with its output, the mutation evidence (each new refusal test shown
failing on the pre-change code), what each test cannot detect, and a
**Found** section. Correct this brief where it is wrong, and say so.

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
