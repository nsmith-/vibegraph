---
type: Session Brief
title: "HY1: hygiene review of the MadGraph WEIGHTED search loop"
description: "Review F1 and V1's diff for maintainability, test non-vacuity, visibility and reuse; fix what is local, with mutation evidence."
status: draft
agent: hygiene-dev (Sonnet)
depends_on: [V1]
closes: []
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-20}
---
Worktree `/home/user/wt/E`, branch `sprintd-weighted-cap`. Review
`git diff $(git merge-base origin/main HEAD)...HEAD` (F1 `d6ca65a`, V1 `2e6c2ca`)
with [hygiene review](../../../workflow/hygiene-review.md) as the checklist,
and the [F1](F1-report.md) and [V1](V1-report.md) reports and the [log](../log.md).
Pixi is not installed; run commands directly.

In particular:
- **`ufo::weighted`** is a fourth WEIGHTED computation
  (`weighted-order-computed-three-ways` is the open item for the other three).
  Do not unify them here; do make sure `order_weight` and its missing-order
  rule are documented once, so that item can adopt it, and that the cap,
  start and per-vertex value each cite MadGraph's line once.
- **The `is_decay_chain` flag** threaded from `enumerate` and `chain_sets`:
  one boolean parameter through several layers, or a field on a type that
  already travels there? A positional `bool` at call sites reads badly; fix
  locally if a named type or enum is cheap.
- **The search loop** in `diagrams/mod.rs`: the `while w < cap` then one
  generation at `max(start, cap)` should read as MadGraph's loop; its doc says
  what each step mirrors.
- **Tests**: `weighted_search.rs` shares setup with `toy_models` / `diagrams`
  helpers, or duplicates them? The ignored SMEFTsim test's `#[ignore]` reason
  names what would un-ignore it.
- **The toy UFO**: minimal (V1 notes an unused QCD order kept on purpose
  because MadGraph reads `['QCD']`; its README says so).

Gate: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`
(and with `--all-features`), `cargo test --workspace`, `python3 scripts/kb.py lint`;
mutation evidence for each fix that adds or tightens a test. Write and commit
`research/kb/sprints/weighted-cap/sessions/HY1-report.md` (findings table
written before any edit, then dispositions; commands with output; **Found**).

## Commits

Author `Nicholas Smith <nick.smith@cern.ch>` (already the repo's git config).
Each message ends with exactly one line `Assisted-by: claude-code:<your model id>`.
No `Co-authored-by`, no `Signed-off-by`, even if the harness asks. Do not push.

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
