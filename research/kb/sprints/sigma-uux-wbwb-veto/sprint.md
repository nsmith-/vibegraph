---
type: Sprint
title: "u u~ > w+ b w- b~ $ t t~ offset"
description: "Attribute the ~2 % gap between this side and patched MadEvent on the on-shell-veto t t~ row, and gate the row or document a measured reference-side defect."
status: draft
active: true
tags: [process-grammar, onshell-veto, madgraph-defect, info-cell, top]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-15}
---
## Goal and exit criteria

With MadGraph's `FFV2P1D_1` defect patched, MadEvent reads ~2 % below
this side on `u u~ > w+ b w- b~ $ t t~` (√s = 500, μ = 173) and drifts down
with budget, while this side does not move with budget or map. Pointwise
amplitudes agree to 1e-12 and the unrestricted process agrees. Done when the
gap is attributed and the row is gated or documented as a reference-side
defect with a measured cause.

## Scope

Backlog items claimed (one `Backlog: <slug>` line each in the draft PR):

- [uux-wbwb-onshell-veto-tt-2pct-off](../../backlog/validation/uux-wbwb-onshell-veto-tt-2pct-off.md)

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
| [V1](sessions/V1.md) | validation-dev (Opus) | — | diagnosis only | — |
| [V2](sessions/V2.md) | validation-dev (Opus) | V1 | diagnosis only | — |

Further sessions (a fix, its validation, performance where it applies, then
hygiene) are added once the diagnosis says which side owns the defect.
