---
type: Session Brief
title: "HY1: hygiene pass on the permuted-first-call census script"
description: "Review and fix validation/madgraph/census_permuted_first_call.py for maintainability, non-vacuity and fit with the existing validation/madgraph tooling."
status: draft
agent: hygiene-dev (Sonnet)
depends_on: [V1]
closes: []
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-15}
---
The change under review is commit `ef002da`'s new
`validation/madgraph/census_permuted_first_call.py`. [V1's report](V1-report.md)
says what it is for and how it was run.

## Scope

- Follow `hygiene-dev`'s protocol: write the findings first, then fix the local
  ones.
- Check whether the script duplicates helpers that already exist under
  `validation/madgraph/` (banner and run-card parsing, process-directory
  walking) and reuse them where they exist.
- Check that its `perms` mode cannot pass vacuously. It must fail loudly on a
  directory with no `symperms.inc`/`configs.inc`, and not report "identity".
  Check that its docstring states the exposure criterion V1 corrected: the
  first call clusters when `ickkw > 0`, or `xqcut > 0`, or
  `dynamical_scale_choice = -1` without all three scales fixed, and σ can move
  only with PDFs or an α_s coupling.
- Hermetic only. You may exercise the script on directories under
  `validation/madgraph/output` (refdata-9 is present) and on a synthetic
  fixture. Run no MadGraph or MadEvent.
- Do not touch `research/kb/` beyond your report.

## Gate

`python3 -I validation/madgraph/census_permuted_first_call.py runs` and `perms`
run on the present refdata, with the output matching V1's table. The repo's
Python lint, if one is configured (`pixi.toml`, CI). `pixi run --frozen
kb-lint`.

Briefs carry errors: if this one is wrong on any point, say so in the report.

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

- **This host** is a 4-core cloud container with ~28 GB free disk, shared with
  sibling sessions. One heavy suite or MadEvent batch at a time from your
  session; kill only PIDs from your own process tree. Delete MadEvent
  scratch directories you no longer need.

## Banking rule (binding)

Diagnostic MadGraph/MadEvent runs (`pixi run -e madgraph ...`) are allowed and
encouraged: throwaway, never committed or banked, run in the scratchpad.
They use the pinned MadGraph 3.7.1 (`validation/madgraph/mg5_pinned.sh` under
`pixi run -e madgraph`, `VG_MG5_ROOT` for a patched copy), never the packaged
3.5.7 `mg5_aMC` on PATH, and LHAPDF sets
([MadGraph pinning](../../../validation/madgraph-oracle-pinning.md)).
**New banked reference data** (anything under the refdata bundle: MadEvent σ
or event references, re-banked rows, new seeds/budgets for banked rows) is
produced only by the user on the M3 host. If your conclusion needs it, write
the exact request in your report (task or script, rows, seeds, budgets,
patches, expected outputs); the manager posts it. MadGraph *standalone*
amplitude tables under `validation/madgraph/standalone/` are exempt.

## Boundary

A parallel sprint owns `vibegraph-lib/src/helas/` (especially `helas/eval`)
and the standalone-amplitude tests. Do not modify them. If your diagnosis
points into them, stop and report.
