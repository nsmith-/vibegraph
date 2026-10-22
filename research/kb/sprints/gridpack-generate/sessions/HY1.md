---
type: Session Brief
title: "HY1: hygiene review of the gridpack-style generate"
description: "Review F1 and F2's diff for maintainability, test non-vacuity, visibility and reuse; fix what is local, with mutation evidence."
status: draft
agent: hygiene-dev (Opus)
depends_on: [F2]
closes: []
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-22}
---
Worktree `/home/user/wt/H1`, branch `sprintd-gridpack-generate`. Review
`git diff origin/main...HEAD` (F1 `723f6d1`, F2 `39a1a25`) with
[hygiene review](../../../workflow/hygiene-review.md) as the checklist,
together with [F1's report](F1-report.md), [F2's report](F2-report.md) and the
[log](../log.md). Pixi is not installed, so run commands directly.

Points to cover in particular:
- **The new library module `generate.rs` and its `source.rs` and `audit.rs`.**
  - What is public, and why?
  - Does the CLI still hold logic that belongs in the library?
  - Are the moved functions free of CLI vocabulary?
- **One place per rule.**
  - The stage table, `split`, `integration_seed`, and the `LHAPDF_DATA_PATH`
    / `LHAPATH` parse should each live in one place, with the design citation
    once.
  - F2 found the "absent run" escape duplicated elsewhere. Check the new code
    for the same pattern.
- **Error types.** `GenerateError` and `RunCardError`'s new variants: are the
  messages self-contained, and does each refusal name what the user should do?
- **Tests.**
  - Do the deleted tests leave a gap? Compare against the refusals the old
    `card_mismatches` covered.
  - Is `common::PdfDataPath` used everywhere a test needs a PDF set?
  - Does any test still depend on the developer's environment?
- **Comments.** They follow AGENTS.md: they describe what the code does now,
  and there are no "used to"/"no longer" notes and no references to sessions
  or stages.

Write the findings table before any edit, then fix what is local. Give
mutation evidence for each fix that adds or tightens a test. Physics findings
go to Found.

Out of scope: `vegas.rs`, `budget.rs`, the phase-space maps, and H2's PDF
work. Do not edit backlog items.

Gate:
- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets -- -D warnings`, also with
  `--features vibegraph/extended-validation`
- `cargo test --workspace`
- `python3 scripts/kb.py lint`

Write and commit `HY1-report.md`. It records every command with its output,
the findings table and its dispositions, and a **Found** section.

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
  exit: never spawn sleep-based watcher shells; draft your notes and report
  between completion notifications instead of idle-waiting. Never `pkill -f`;
  kill only PIDs you started.
- **`git status` can hang for minutes** in a worktree full of untracked build
  data; spot-check with `git log` / `git diff --stat` instead.
