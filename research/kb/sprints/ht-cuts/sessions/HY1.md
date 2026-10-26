---
type: Session Brief
title: "HY1: hygiene review of the HT cuts and their gates"
description: "Review the PR's diff for maintainability, test non-vacuity, visibility and reuse; fix what is local, with mutation evidence."
status: draft
agent: hygiene-dev (Opus)
depends_on: [F1, V1]
closes: []
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-23}
---
Worktree `/home/user/wt/K`, branch `sprintd-ht-cuts`, at `7daaab9` (pushed).
Review `git diff origin/main...HEAD` (F1 `d4731f0`, V1 `4cb654d`, the M3's
`ed5d8ea`) with [hygiene review](../../../workflow/hygiene-review.md) as the
checklist, and the F1 and V1 reports beside this brief. Pixi is not installed;
run commands directly. Do not edit `vegas.rs`, `budget.rs` or the phase-space
map code; the backlog item is already deleted, and you edit no backlog item.

In particular:
- **The HT cut definitions.** Which legs count for `htj` (jet class:
  |pdg| ≤ maxjetflavor, plus gluons) and for `iht` (htj plus the b class)
  should be decided in one place that both cuts and any other jet-class
  consumer share, not re-derived. Is the default-off rule (`htjmax = -1`,
  `ihtmax = -1`) expressed once? Do the docs say the cuts are not gated by
  `do_cuts`, as `cuts.f` does it, and does a test pin that?
- **Test non-vacuity.** For each unit test of the cuts, would it fail if the
  b class were counted in `htj`, or if `maxjetflavor` were ignored?
- **The two gates in `validate_hadronic.rs`.** Duplication with
  `measure_recarded_sigma` and the neighbouring σ gates. The planned-row
  spellings are a filed follow-up; leave them unless they are local to this
  PR's code.
- **The planned-row port.** The `manifest.rs` and collator hunks are
  byte-identical copies of the ones on #67 and #72, and must stay identical so
  that those PRs merge cleanly. Do not edit them.
- **Docs and concepts.** Do the run-card field classification, the
  user-facing docs and the knowledge-bundle concepts describe the four fields
  as compiled now, with no remaining refusal text?

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
