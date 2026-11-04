---
type: Session Brief
title: "G-F1: refuse spin 2 and the Lorentz/spin-code panics"
description: "A spin-2 particle, internal or external, and a Lorentz index naming no leg are refused with named errors; build_child's todo! and the three spin-code panic! arms return typed errors or are pinned unreachable."
status: draft
agent: feature-dev (Opus)
depends_on: []
closes: [spin-2-panics-instead-of-refusing, lorentz-and-spin-code-paths-panic-instead-of-refusing]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-03}
---
Worktree `/home/user/wt/G1-F1`, branch `sprintg-refusals-f1`, off
`sprintg-refusals` (main 86b8e5c plus the sprint folder). The manager merges
it into `sprintg-refusals`.

## Read first

- `python3 scripts/kb.py backlog --item spin-2-panics-instead-of-refusing` and
  `--item lorentz-and-spin-code-paths-panic-instead-of-refusing`: both bodies
  carry the cause, the sites and a reproducer.
- [Release scope](../../../pipeline/release-scope.md), the hard-error rule.
- Decision G1 in `../sprint.md`: refuse (option (a)); spin-2 support is out of
  scope.

## Work

1. A spin-2 toy model built as `tests/toy_models.rs`' `edited_toy_model` does
   (the item's reproducer: `gt`, spin 5, with `lt~ lt gt` and `vt vt gt`).
2. Refuse spin code 5 on **any** line, internal or external, with an error
   naming the particle. Decide where (UFO load, enumeration or
   `AmplitudeEvaluator::compile`) so that a model containing a graviton but a
   process not touching it still runs; say which and why.
3. Refuse a Lorentz index that names no leg of its vertex (UFO `to_isize` or
   `build_child`), so no such index reaches `lower_lorentz` as a `Leg`.
4. The four sites of the hygiene item: `build_child`'s `todo!` returns a
   `RootLorentzError` naming the operator and structure; each spin-code
   `panic!` returns its typed error or becomes `unreachable!` with the
   invariant stated and a test pinning the upstream refusal. Sweep the in-tree
   models for operators reaching the `todo!` arm and report the result.
5. Tests: internal spin-2 (`lt~ lt > vt vt`) and external (`lt~ lt > gt`)
   refused through the public compile path, each failing (panicking) before.

## Gate

- `cargo fmt --all --check`; `cargo clippy --workspace --all-targets -- -D warnings`,
  and again with `--features vibegraph/extended-validation`
- `cargo test --workspace` (includes the hermetic `amplitude_oracle` and
  `standalone_jamps`)
- the `extended-validation` skill's rows for `helas/` amplitude evaluation, as
  far as this container can run them (the banked amplitude gates); state which
  ran
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
