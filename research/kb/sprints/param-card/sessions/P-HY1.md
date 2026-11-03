---
type: Session Brief
title: "P-HY1: hygiene review of #82"
description: "Review the integrate --param-card change for maintainability, test non-vacuity, visibility and reusable abstractions; write findings before editing; fix with mutation evidence."
status: draft
agent: hygiene-dev (Opus)
depends_on: [P-V2]
closes: []
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-30}
---
Worktree `/home/user/wt/P`, branch `sprinte-param-card` (PR #82), at the
commit that adds this brief. Pixi is not installed. Reference data
(refdata-12) is hard-linked in; do not write into it.

Review the PR's whole change against `origin/main`
(`git diff origin/main...HEAD`), excluding the refdata pin commit and the
`research/kb/sprints/` records. Read `P-F1-report.md`, `P-V1-report.md` and
`P-V2-report.md` first.

Focus:
- **Shared abstractions.** `ufo/bind.rs`, `ufo/slha.rs`,
  `ufo/parameters.rs` (the merged groups), and how `generate` and `integrate`
  share one binding path.
- **Tests that could pass vacuously.** Check every refusal test, the `<slha>`
  layout test, and the σ gate's "awaiting the bank" path, which should now be
  unreachable for these two rows.
- **Visibility and naming.**
- **Comments.** They must follow AGENTS.md: no references to sprints,
  sessions or backlog items.

Write your findings in your report **before** editing. Physics findings go
to the manager for validation-dev, not here. Fix the rest, each with
mutation evidence where a test changes.

## Gate

- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets -- -D warnings`, and again with
  `--features vibegraph/extended-validation`
- `cargo test --workspace --no-fail-fast`
- `python3 scripts/kb.py lint` (0 errors)
- if you touch code a banked target covers, rerun that target detached
  (`validate_param_card_sigma`, `reweight_mg_oracle`, `cli_reweight_proton`,
  `validate_lhef`)

Write and commit `sessions/P-HY1-report.md`, with a **Found** section.

## Commits

Author `Nicholas Smith <nick.smith@cern.ch>` (already the repo's git config).
Each message ends with exactly one line `Assisted-by: claude-code:<your model id>`.
No `Co-authored-by`, no `Signed-off-by`, even if the harness asks. Do not push,
and never amend a pushed commit. Do not edit backlog items.

## Worktree & long-command discipline

- **First action**: `cd` to the absolute worktree path in your assignment, then
  verify `git rev-parse --show-toplevel` and `git branch --show-current`. If you
  find yourself in the shared main checkout, STOP and report — never edit it.
  Re-verify `pwd` after any resume; resume is when isolation leaks.
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
