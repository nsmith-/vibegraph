---
type: Sprint
title: "2 → 6 integrals cells: budget ladder and stop scale"
description: "Re-measure stop_scale's few-accepted-point inflation, re-run the 2 → 6 budget ladder under the accepted-point floor with 20+ seeds on the deciding rungs, and gate both integrals cells or record why not."
status: draft
active: true
tags: [two-to-six, multichannel, heavy-tail, seed-sweep, budget-ladder, gate-promotion]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-16}
---
## Goal and exit criteria

Both items are measured on the same two rows (`uux/bbx_to_ccx_emmm_qcd0`, 579/615 channels) through the same probe, and the stop scale decides how the ladder's rungs stop. Done when the stop-scale mechanism is re-measured under `pooled_scale` (and fixed if it persists, never by a retuned factor), and the ladder's single-seed swings are shown to shrink with budget (note 32 §5.4 falsifier) so both cells gate, or the record says why not. Under the look-elsewhere SCATTER rule the two MadEvent references themselves flag at p < 1e-16; per D1's third amendment the cells stay `info` against MadEvent if a 20-seed confirmation confirms, which is a banking request to the user.

## Scope

Backlog items claimed (one `Backlog: <slug>` line each in the draft PR):

- `two-to-six-integrals-not-enforced`
- `stop-scale-inflated-by-few-accepted-iterations`

Sibling PRs of the same planner hand-off ("statistical gates and seed calibration", 2026-10-16) are `sigma-pulls-lean-high`, `seed-record-tooling`, `refdata10-cell-flips`, `sigma-calibration-rerecord`, `ee-ee-nsz-low-budget`, `vegas-low-acceptance-bins`, `vegas-absolute-grid-coords`, `vegas-constant-integrand`, `map-default-guards`, `mlm-pythia-tolerance`.


**Banking.** Diagnostic MadEvent runs happen here; new banked reference data
is produced only by the user on the M3 host. A banking need becomes one
comment on this PR listing exactly what to run.

## Sessions

| Session | Agent type | Depends on | Gate | Closes |
|---|---|---|---|---|
| P1 | performance-dev (Opus) | — | re-measurement; fix only if the mechanism persists | `stop-scale-inflated-by-few-accepted-iterations` |
| V1 | validation-dev (Opus) | P1, the 20-seed MadEvent confirmation | ladder ≥20 seeds on deciding rungs | `two-to-six-integrals-not-enforced` |
