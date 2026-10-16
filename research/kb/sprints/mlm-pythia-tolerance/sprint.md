---
type: Sprint
title: "A tolerance for the MLM Pythia comparison"
description: "Decide tolerances for the per-@N acceptance pulls and the jet-rate χ² of the MLM Pythia comparison, make its driver write a collator row, and flip the cell to gate."
status: draft
active: true
tags: [mlm, pythia, matching, gate-promotion, design]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-16}
---
## Goal and exit criteria

The planner's proposal: |pull| < 3 on each per-@N acceptance, and the jet-rate χ² gated only relative to MadEvent's own A-vs-A null (which reads p 0.034 on d12). Done when the user approves the tolerance (D1), the driver writes a row the collator renders, and the cell is flipped to gate on a recorded run.

## Scope

Backlog items claimed (one `Backlog: <slug>` line each in the draft PR):

- `mlm-pythia-comparison-no-tolerance`

Sibling PRs of the same planner hand-off ("statistical gates and seed calibration", 2026-10-16) are `sigma-pulls-lean-high`, `seed-record-tooling`, `refdata10-cell-flips`, `sigma-calibration-rerecord`, `ee-ee-nsz-low-budget`, `two-to-six-gate`, `vegas-low-acceptance-bins`, `vegas-absolute-grid-coords`, `vegas-constant-integrand`, `map-default-guards`.

**Design first.** D1 goes to the user through the planner before anything is implemented.

**Banking.** Diagnostic MadEvent runs happen here; new banked reference data
is produced only by the user on the M3 host. A banking need becomes one
comment on this PR listing exactly what to run.

## Sessions

| Session | Agent type | Depends on | Gate | Closes |
|---|---|---|---|---|
| D1 | manager → user | — | user approval | — |
| V1 | validation-dev (Opus) | D1 approved | `pixi run -e pythia validate-mlm-pythia` recorded | the item |
