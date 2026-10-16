---
type: Sprint
title: "Constant integrands integrate once"
description: "A single-channel two-body (constant-integrand) run integrates once without grid refinement and reports the exact width, pinned by a test."
status: draft
active: true
tags: [vegas, decay-width, two-body, performance]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-16}
---
## Goal and exit criteria

Done when the item's closes_when holds: the run reports the exact width and a test pins it, with every other seeded integration unmoved.

## Scope

Backlog items claimed (one `Backlog: <slug>` line each in the draft PR):

- `vegas-refinement-noises-constant-integrand`

Sibling PRs of the same planner hand-off ("statistical gates and seed calibration", 2026-10-16) are `sigma-pulls-lean-high`, `seed-record-tooling`, `refdata10-cell-flips`, `sigma-calibration-rerecord`, `ee-ee-nsz-low-budget`, `two-to-six-gate`, `vegas-low-acceptance-bins`, `vegas-absolute-grid-coords`, `map-default-guards`, `mlm-pythia-tolerance`.


**Banking.** Diagnostic MadEvent runs happen here; new banked reference data
is produced only by the user on the M3 host. A banking need becomes one
comment on this PR listing exactly what to run.

## Sessions

| Session | Agent type | Depends on | Gate | Closes |
|---|---|---|---|---|
| P1 | performance-dev (Sonnet) | — | hermetic + `--skip-deps validate` unmoved | the item |
| HY1 | hygiene-dev (Sonnet) | P1 | hermetic | — |
