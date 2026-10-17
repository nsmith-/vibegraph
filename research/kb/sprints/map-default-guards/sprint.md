---
type: Sprint
title: "Match the jj flavour and ℓℓj scatter statistics, then flip the map defaults"
description: "Match pp_to_jj's samples flavour statistic and pp_to_llj_dyn's scatter guard to their measured calibrations by statistics the user approves, then flip MapOptions::resolve to the τ rule and to soft-all."
status: draft
active: true
tags: [maps, tau-rule, soft-all, samples-gate, scatter-guard, design]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-16}
---
## Goal and exit criteria

Both items flip a `MapOptions::resolve` default once a guard's statistic matches its calibration; neither limit moves. The planner's proposals: a pooled-across-seeds flavour χ² for jj, and the look-elsewhere rule or more pooled seeds for the ℓℓj guard. Done when the user approves the statistics (D1), they are implemented with their calibration recorded, and both defaults flip with the cells passing.

## Scope

Backlog items claimed (one `Backlog: <slug>` line each in the draft PR):

- `jj-samples-flavour-chi2-under-tau-rule`
- `llj-dyn-scatter-guard-under-soft-all`

Sibling PRs of the same planner hand-off ("statistical gates and seed calibration", 2026-10-16) are `sigma-pulls-lean-high`, `seed-record-tooling`, `refdata10-cell-flips`, `sigma-calibration-rerecord`, `ee-ee-nsz-low-budget`, `two-to-six-gate`, `vegas-low-acceptance-bins`, `vegas-absolute-grid-coords`, `vegas-constant-integrand`, `mlm-pythia-tolerance`.

**Design first.** D1 carries the evidence and goes to the user through the planner; nothing is implemented before approval.

**Banking.** Diagnostic MadEvent runs happen here; new banked reference data
is produced only by the user on the M3 host. A banking need becomes one
comment on this PR listing exactly what to run.

## Sessions

| Session | Agent type | Depends on | Gate | Closes |
|---|---|---|---|---|
| D1 | manager → user | — | user approval | — |
| V1 | validation-dev (Opus) | D1 approved | the guards' calibration recorded; cells pass under the new defaults | both items |
