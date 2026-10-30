---
type: Session Brief
title: "S-HY1: hygiene review of #83"
description: "Review the systematics change for maintainability, test non-vacuity, visibility and reusable abstractions; write findings before editing; fix with mutation evidence."
status: draft
agent: hygiene-dev (Opus)
depends_on: [S-V2]
closes: []
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-30}
---
Worktree `/home/user/wt/S`, branch `sprinte-systematics` (PR #83), at the
commit that adds this brief. Pixi is not installed. refdata-12 is hard-linked
in: never write into it. Review the PR's whole change against `origin/main`
(`git diff origin/main...HEAD`). Leave out `research/kb/sprints/` and the
refdata pin. Read the S-F1, S-F2, S-V1 and S-V2 reports and the design concept
`events/systematics-weights.md` first.

The change is large, so prioritise:
- `vibegraph-lib/src/systematics/` (request, weight, header, record);
- the run-card dispatch in `runcard.rs` and `runcard/classes.rs`;
- the PDF `.info` and member-header cascade in `pdf/grid.rs`;
- the LHEF writer changes;
- the test layer: `validate_systematics.rs`, `validate_mlm_records.rs`,
  `validate_mlm_dumps.rs`, `cli_systematics.rs`.

Check for:
- **Duplication.** Formatters, parsers and record builders written more than
  once.
- **Vacuous tests.** Each gate must be able to fail. Check the pinned counts
  (219/31/19 and the `--alps` counts) and the controls.
- **Visibility.** Nothing more public than its callers need.
- **Comments.** Follow AGENTS.md: no sprint, session or backlog references,
  and no history narration.
- **Size.** Functions that need splitting.

Write the findings into your report **before** editing. Physics findings go
to the manager, not into fixes. Fix the rest, with mutation evidence where a
test changes. A refactor that changes no test must be shown byte-unchanged
on its outputs: dump them before and after, then remove the probe.

## Gate

- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets -- -D warnings`, and again with
  `--features vibegraph/extended-validation`
- `cargo test --workspace --no-fail-fast`
- `python3 scripts/kb.py lint` (0 errors)
- If you touch code a banked target covers, re-run those targets detached:
  `validate_systematics`, `validate_mlm_records`, `cli_systematics`,
  `validate_lhef`, `validate_param_card_sigma`.

Build with debuginfo off and `-j 2`. Write and commit `sessions/S-HY1-report.md`,
with a **Found** section.

## Commits

Author `Nicholas Smith <nick.smith@cern.ch>` (already the repo's git config).
Each message ends with exactly one line `Assisted-by: claude-code:<your model id>`.
No `Co-authored-by`, no `Signed-off-by`, even if the harness asks. Do not push,
and never amend a pushed commit. Do not edit backlog items. Commit
checkpoints as you go.

## Worktree & long-command discipline

- **First action**: `cd` to the absolute worktree path in your assignment, then
  verify `git rev-parse --show-toplevel` and `git branch --show-current`. If you
  find yourself in the shared main checkout, STOP and report — never edit it.
  Re-verify `pwd` after any resume; resume is when isolation leaks.
- **Use absolute paths in every command**, and never background a
  `cd … && …` chain.
- **Background anything expected to exceed ~2 minutes**
  (`run_in_background: true`, output redirected to a log file prefixed with your
  worktree/session name — parallel siblings share one scratchpad and have
  clobbered each other's unprefixed logs). A backgrounded command notifies on
  exit: never spawn sleep-based or `pgrep`-based watcher shells; draft your
  notes and report between completion notifications instead of idle-waiting.
  Never `pkill -f`; kill only PIDs you started.
- **No foreground command over ~8 minutes.** Detach longer runs.
- **`git status` can hang for minutes** in a worktree full of untracked build
  data; spot-check with `git log` / `git diff --stat` instead.
