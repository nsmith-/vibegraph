---
type: Session Brief
title: "T-F1: build the peripheral spine without a single-leg pT cut"
description: "Separate the pole floor (always on, from a fixed default scale) from the support bound (only from a provable pT floor), build the spine for every final state, and measure bias and variance before any σ gate."
status: draft
agent: feature-dev (Opus)
depends_on: []
closes: []
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-28}
---
Worktree `/home/user/wt/T`, branch `sprinte-spine-fallback`, at the commit
that adds this brief (main 44bf09c9 plus the sprint folder). Pixi is not
installed: run the commands it wraps directly (`pixi.toml` lists them). The
reference data is already in the worktree as hard links: the banked MadGraph
runs (`validation/madgraph/output`), the PDF sets in `validation/pdf`, and the
`research/refs/mg5amcnlo` submodule content. Do not commit any of it, and do
not write into it.

## Read first

- The item, rewritten in #80 (not yet on main):
  `git show origin/claude/fervent-mayer-2hv310:research/kb/backlog/feature/three-body-spine-requires-fiducial-scale.md`.
- The evidence: PR #71 comment 6438701885, summarised in the item. On the
  three CMS cards (`ptj = ptl = 0`, an `mmjj` cut only) every run builds
  0 peripheral channels, and the 5-seed median σ climbs from 0.22–0.62 to
  0.62–0.81 of MadGraph between 10k × 3 and 160k × 3, with seed χ²/dof 3–92.
  The control `ewkzlljj_5f_LO` (`ptj = 10`) builds 56 of 68 peripheral
  channels and reads 1.005.
- `vibegraph-lib/src/phasespace/diagram_channel.rs`: the module docs on
  regulating the spacelike pole (lines 55–85) and `from_diagram_with` (~400).
- `vibegraph-lib/src/cuts.rs`: `spacelike_floor` and its tests.
- `vibegraph-lib/tests/diagram_channel.rs`: `WALK_DENSITY_TOL` and the walk
  against density checks.
- AGENTS.md, "Physics Validation", in full.

## The decision (user, 2026-10-28)

`fiducial_scale` does two jobs today, and a scale of zero turns both off:

1. **The support bound** `t ≤ −scale`. It narrows the channel's support and
   is unbiased only while the cuts provably reject `|t| < scale`. A single-leg
   pT cut does; `mmjj` does not.
2. **The pole floor**, `POLE_FRACTION_OF_FIDUCIAL_SCALE · scale` under the
   propagator pole. It moves only the sampling density, since `draw_t` and
   `t_measure` read the same value, so it is unbiased for any non-negative
   value. It is also what keeps the draw away from the collinear edge's
   rounding noise, the original bias.

Implement:

- **Separate them.** The support bound stays exactly as now, from
  `Cuts::spacelike_floor()` only, and is off when that is zero.
- **The pole floor is always on.** When the cuts give no scale, it comes from
  a **fixed default scale** (T1), a named constant with its rationale in a
  doc comment. Choose the value by measured variance (below), not by
  assertion. Do not derive it from `mmjj` or any other cut.
- **Build the spine for every final state** (T2), so a final state of more
  than two legs no longer falls back to the all-timelike tree when the scale
  is zero. Update the module docs, `from_diagram_with`'s doc,
  `spacelike_floor`'s doc and its test that says a zero floor leaves the
  channel unbuilt, and the `use_multichannel` doc in `hadronic.rs`. Describe
  the code as it now is.
- If the bias check below fails on composite-emitted-subsystem ladders, do not
  paper over it. Stop and report the measurement. The fallback (T2) is "build
  unless the emitted side is composite", and that choice goes back to the
  manager.

## What must be measured (bias first, then variance)

- **Bias at the density level.** Walk weight against `Channel::density` on
  peripheral diagrams with no scale, within `WALK_DENSITY_TOL` and with that
  tolerance unchanged. Include composite emitted subsystems (a ladder whose
  emitted side is a pair), massless exchanged lines, and points near the
  collinear edge. Run it over enough points that a rounding-scale edge flip
  would show, and say how many.
- **Bias at the σ level.** On a finite peripheral process with no single-leg
  pT cut, regulated only by invariant-mass cuts as the CMS cards are (for
  example a fixed-flavour `u c > e+ e- u c`-type EWK subprocess with `mmjj`
  and `mmll`), compare against an estimator that cannot share the bias: flat
  RAMBO, or the all-timelike tree at a budget where it has converged. Use
  ≥5 seeds per side, combine by unweighted mean, and report the spread and
  χ²/dof, not one pull (AGENTS.md).
- **Variance.** Per-point variance and σ spread with the spine, against the
  all-timelike tree, at two or three candidate default scales. This chooses
  the constant.
- **A known-wrong informational row.** Keep today's no-spine behaviour
  reachable in the measurement (a test-only switch or the parent commit) so
  the before/after numbers come from the same harness.
- **What else moves.** List every banked row whose channel set changes
  (a final state of more than two legs with a zero `spacelike_floor`, e+e-
  four-fermion rows included). Invoke the `extended-validation` skill to pick
  the gates those rows feed. Run them under the seed policy, detached, and
  report each before/after. A fixed-seed σ that moves is expected, not
  evidence either way.

The 5-seed ladder on the three CMS cards is T-V1's, not yours.

## Gate

- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets -- -D warnings`, and again with
  `--features vibegraph/extended-validation`
- `cargo test --workspace`
- `python3 scripts/kb.py lint` (0 errors)
- the gates the `extended-validation` skill names for a phase-space change,
  run detached

Each changed rule gets a test with mutation evidence: show that the test fails
when the spine is withheld again, and when the pole floor is turned off with no
scale. Write and commit `sessions/T-F1-report.md`: every command with its
output, the measurements as tables, the mutations, and a **Found** section.
Correct this brief where it is wrong, and say so in the report.

## Commits

Author `Nicholas Smith <nick.smith@cern.ch>` (already the repo's git config).
Each message ends with exactly one line `Assisted-by: claude-code:<your model id>`.
No `Co-authored-by`, no `Signed-off-by`, even if the harness asks. Do not push.
Do not edit backlog items.

## Worktree & long-command discipline

- **First action**: `cd` to the absolute worktree path in your assignment, then
  verify `git rev-parse --show-toplevel` and `git branch --show-current`. If you
  find yourself in the shared main checkout, STOP and report — never edit it.
  Re-verify `pwd` after any resume; resume is when isolation leaks.
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
