---
type: Sprint
title: "ee_to_ee_nsz under-samples below the plateau"
description: "Locate the region e+ e- > e+ e- $$ z misses below the plateau budget, and show the CLI default budget reaches the plateau on ≥20 seeds or fix the sampler so it does."
status: draft
active: false
tags: [process-grammar, vegas, budget-ladder, bhabha, seed-sweep]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-16}
---
## Goal and exit criteria

Twenty-seed scatter is 3.84 and 2.01 χ²/dof at ×¼ and ×1 against 1.2–1.5 on the plateau. Done when the under-sampled region is identified (per-channel split of the low seeds against the plateau) and the CLI default budget is shown to reach the plateau on every one of ≥20 seeds, or the sampler is fixed so it does.

## Scope

Backlog items claimed (one `Backlog: <slug>` line each in the draft PR):

- `ee-ee-nsz-sigma-low-at-low-budget`

Sibling PRs of the same planner hand-off ("statistical gates and seed calibration", 2026-10-16) are `sigma-pulls-lean-high`, `seed-record-tooling`, `refdata10-cell-flips`, `sigma-calibration-rerecord`, `two-to-six-gate`, `vegas-low-acceptance-bins`, `vegas-absolute-grid-coords`, `vegas-constant-integrand`, `map-default-guards`, `mlm-pythia-tolerance`.


**Banking.** Diagnostic MadEvent runs happen here; new banked reference data
is produced only by the user on the M3 host. A banking need becomes one
comment on this PR listing exactly what to run.

## Sessions

| Session | Agent type | Depends on | Gate | Closes |
|---|---|---|---|---|
| V1 | validation-dev (Opus) | — | diagnostic | — (or the item, if the default budget already reaches the plateau) |
| F1 | performance-dev (Opus), if the sampler needs a fix | V1 | 20-seed ladder + `--skip-deps validate` | the item |
