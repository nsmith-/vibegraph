---
type: Session Brief
title: "HY1: hygiene review of the ckm_no_b_mass restriction and its banking plumbing"
description: "Review F1 and V1's diff for maintainability, test non-vacuity, visibility and reuse; fix what is local, with mutation evidence."
status: draft
agent: hygiene-dev (Sonnet)
depends_on: [V1]
closes: []
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-20}
---
Worktree `/home/user/wt/G`, branch `sprintd-ckm-no-b-mass`. Review
`git diff origin/main...HEAD` (F1 `9dc76b4`, V1 `768a207`) with
[hygiene review](../../../workflow/hygiene-review.md) as the checklist, and
[F1's report](F1-report.md), [V1's report](V1-report.md) and the
[log](../log.md). Pixi is not installed; run commands directly.

In particular:
- **`SMRestrict::ships_with_madgraph()`**: one predicate, or will the next
  vendored card need a second? Is the list of restrictions still derived from
  `ALL` everywhere (CLI message, digest caches, tests)?
- **Two new test files** (`sm_ckm_no_b_mass.rs`, 9 tests; `interned_sm_rows.rs`, 4):
  shared setup duplicated between them or with `coupling_oracle`/`smeftsim`?
  Does each test's doc state what it can and cannot see (V1 wrote a blind-spot
  table; the code should carry the short form)? Thresholds (1e-13, 5e-7, 1e-20)
  named and justified once?
- **`tests/common`**: `load_row_model` became public alongside
  `model_for_row`; is there now one way to get a row's model, or two?
- **Python** (`gen_couplings.py`): the `VG_MG5_ROOT` read matches
  `mg5_pinned.sh`'s semantics exactly (unset and empty).

Gate: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`
(and with `--features vibegraph-lib/extended-validation`), `cargo test --workspace`,
`python3 scripts/kb.py lint`; mutation evidence for each fix that adds or
tightens a test. Write `research/kb/sprints/ckm-no-b-mass/sessions/HY1-report.md`
(findings table written before any edit, then dispositions; commands with
output; **Found**).

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
