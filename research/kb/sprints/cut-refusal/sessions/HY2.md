---
type: Session Brief
title: "HY2: hygiene review of F2 (run card first; the shared CLI test module)"
description: "Review F2's diff for maintainability, test non-vacuity, visibility and reuse; fix what is local, with mutation evidence."
status: draft
agent: hygiene-dev (Sonnet)
depends_on: [F2]
closes: []
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-21}
---
Worktree `/home/user/wt/B`, branch `sprintd-cut-refusal`. Review
`git diff 86e3df6..HEAD -- vibegraph-cli` (F2: `30d51b4`, `66b5551`) with
[hygiene review](../../../workflow/hygiene-review.md) as the checklist, and
[F2's report](F2-report.md). Pixi is not installed; run commands directly.

In particular:
- **`vibegraph-cli/tests/common/mod.rs`**: `#![allow(dead_code)]` covers the
  whole module; is that the narrowest allow (each test binary compiles the
  module and uses a subset)? Do helper names and docs say what each sets up
  (`vibegraph_behind_dead_proxy`'s environment)?
- **`generate`'s new order**: the comment at the run-card load says why it
  comes before the artifact, in the code's own terms; no stale comment
  elsewhere still describes the old order.
- **The new ordering test**: does it fail if the order is reversed (F2's
  mutation says yes) *and* if the run card is not checked at all?
- **Any CLI test file F2 missed** that still copies the helpers (grep).
- Out of scope: the √s item (it moves to its own PR); the fixed-beam docs
  that still say E1 + E2 belong to that PR.

Gate: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`
(and with `--all-features`), `cargo test --workspace`, `python3 scripts/kb.py lint`;
mutation evidence for each fix that adds or tightens a test. Write and commit
`research/kb/sprints/cut-refusal/sessions/HY2-report.md` (findings table
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
