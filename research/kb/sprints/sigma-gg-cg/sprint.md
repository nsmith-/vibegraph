---
type: Sprint
title: "gg_to_gg_cg σ offset"
description: "Attribute the converged −0.22 % offset of gg_to_gg_cg (dynamical_scale_choice = 3) with a measurement, and flip its integrals cell to a gate or record why it stays info."
status: draft
active: false
tags: [smeftsim, sigma, scale-fallback, dynamical-scale-choice-3, info-cell]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-15}
---
## Goal and exit criteria

Five seeds converge at rel −2.21e-3 against a reference error of 8.5e-4;
the budget ladder settles rather than shrinks. The scale formula replays all
10000 banked events and `gg_to_gg` under a card differing only in the scale
choice agrees, so what is left is the coupling across the cut region rather
than on kept events. Done when the offset is attributed with a measurement and
the integrals cell is flipped to a gate, or the attribution says why it stays
info.

## Scope

Backlog items claimed (one `Backlog: <slug>` line each in the draft PR):

- `gg-to-gg-cg-sigma-offset` (closed by this PR; see [closeout](closeout.md))

Sibling PRs of the same planner hand-off ("cross sections that disagree
with MadEvent, cause unattributed") are `sigma-2to4-offset`, `sigma-ttx-mlm-at1`,
`sigma-h1-ickkw0-census`, `sigma-reference-scatter`, `sigma-ee-mumua`,
`sigma-gg-cg` and `sigma-uux-wbwb-veto`. Five of the seven offsets put this
side high, which makes reference-side under-coverage a candidate common cause;
`sigma-2to4-offset` is the cheapest test of it and its result is relayed to the
others as a watch item.

**Banking.** Diagnostic MadEvent runs happen here; new banked reference
data is produced only by the user on the M3 host. A banking need becomes one
comment on this PR listing exactly what to run.

## Sessions

| Session | Agent type | Depends on | Gate | Closes |
|---|---|---|---|---|
| [V1](sessions/V1.md) | validation-dev (Opus) | — | diagnosis only | gg-to-gg-cg-sigma-offset |

Further sessions (a fix, its validation, performance where it applies, then
hygiene) are added once the diagnosis says which side owns the defect.
