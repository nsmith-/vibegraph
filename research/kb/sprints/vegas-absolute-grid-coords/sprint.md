---
type: Sprint
title: "VEGAS grid on absolute invariant coordinates"
description: "Add --map-grid-coords: bin each invariant's VEGAS coordinate on the absolute s/s_tot or −t/s_tot scale with the draw restricted to the point's window, as MadEvent's sample_get_x does, and measure it on the cut-edge rows."
status: draft
active: true
tags: [vegas, phase-space, maps, cut-edges, performance]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-16}
---
## Goal and exit criteria

An option the user asked for. Done when the VEGAS↔channel contract is inverted with the eager path bit-identical, every analytic map is a fixed transform with the window inverted through it, `--map-grid-coords` (and its `auto`) exists, and its payoff is measured on `pp_to_llj` and `pp_to_jj` with ≥5 seeds.

## Scope

Backlog items claimed (one `Backlog: <slug>` line each in the draft PR):

- `vegas-grid-coords-not-absolute`

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
