---
type: Sprint
title: "Cells refdata-10 settles: ttx MLM @1 and gg_to_gg_cg"
description: "Measure pp_to_ttx_0j1j_mlm's integrals cell and gg_to_gg_cg's against refdata-10's seeded hard_survey = 2 references, and flip each to gate on a five-seed agreement, rewriting both manifest notes on the current code."
status: draft
active: true
tags: [sigma, mlm, ttx, smeftsim, info-cell, gate-promotion, refdata]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-16}
---
## Goal and exit criteria

Both cells were attributed to the reference side in sprint A and wait only on refdata-10. Done when each gates on a recorded ≥5-seed agreement with |pull| < 3 against the refdata-10 record, or the record says why not; neither tolerance moves.

## Scope

Backlog items claimed (one `Backlog: <slug>` line each in the draft PR):

- `ttx-mlm-at1-sigma-high`
- `gg-to-gg-cg-integrals-cell-still-info`

Sibling PRs of the same planner hand-off ("statistical gates and seed calibration", 2026-10-16) are `sigma-pulls-lean-high`, `seed-record-tooling`, `sigma-calibration-rerecord`, `ee-ee-nsz-low-budget`, `two-to-six-gate`, `vegas-low-acceptance-bins`, `vegas-absolute-grid-coords`, `vegas-constant-integrand`, `map-default-guards`, `mlm-pythia-tolerance`.


**Banking.** Diagnostic MadEvent runs happen here; new banked reference data
is produced only by the user on the M3 host. A banking need becomes one
comment on this PR listing exactly what to run.

## Sessions

| Session | Agent type | Depends on | Gate | Closes |
|---|---|---|---|---|
| V1 | validation-dev (Opus) | — | the two cells measured; `--skip-deps validate` unmoved | both items |
