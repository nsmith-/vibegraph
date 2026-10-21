---
type: Session Brief
title: "HY1: hygiene review of the conditional-definition refusals and the fallible recompute"
description: "Review F1 and F2's diff for maintainability, test non-vacuity, visibility and reuse; pin the no-aS rescale path; fix what is local, with mutation evidence."
status: draft
agent: hygiene-dev (Sonnet)
depends_on: [F2]
closes: []
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-20}
---
Worktree `/home/user/wt/C`, branch `sprintd-ufo-conditional-params`. Review
`git diff $(git merge-base origin/main HEAD)...HEAD` (F1 `09eff16`, F2 `92152df`) with
[hygiene review](../../../workflow/hygiene-review.md) as the checklist, and the
[F1](F1-report.md) and [F2](F2-report.md) reports and the [log](../log.md).
(main has moved since the branch point; diff against the merge base, not
`origin/main`, or you will see main's own changes reversed.) Pixi is not
installed; run commands directly.

In particular:
- **The no-`aS` rescale path.** `ScaleAwareAmplitude::set_alpha_s`
  (`helas/eval/rescale.rs`) now records α_s and leaves the pools alone when
  the model has no `aS`. Is that right for every model that reaches it (a
  model with no strong coupling has nothing to move), and is it pinned by a
  test that would fail if the early return were removed or inverted? Its doc
  says why it is not an error.
- **`reweight/engine.rs`**: four `expect`s with stated invariants. Is each
  invariant actually established upstream (cite the line), and is there one
  helper rather than four copies?
- **`refuse_conditional_definitions`**: one table of (file, constructor) used
  by both the parse and the refusal, or two lists that can drift?
- **`try_edited_toy_model`**: shared with the other toy tests, minimal.
- **Visibility**: `UnknownParameter`, `ParameterSet::defines`,
  `conditional_constructor_names`: narrowest that compiles.

Gate: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`
(and with `--all-features`), `cargo test --workspace`, `python3 scripts/kb.py lint`;
mutation evidence for each fix that adds or tightens a test. Write
`research/kb/sprints/ufo-conditional-params/sessions/HY1-report.md`
(findings table written before any edit, then dispositions; commands with
output; **Found**) and commit it.

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
