---
type: Session Brief
title: "G-V2: why VEGAS under-covers a 3 % slice, and making the stop see it"
description: "Locate the per-channel floor or allocation that leaves the both-W |cos θ| > 0.99 slice of u u~ > w+ b w- b~ / t t~ 3 % low at the default budget, and make the convergence stop detect it, or propose a runtime warning to the user."
status: draft
agent: validation-dev (Opus)
depends_on: []
closes: [vegas-under-covers-heavily-cut-slice]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-04}
---
Worktree `/home/user/wt/G2-V2`, branch `sprintg-diagnoses-v2`, off
`sprintg-diagnoses` (main 86b8e5c plus the sprint folder). The manager
merges it into `sprintg-diagnoses`.

## Read first

- `python3 scripts/kb.py backlog --item vegas-under-covers-heavily-cut-slice`.
- The V2 report of the sigma-uux-wbwb-veto sprint
  (`../../sigma-uux-wbwb-veto/sessions/V2-report.md`), §2 (the FW slice
  table: default neval −2.8 % at the banked card, five seeds χ²/dof 0.66,
  against `--neval 1e6 --target-rel 3e-3`) and the Found entry. Its probe
  (`VG_PROBE_FWD` in `Cuts::pass`) was reverted; rebuild it as an
  uncommitted, or test-only, filter.
- [Seed sweeps and budget ladders](../../../validation/seed-sweeps-and-budget-ladders.md)
  and AGENTS.md "Samplers gate statistically".
- Decision G5 in `../sprint.md`.

## Work

1. **Reproduce** the FW deficit on main at the banked card: ≥5 seeds at the
   default budget against the `--neval 1e6` reference (≥3 seeds), and the
   non-adaptive estimate (a flat or frozen-grid estimator with no
   adaptation, which cannot hide a region by learning it away).
2. **Locate the mechanism.** Per channel: points allocated, points passing
   the cut, the channel's σ share in the slice, against the reference. Is it
   the 512-point per-channel floor, the Neyman allocation starving a channel
   whose cut-passing variance is under-estimated early, grid adaptation
   collapsing bins, or iteration weighting (1/σ² combination rewarding
   confidently-low iterations)? Test each hypothesis with a run that would
   come out differently if it were false.
3. **Detection.** Design a check the convergence stop can apply cheaply (for
   example a per-channel effective-sample or acceptance-count floor, an
   inter-iteration consistency test that is not 1/σ²-weighted, or a
   leave-one-out over iterations). Show it fires on the FW slice at the
   default budget, and measure its false-positive rate across the banked σ
   rows' configurations (it must not fire, or must only raise the budget, on
   rows that converge today). If it can fire, the stop either raises the
   budget until it clears or reports the run unconverged.
4. **If detection is not reliable** within the session, STOP before writing
   production code and report a proposal for a runtime warning (decision
   G5): the indicator, its threshold, the measured hit and false-positive
   rates. The manager takes it to the user.
5. Banked σ gates must stay green (`validate_sigma`, `validate_hadronic`
   under `--profile release-debug`), and any change to sampling order is
   stated so the fixed-seed pins can be judged.

## Gate

- `cargo fmt --all --check`; `cargo clippy --workspace --all-targets -- -D warnings`,
  and again with `--features vibegraph/extended-validation`
- `cargo test --workspace`
- the `extended-validation` skill's rows for VEGAS and multichannel budgets
  (`validate-sigma`, `validate-hadronic`), run detached, one at a time
- `python3 scripts/kb.py lint` (0 errors)

## Environment

Pixi is not installed: run the commands each pixi task wraps directly
(`pixi.toml`). The reference data (`validation/madgraph/output`,
`validation/pdf`, `research/refs/mg5amcnlo`) are hard links to the main
checkout's copies: never write into, regenerate or delete them. 4 cores are
shared with one sibling session; run cargo with `--jobs 2` and integrations
with at most 2 threads, one heavy suite at a time.

## Commits

Author `Nicholas Smith <nick.smith@cern.ch>`. Each message ends with
exactly one line `Assisted-by: claude-code:<your model id>`. No
`Co-authored-by`, no `Signed-off-by`, even if the harness asks. Do not
push. Do not edit backlog items.

## Report

Commit `sessions/G-V2-report.md` (`type: Session Report`): every command
with its output, the seed tables (mean, spread, χ²/dof), the hypotheses
tested and how each was falsified or confirmed, what the detection cannot
see, and a **Found** section. Correct this brief where it is wrong.

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
