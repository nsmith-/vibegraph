---
type: Sprint
title: "seed-record: ladder verdict, 20-seed recipe, p-value floor"
description: "seed-record prints the 20 + 20 ladder verdict on measured-spread σ, the driver documents the 20-seed SCATTER confirmation, worst-channel ties break on χ², and a deep-tail χ² p-value is a value or a bound, never 0."
status: draft
active: false
tags: [seed-record, tooling, chi2, ladder, refdata]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-16}
---
## Goal and exit criteria

Tooling the rest of the hand-off reads: the 2 → 6 SCATTER confirmation and any 20 + 20 ladder this sprint requests from the M3 are judged by it. Done when each item's closes_when holds, each new branch pinned by a test (a known-answer test for the upper incomplete gamma).

## Scope

Backlog items claimed (one `Backlog: <slug>` line each in the draft PR):

- `seed-record-ladder-verdict-and-tie-break`
- `chi2-p-value-floors-at-zero`

Sibling PRs of the same planner hand-off ("statistical gates and seed calibration", 2026-10-16) are `sigma-pulls-lean-high`, `refdata10-cell-flips`, `sigma-calibration-rerecord`, `ee-ee-nsz-low-budget`, `two-to-six-gate`, `vegas-low-acceptance-bins`, `vegas-absolute-grid-coords`, `vegas-constant-integrand`, `map-default-guards`, `mlm-pythia-tolerance`.


**Banking.** Diagnostic MadEvent runs happen here; new banked reference data
is produced only by the user on the M3 host. A banking need becomes one
comment on this PR listing exactly what to run.

## Sessions

| Session | Agent type | Depends on | Gate | Closes |
|---|---|---|---|---|
| F1 | validation-dev (Sonnet) | — | hermetic + touched-crate tests | both items |
| HY1 | hygiene-dev (Sonnet) | F1 | hermetic | — |
