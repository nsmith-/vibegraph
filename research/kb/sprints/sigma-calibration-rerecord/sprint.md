---
type: Sprint
title: "Re-record the σ seed calibrations from one current run"
description: "Re-record plan_for's seed figures, the census page, the σ gates' calibration comments and the ℓℓj ladder rationale from one current run, identify the drift's cause, and re-measure the two thin-headroom cells against refdata-10."
status: draft
active: true
tags: [sigma-gate, calibration, seed-sweep, budget-ladder, refdata]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-16}
---
## Goal and exit criteria

All four items ask for the same thing, figures quoted in `validate_sigma.rs`, `validate_hadronic.rs`, `plan_for` and the census page re-recorded from one current run, so they share one run and one review. Done when every quoted figure is a recorded measurement on one stated commit, the drift since the 2026-09 census is attributed (falsifier: `probe_gate_row_seed_headroom` at `f85718d^`), and `ddx_to_epemg` / `gux_to_epemux` are re-measured against refdata-10 with a budget decision brought to the user only for what remains.

## Scope

Backlog items claimed (one `Backlog: <slug>` line each in the draft PR):

- `sigma-seed-calibrations-drifted`
- `sigma-calibration-comments-stale`
- `llj-gate-comments-quote-pre-floor-ladders`
- `sigma-cells-thin-budget-headroom`

Sibling PRs of the same planner hand-off ("statistical gates and seed calibration", 2026-10-16) are `sigma-pulls-lean-high`, `seed-record-tooling`, `refdata10-cell-flips`, `ee-ee-nsz-low-budget`, `two-to-six-gate`, `vegas-low-acceptance-bins`, `vegas-absolute-grid-coords`, `vegas-constant-integrand`, `map-default-guards`, `mlm-pythia-tolerance`.

**Sequencing.** This PR re-records figures that every sampler or reader change moves, and it edits both σ test files. It runs after `sigma-pulls-lean-high` settles the reader and after `refdata10-cell-flips` lands; a later default change (`map-default-guards`) re-records its own rows.

**Banking.** Diagnostic MadEvent runs happen here; new banked reference data
is produced only by the user on the M3 host. A banking need becomes one
comment on this PR listing exactly what to run.

## Sessions

| Session | Agent type | Depends on | Gate | Closes |
|---|---|---|---|---|
| V1 | validation-dev (Opus) | sigma-pulls-lean-high D1, refdata10-cell-flips merged | one recorded run; every enforced cell unmoved | all four, budget decision aside |
| D1 | manager → user | V1 | user decision on thin budgets | `sigma-cells-thin-budget-headroom` |
