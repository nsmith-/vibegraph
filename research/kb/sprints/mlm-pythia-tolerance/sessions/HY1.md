---
type: Session Brief
title: "HY1: hygiene review of the MLM Pythia gate"
description: "Review V1's driver, tests and collator change for maintainability, test non-vacuity, visibility and reuse; fix what is local, with mutation evidence; record the Pythia build in the row."
status: draft
agent: hygiene-dev (Sonnet)
depends_on: [V1]
closes: []
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-17}
---
Review `git diff origin/main...HEAD` (V1, `df3bbf9`) with
[hygiene review](../../../workflow/hygiene-review.md) as the checklist,
[D1 with its amendment](../decisions/D1-tolerance.md) as the binding rule,
and [V1's report](V1-report.md).

In particular:
- **The rule against D1.** Check the five pulls at < 3, the χ² leg (min p × 3
  against α = 0.5 %), the per-bin leg (two-sided p_min × N_bins against
  α = 0.5 %, z* from the actual binning), and the null calibration (a failing
  MadEvent null makes the jet rates `info` and rescales nothing). Ask which
  one-line mutations of `mlm_match.py`'s verdict logic the 16 tests are blind
  to (α per leg, the trial factors 3 and N_bins, one- vs two-sided, `<` vs
  `≤`, the null's split), and show mutation evidence.
- **The collator.** Look at `mlm_pythia_verdict` and the new optional
  `[[standalone]] mode` with `check_standalone_modes`: test power, and
  whether a standalone row that omits `mode` stays as it was.
- **Two small, local Found items from V1 to fix here:**
  - the row should record which Pythia build and platform ran
    (`pythia8-config --version` or equivalent, and the platform), because
    results differ across builds;
  - the version prints as `8.3120e+00` (`mlm_match.cc` prints a float).
  Print it as the version string.
- Duplication between `mlm_match.py`'s statistics and anything already in
  `validation/pythia/` or the Rust stats module.

Write findings first (in the report), then fix the local ones; file the rest
under **Found**. Gate: `pixi run -e pythia --skip-deps test-mlm-pythia`,
`pixi run -e pythia --skip-deps validate-mlm-pythia` (reads the cache, about
70 s; its verdict and every printed number must be unchanged apart from the
added build field), `cargo test -p vibegraph-validation-report`,
`cargo fmt --all --check`, and `cargo clippy --workspace --all-targets -- -D
warnings`.

## Report

`sessions/HY1-report.md`, **Found**.

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
  re-running it. Anything that may exceed ~10 minutes runs detached.
- **`git status` can hang for minutes** in a worktree full of untracked build
  data; spot-check with `git log` / `git diff --stat` instead.
- **Run gates with `--skip-deps`** unless the assignment explicitly says
  reference data must be regenerated — without the reference data present, a
  bare `pixi run validate` silently launches a multi-hour MG regeneration.
  `pixi` is at `~/.pixi/bin/pixi` (add `~/.pixi/bin` to PATH).
- **Disk is finite, and a full disk looks like a failure.** Run cargo with
  `CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0`.
  An `ENOSPC`, a linker `Bus error` or a truncated log is a kill, not a result:
  free only your own `target/` and re-run. Kill processes by PID, never
  `pkill -f`.
- **This host** is a 4-core cloud container with ~28 GB free disk, shared with
  one sibling session. One heavy suite or MadEvent batch at a time from your
  session; kill only PIDs from your own process tree. Delete MadEvent scratch
  directories you no longer need.

## Banking rule (binding)

Diagnostic MadGraph/MadEvent runs (`pixi run -e madgraph ...`) are allowed:
throwaway, never committed or banked, run in the scratchpad, with the pinned
MadGraph 3.7.1 (`validation/madgraph/mg5_pinned.sh`), never the packaged
`mg5_aMC` on PATH. **New banked reference data** (re-banked rows, new seeds or
budgets, 20-seed confirmations, ladder runs on MadEvent) is produced only by
the user on the M3 host. If your conclusion needs it, write the exact request in
your report (command, rows, seeds, budgets, expected outputs); the manager
posts it. Never regenerate reference data through MadGraph.

## Boundaries

- A separate session owns `validation/madgraph/repro/` and the knowledge-base
  pages about MadGraph defects. Do not touch them.
- Never lower a floor, raise a limit or widen a tolerance to get green
  (`research/kb/workflow/expose-dont-fix.md`). `AGENTS.md` "Samplers gate
  statistically" is binding: ≥5 seeds, unweighted mean, read spread and
  χ²/dof; 20+ seeds on any rung-to-rung comparison. A fixed-seed pull is not
  evidence.
- You write only your report under `research/kb/` (as
  `sessions/<id>-report.md` in this PR's sprint folder, `generated.by` set to
  yourself, `sources` your commit hashes). New work goes in its **Found**
  section; never edit backlog items.
- Commits: end with `Assisted-by: claude-code:<your real model id>`; never a
  `Co-Authored-By` or `Signed-off-by` for a model. The worktree's git identity
  is already the human owner's; do not change it. Do not push; the manager
  pushes.

Briefs carry errors: if this one is wrong on any point, say so in the report.
Your final message is the report itself, with every claimed number next to the
command that produced it.
