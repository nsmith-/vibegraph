---
type: Session Brief
title: "HY1: hygiene review of the early cut refusal and the no-op threshold"
description: "Review F1's diff for maintainability, test non-vacuity, visibility and reuse; fix what is local, with mutation evidence."
status: draft
agent: hygiene-dev (Sonnet)
depends_on: [F1]
closes: []
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-20}
---
Worktree `/home/user/wt/B`, branch `sprintd-cut-refusal`. Review
`git diff origin/main...HEAD` (F1, `cb18fd5`) with
[hygiene review](../../../workflow/hygiene-review.md) as the checklist, and
[F1's report](F1-report.md). Pixi is not installed; run commands directly.

In particular:
- **`detect_unimplemented` became `pub`.** Is that the narrowest visibility
  the CLI call needs, and is the function's contract (what it reads, that it
  needs no legs) documented where a caller sees it?
- **`SINGLE_PT_MAXIMA`.** Each of the nine names: does MadGraph's run card
  define it with the meaning the bound assumes (a single object's pT, or a
  missing-ET vector sum)? A name that is not a single-object pT maximum must
  not be in the list. Is the ceiling's derivation stated once, at the
  constant, in the code's own terms?
- **Test non-vacuity.** F1 notes
  `integrate_refuses_an_unimplemented_cut_naming_it_and_its_value` stays
  green with the early call removed. Is it still worth keeping, and does its
  name say what it pins? Do the CLI tests share helpers with the existing
  CLI test files rather than re-implementing card/proc writing?
- **One implementation.** Confirm the compile-time path and the early path
  call the same function and cannot drift.

Gate: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`,
`cargo test --workspace`, `python3 scripts/kb.py lint`; mutation evidence
for each fix that adds or tightens a test. Write
`research/kb/sprints/cut-refusal/sessions/HY1-report.md` (findings table
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
