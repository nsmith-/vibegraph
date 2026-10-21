---
type: Session Brief
title: "HY1: hygiene review of the default-name table and --modelname"
description: "Review F1's diff for maintainability, test non-vacuity, visibility and reuse; fix what is local, with mutation evidence."
status: draft
agent: hygiene-dev (Sonnet)
depends_on: [F1]
closes: []
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-20}
---
Worktree `/home/user/wt/F`, branch `sprintd-default-particle-names`. Review
`git diff $(git merge-base origin/main HEAD)...HEAD` (F1, `63f50a6`) with
[hygiene review](../../../workflow/hygiene-review.md) as the checklist, and
[F1's report](F1-report.md) and the [log](../log.md). Pixi is not installed;
run commands directly.

In particular:
- **The table can go stale** (F1's Found 1): `ParticleNames` is built from the
  `pub` `UFOModel::particles`, and a test that inserted a particle had to
  rebuild it by hand. Find every writer of `particles` after load. If they
  are few and in-crate, make the rebuild impossible to forget (a constructor
  or a method that edits both, or a debug-build consistency check at the
  lookup); if making `particles` private is a wide change, list its users and
  stop at the check.
- **The `UFOModel` doc** (F1's Found 2) says maps are keyed by the Python
  variable name; particles are keyed by the UFO `name`. Fix the doc.
- **`ParticleNaming` / `load_with_digest_and_naming`**: one more loader entry
  point beside `load_with_digest`. Is a parameter on the existing one, or a
  builder, cleaner? Fix locally if cheap.
- **Tests**: `particle_names.rs` and the toy UFO `vibegraph_toy_names_UFO`
  minimal and sharing the toy helpers; the vendored table's sha256 pinned
  via `sha2` (not hand-written).

Gate: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`
(and with `--all-features`), `cargo test --workspace`, `python3 scripts/kb.py lint`;
mutation evidence for each fix that adds or tightens a test. Write and commit
`research/kb/sprints/default-particle-names/sessions/HY1-report.md` (findings
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
