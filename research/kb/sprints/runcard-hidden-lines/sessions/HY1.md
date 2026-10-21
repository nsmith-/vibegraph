---
type: Session Brief
title: "HY1: hygiene review of the hidden-line parser fix and the systematics warning"
description: "Review F1's diff for maintainability, test non-vacuity, visibility and reuse; fix what is local, with mutation evidence."
status: draft
agent: hygiene-dev (Sonnet)
depends_on: [F1]
closes: []
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-20}
---
Worktree `/home/user/wt/A`, branch `sprintd-runcard-hidden-lines`. Review
`git diff origin/main...HEAD` (F1, `de96d8c`, `16f19d6`) with
[hygiene review](../../../workflow/hygiene-review.md) as the checklist, and
[F1's report](F1-report.md). Pixi is not installed; run commands directly.

In particular:
- **The test-capture subscriber.** F1 found three private copies of the
  tracing capture subscriber (`progress.rs`, `budget.rs`, `classes.rs`), each
  needing `tracing::callsite::rebuild_interest_cache()` in scope. One shared
  `#[cfg(test)]` helper, with the cache trap handled once, is in scope here.
- **`split_line`'s doc** states MadGraph's rule in the code's own terms
  (MadGraph strips at `#`, then `!`, then splits at the last `=`:
  `banner.py:2900-2902`). This PR deliberately leaves the last-`=` split
  to `runcard-value-with-equals-dropped`, so the doc must say what the code
  does now, not what MadGraph does in full.
- **`warn_unwritten_systematics`**: visibility, whether its field list is
  derived from `FIELD_CLASSES` (`B_SYST`) or repeated by hand (a hand list can
  drift from the table), and whether the warning text names what the user
  loses.
- **Test non-vacuity**: the use_syst-off test asserts an absence. Does
  anything make it fail if the gate on `use_syst` is removed?

Gate: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`,
`cargo test --workspace`, `python3 scripts/kb.py lint`; mutation evidence
for each fix that adds or tightens a test. Write
`research/kb/sprints/runcard-hidden-lines/sessions/HY1-report.md` (findings
table written before any edit, then dispositions; commands with output;
**Found**).

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
