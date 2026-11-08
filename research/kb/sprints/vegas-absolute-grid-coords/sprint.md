---
type: Sprint
title: "Where the weight tail meets the cut edges: measurements for the sampler seam design"
description: "Measurement only, after the user's pivot (2026-10-20): per-channel edge proximity and coordinate alignment of the weight tail, cross-channel (τ, y) grid similarity and rejected-point cost, feeding the sampler and phase-space seam design. Absolute grid coordinates are not built."
status: draft
active: true
tags: [vegas, phase-space, maps, cut-edges, performance]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-16}
---
## Goal and exit criteria

**Pivoted 2026-10-20 (the user's decision).** Absolute grid coordinates are not built: the planner's design draft (PR #57) finds they sharpen almost no edge on the banked rows. This sprint now runs one measurement session, [M1](sessions/M1.md), whose numbers fed `sampler-and-phase-space-seams-designed-apart`, settled in [the joint seam design](../../phase-space/sampler-and-map-seams.md). The original plan follows for the record.

An option the user asked for. Done when the VEGAS↔channel contract is inverted with the eager path bit-identical, every analytic map is a fixed transform with the window inverted through it, `--map-grid-coords` (and its `auto`) exists, and its payoff is measured on `pp_to_llj` and `pp_to_jj` with ≥5 seeds.

## Scope

Backlog items claimed (one `Backlog: <slug>` line each in the draft PR):

- `vegas-grid-coords-not-absolute`: closed as refused (D2) by [the joint seam design](../../phase-space/sampler-and-map-seams.md), which M1's numbers fed

Sibling PRs of the same planner hand-off ("statistical gates and seed calibration", 2026-10-16) are `sigma-pulls-lean-high`, `seed-record-tooling`, `refdata10-cell-flips`, `sigma-calibration-rerecord`, `ee-ee-nsz-low-budget`, `two-to-six-gate`, `vegas-low-acceptance-bins`, `vegas-constant-integrand`, `map-default-guards`, `mlm-pythia-tolerance`.

**Sequencing.** After `vegas-low-acceptance-bins` (both rework the grid's binning in `vegas.rs`). Default unchanged.

**Banking.** Diagnostic MadEvent runs happen here; new banked reference data
is produced only by the user on the M3 host. A banking need becomes one
comment on this PR listing exactly what to run.

## Sessions

| Session | Agent type | Depends on | Gate | Closes |
|---|---|---|---|---|
| P1 | performance-dev (Opus) | vegas-low-acceptance-bins P1 | inert plumbing: every seeded golden bit-identical | — |
| P2 | performance-dev (Opus) | P1 | option measured, ≥5 seeds, defaults unmoved | the item |
| M1 | performance-dev (Opus) | the pivot | measurement only, probes ignored, no production change | — (feeds the seam design) |
