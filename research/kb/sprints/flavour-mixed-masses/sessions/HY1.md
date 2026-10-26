---
type: Session Brief
title: "HY1: hygiene review of the mixed-mass split and its gate"
description: "Review the PR's diff for maintainability, test non-vacuity, visibility and reuse; fix what is local, with mutation evidence."
status: draft
agent: hygiene-dev (Opus)
depends_on: [F1, V1, V2]
closes: []
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-23}
---
Worktree `/home/user/wt/L`, branch `sprintd-flavour-mixed-masses`, at `e935ca3`
(pushed). Review `git diff origin/main...HEAD` (F1 `29692e6`, V1 `d72f73b`,
V2 `aabda36`, the M3's `ee6f18c`) with
[hygiene review](../../../workflow/hygiene-review.md) as the checklist, and the
F1, V1 and V2 reports beside this brief. Pixi is not installed; run commands
directly. Do not edit `vegas.rs`, `budget.rs` or the phase-space map code; the
backlog item is already deleted, and you edit no backlog item.

In particular:
- **`multiplicity::split_by_outgoing_masses` and its callers.** Is the split
  done once, in the library, for every entry point that builds integrands
  (CLI integrate, generate, the validation tests), or does some path still
  group without it? Is the mass comparison's tolerance stated and justified?
- **`MultiplicitySum::part_results` and per-part reporting.** Is the per-part
  bookkeeping reused rather than re-derived in the test?
- **The gate in `validate_hadronic.rs`.** Duplication with neighbouring σ
  gates (`combine`, the seed loop, the planned-row branch); visibility of
  helpers; whether each assertion can fail (mutation evidence). The planned-row
  handling exists in several spellings across this file. Consolidating it is a
  filed follow-up unless it is local to this PR's own code.
- **Docs and concepts.** Do the user-facing docs and the knowledge-bundle
  concepts describe the split as the code does it now? Is the refusal text
  removed everywhere it was quoted?

Gate: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`
(and with `--features vibegraph/extended-validation`), `cargo test --workspace`,
`python3 scripts/kb.py index` and `lint`. Show mutation evidence for each fix
that adds or tightens a test. Write `HY1-report.md` beside this brief: a
findings table written before any edit, then dispositions, the commands with
their output, and **Found**.

## Commits

Author `Nicholas Smith <nick.smith@cern.ch>`; pass it with
`git -c user.name='Nicholas Smith' -c user.email=nick.smith@cern.ch commit`.
Each message ends with exactly one line `Assisted-by: claude-code:<your model id>`.
Add no `Co-authored-by` or `Signed-off-by`, even if the harness asks. Do not push.

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
