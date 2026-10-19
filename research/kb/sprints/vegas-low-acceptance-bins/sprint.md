---
type: Sprint
title: "VEGAS grid starves low-acceptance bins"
description: "Measure MadEvent's per-bin acceptance rescale, Σ|w| adaptation and a bin-width bound as opt-in grid rules on the banked rows (σ pulls, ε_unw, Hill index, ≥5 seeds) before adopting one."
status: draft
active: false
tags: [vegas, grid-adaptation, heavy-tail, mlm, performance]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-16}
---
## Goal and exit criteria

Every integration moves under any of these rules, so they land opt-in and measured; a default change is a user decision with the measurements beside it. Done when the three rules are measured on the banked rows with ≥5 seeds and either one is proposed for adoption or the record says why none is.

## Scope

Backlog items claimed (one `Backlog: <slug>` line each in the draft PR):

- `vegas-grid-starves-low-acceptance-bins`

Sibling PRs of the same planner hand-off ("statistical gates and seed calibration", 2026-10-16) are `sigma-pulls-lean-high`, `seed-record-tooling`, `refdata10-cell-flips`, `sigma-calibration-rerecord`, `ee-ee-nsz-low-budget`, `two-to-six-gate`, `vegas-absolute-grid-coords`, `vegas-constant-integrand`, `map-default-guards`, `mlm-pythia-tolerance`.

**Sequencing.** Touches `vegas.rs` with `vegas-absolute-grid-coords` and `vegas-constant-integrand`; those are sequenced, smallest first. The user decided (2026-10-17) to make K = 6 the default bin-width bound, so σ calibrations move; the re-record (`sigma-calibration-rerecord`) runs after this PR and `map-default-guards`.

**Banking.** Diagnostic MadEvent runs happen here; new banked reference data
is produced only by the user on the M3 host. A banking need becomes one
comment on this PR listing exactly what to run.

## Sessions

| Session | Agent type | Depends on | Gate | Closes |
|---|---|---|---|---|
| P1 | performance-dev (Opus) | vegas-constant-integrand merged | opt-in, defaults bit-identical; ≥5-seed table | the item, if no default change is proposed |
