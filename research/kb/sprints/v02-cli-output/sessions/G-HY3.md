---
type: Session Brief
title: "G-HY3: integrate summary lines that mislead (σ digits, χ²/dof over one iteration, μF for a dynamical scale)"
description: "integrate prints σ with significant figures, χ²/dof as n/a below two entering iterations, and the scale choice instead of μF = 0 GeV; the artifact records 'undefined' and 'no constant scale' distinctly from 0; the survey parser reads the new lines."
status: draft
agent: hygiene-dev (Opus)
depends_on: []
closes: [sigma-printed-to-fixed-decimals, iteration-chi2-printed-as-zero-with-one-result, dynamical-scale-summary-prints-mu-f-zero]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-04}
---
Worktree `/home/user/wt/G3-HY3`, branch `sprintg-cli-output-hy3`, off
`sprintg-cli-output`, which is stacked on `sprintg-refusals` (#88: the
σ = 0 and effective-sample-size refusals, artifact format 18) and main at
fa10e2d. The manager merges it back.

## Read first

- `python3 scripts/kb.py backlog --item <slug>` for each of the three items:
  their `closes_when` are the spec.
- `vibegraph-cli/src/integrate.rs`: the result line (σ `{:.6}`), the
  per-multiplicity summary, the `VEGAS: … (χ²/dof = {:.3})` line,
  `recorded_mu_f` and the `√s: … μF = {} GeV` line; and where
  `result.chi2_per_dof` gets its zero for one entering iteration.
- `vibegraph-lib/src/artifact.rs`: the format version rules. The branch is at
  18; a shape change bumps to 19 and refuses 18, with a test built on the old
  layout as G-F5 did for 17.
- `validation/cms_gridpacks/` (`vgrun.py`, `parse.py`, `test_parse.py`):
  the survey database parses these lines (#71). Update the parser and its
  tests in the same change; old campaigns' recorded output must still parse.
- AGENTS.md: "ULP exactness is never the target"; pick a σ format that keeps
  the digits the error supports, not more.

## Work

1. σ and its error print to significant figures in the result line and the
   per-multiplicity summary. A CLI test pins a σ of order 1e-5 pb printed in
   full.
2. χ²/dof prints n/a when fewer than two iterations enter the result (a
   `--fixed-budget --niter 3` CLI test), and the artifact or report field
   records "undefined" distinctly from zero (`Option<f64>` or equivalent).
   Check the validation report collator for the same zero.
3. With no constant factorization scale (per-event, or differing between
   beams) the summary names the scale choice (e.g. "μF dynamical, choice
   -1"), and the artifact field records the absence distinctly from 0. A CLI
   test pins both.
4. Everything reading the changed artifact fields (`generate`, reweighting,
   the report collator) handles the new states.

Write findings before editing, per your definition.

## Gate

- `cargo fmt --all --check`; `cargo clippy --workspace --all-targets -- -D warnings`,
  and again with `--features vibegraph/extended-validation`
- `cargo test --workspace`, and the banked CLI targets that read the summary
  or the artifact (`cli_integrate`, `cli_generate_proton`) under
  `--features extended-validation`
- `python3 -m pytest validation/cms_gridpacks` (or how its tests run)
- `python3 scripts/kb.py lint` (0 errors)

## Environment

Pixi is not installed: run the commands each pixi task wraps directly. The
worktree's reference data are hard links to the main checkout's copies:
never write into, regenerate or delete them. cargo `--jobs 2`. The
container has restarted twice: detach long runs and commit at checkpoints.

## Commits

Author `Nicholas Smith <nick.smith@cern.ch>`. Each message ends with
exactly one line `Assisted-by: claude-code:<your model id>`. No
`Co-authored-by`, no `Signed-off-by`, even if the harness asks. Do not
push. Do not edit backlog items.

## Report

Commit `sessions/G-HY3-report.md`: findings, fixes with mutation evidence,
every gate command with its final output, and a **Found** section.

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
