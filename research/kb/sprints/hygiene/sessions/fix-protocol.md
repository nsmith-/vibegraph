---
type: Procedure
title: "Hygiene fix protocol"
description: "How an F-session fixes one cluster's triaged findings and claimed items: what counts as done, the stop-rule, proving a tightened test, gates, commits and the report."
status: draft
tags: [hygiene, fix, protocol]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
---
Every F-session brief links here. The brief names the findings (ids into the
`R-<x>-report.md` reports) and the claimed backlog items; this page says how
to fix them.

## Read first

- your brief, then each finding in its review report, and each claimed item
  (`python3 scripts/kb.py backlog --item <slug>`);
- [triage.md](../triage.md) for how the finding was classed;
- [D3](../decisions/D3-fix-small-file-large.md);
- AGENTS.md, especially the comment guidelines and "Physics Validation";
- the `extended-validation` skill, before touching amplitudes, colour,
  couplings, diagrams, phase space, sampling, PDFs, scales or the LHEF writer.

## Rules

1. **Re-verify each finding before fixing it.** Findings are a reviewer's
   claim, made at `f7efda6`; lines may have moved. A finding that does not
   hold is reported as *not reproduced*, with the evidence, and left alone.
2. **Fix only what the brief lists.** New work goes to Found. A fix that turns
   out to need a new shared type, a cross-cluster change or an API change
   stops, and goes to Found as *should be filed*.
3. **Stop-rule for real bugs.** Fix one only when the fix is unambiguous, and
   pin it with a test that fails before the fix and passes after. Report both
   runs.
4. **Tightening a test is not inert** (D3). For each tightened or newly
   asserting test, show that:
   - it passes on the fixed code;
   - it fails on the finding's mutation (or an equivalent one-line defect),
     applied temporarily and reverted before the commit.

   Report the mutation and both results. A new bound comes from a recorded
   reading (census, manifest note, banked error) or a stated multiple of the
   run's own quoted error, never from a fresh fixed-seed number.
5. **Delete rather than gate** test-only code whose only purpose was the
   thing a finding removes. Never `#[ignore]` or weaken a test to get green.
6. **Comments:** describe the code as it is. No sprint, session, note or plan
   names; no history.
7. **Don't edit `research/kb/` or backlog items.** Kb and item sites that a
   fix makes stale go to Found, one entry each, for close-out. Book pages
   under `docs/` may be edited when a fix changes what they name.

## Gate

Run all of these on the final commit, each with the tail of its output in the
report:

- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace` (hermetic). The suite count may change only
  through tests you added or deleted; list them.
- `cargo doc --workspace --no-deps --document-private-items`, with no new
  warnings
- **the banked targets your changes touch:**
  `cargo test -p <crate> --profile release-debug --features extended-validation --test <target>`.
  Name each target and why. The reference bundle, PDF set and `mg5amcnlo`
  content are in the worktree. `pixi` is not installed; `bash validation/validate.sh`
  is the full banked run, and the manager runs it at close-out, not you.

## Commits

Commit at natural checkpoints, one finding or a small related group per
commit.
- The message says what changed.
- End it with `Assisted-by: claude-code:<model id>`.
- Never add `Co-authored-by` or `Signed-off-by`.
- Never run `git checkout -- .`, `git reset --hard` or `git clean` on
  uncommitted work.

## Report

The final message. Its sections:

- **Fixed:** finding id or item, commit, what changed, the test or mutation
  evidence.
- **Not reproduced:** with the evidence.
- **Stopped:** should be filed, and why.
- **Gate:** each command with the tail of its output.
- **Method:** what was quick, what was slow, and what the brief or protocol
  got wrong. This is lesson data.
- **Found** and **Brief corrections.**

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
