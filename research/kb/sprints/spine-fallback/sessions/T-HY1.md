---
type: Session Brief
title: "T-HY1: hygiene review of #81"
description: "Review the no-scale spine change (pole floor, remainder mixture, version 15 and the channel-map digest, the spine σ and EWK ℓℓjj gates) for maintainability, test non-vacuity, visibility and reusable abstractions; write findings before editing; fix with mutation evidence."
status: draft
agent: hygiene-dev (Opus)
depends_on: [T-V3]
closes: []
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-31}
---
Worktree `/home/user/wt/T`, branch `sprinte-spine-fallback` (PR #81), at the
commit that adds this brief. Pixi is not installed. Reference data is hard
linked in; don't write into it.

Review the PR's whole change against `origin/main` with
`git diff origin/main...HEAD`. Leave out `research/kb/sprints/` and the
backlog items. Read the T-F1, T-V1, T-V2, T-F2 and T-V3 reports first.

Priorities:
- `vibegraph-lib/src/phasespace/diagram_channel.rs`: the pole floor, the
  remainder mixture, `map_key`, and the rung-order fix in `d54f0e46`;
- the artifact's version 15 and `channel_map_digest`, with their
  integrate/generate checks;
- `generate`'s overweight dump from `7263d4c8`;
- the tests: `diagram_channel.rs`, `spine_sigma.rs`,
  `validate_ewk_lljj_sigma.rs`, and the digest refusal tests;
- `bank_seeded_sigma.sh`'s new `card` kind, `DRY_RUN` mode and
  `@ewk-lljj` group.

Look for:
- duplication;
- tests that could pass vacuously (each gate must be able to fail; check the
  spread bound, the digest refusal and the "awaiting the bank" path);
- visibility wider than its callers need;
- comments that break AGENTS.md's rules (no sprint, session or item
  references, no history);
- functions that need splitting.

T-F2 listed Found items. Leave them alone, except where one is plainly a
hygiene defect in this PR's own code.

Write the findings in your report **before** editing. Physics findings go to
the manager, not into fixes. Fix the rest, with mutation evidence wherever a
test changes. Show that a refactor which changes no test leaves its outputs
byte-for-byte unchanged: dump them before and after, then remove the probe.

## Gate

- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets -- -D warnings`, and again with
  `--features vibegraph/extended-validation` and with
  `-p vibegraph-lib --features extended-validation`
- `cargo test --workspace --no-fail-fast`
- `python3 scripts/kb.py lint` (0 errors)
- if you touch code a banked target covers, re-run those targets detached
  (`spine_sigma`, `validate_sigma`, the decay-chain and on-shell veto rows).

Build with debuginfo off and `-j 2`. Write and commit
`sessions/T-HY1-report.md`, with a **Found** section.

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
