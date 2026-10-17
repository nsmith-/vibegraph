---
type: Sprint
title: "σ pulls lean high on the seeded references"
description: "Re-read the 47 σ pulls against refdata-10 against each record's equal-weight mean, and attribute the population's +0.65 lean to the reference or to this side."
status: draft
active: false
tags: [sigma, madevent-reference, seeds, iteration-combination, refdata]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-16}
---
## Goal and exit criteria

The first banked run on refdata-10 passes every σ gate, but its 47 pulls average +0.65 with spread 0.87, about 5 standard errors from zero, in the direction of MadEvent's mean²/variance iteration combination. Done when the population mean is re-read against each record's equal-weight mean (no MadEvent run needed) and the lean is either quantified as reference-side bias or traced to this side. If it survives the re-read, the next oracle is a few-row adaptation-free estimate. This is the systematic item of the hand-off, so it goes first: if the gates' reader changes, every calibration re-recorded afterwards depends on it.

## Scope

Backlog items claimed (one `Backlog: <slug>` line each in the draft PR):

- `sigma-pulls-lean-high-on-seeded-refs`

Sibling PRs of the same planner hand-off ("statistical gates and seed calibration", 2026-10-16) are `seed-record-tooling`, `refdata10-cell-flips`, `sigma-calibration-rerecord`, `ee-ee-nsz-low-budget`, `two-to-six-gate`, `vegas-low-acceptance-bins`, `vegas-absolute-grid-coords`, `vegas-constant-integrand`, `map-default-guards`, `mlm-pythia-tolerance`.


**Banking.** Diagnostic MadEvent runs happen here; new banked reference data
is produced only by the user on the M3 host. A banking need becomes one
comment on this PR listing exactly what to run.

## Sessions

| Session | Agent type | Depends on | Gate | Closes |
|---|---|---|---|---|
| V1 | validation-dev (Opus) | — | diagnostic: population table, every cell a recorded measurement | — |
| HY1 | hygiene-dev (Sonnet) | V1 | output byte-identical; mutations | — |
| [D1](decisions/D1-fixed-beam-seeds-and-ew.md) | manager → user | V1 | user approval (2026-10-17) | — |
| F1 | validation-dev (Opus) | D1 | `--skip-deps validate`; one-seed mutation fails | — |
| V2 | validation-dev (Opus) | D1, F1 | adaptation-free estimate recorded | — |
