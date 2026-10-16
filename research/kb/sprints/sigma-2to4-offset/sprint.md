---
type: Sprint
title: "2 → 4 σ offset against MadEvent"
description: "Establish which side is wrong in the +0.3 % fixed-beam u u~ > e+ e- g g offset against MadEvent, and fix it on the side that owns it."
status: draft
active: false
tags: [sigma, madevent-reference, phase-space, 2to4]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-15}
---
## Goal and exit criteria

The fixed-beam reproducer (√s = 500 GeV, `u u~ > e+ e- g g`, `ptj = mmjj
= 20`, fixed scales) reads +0.29 % (3.2σ) above MadEvent with |M|² identical
point by point, and the surplus sits in the high-ŝ tail. Done when the offset
is attributed to one side by a measurement whose own coverage is established
(σ in sliced regions on both sides, and an estimator whose error is honest
without adaptation), and either fixed here or written up as a reference-side
defect with a banking request for a corrected reference.

## Scope

Backlog items claimed (one `Backlog: <slug>` line each in the draft PR):

- `generic-2to4-sigma-offset-vs-madevent` (closed by this PR; see [closeout](closeout.md))

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
| [V1](sessions/V1.md) | validation-dev (Opus) | — | diagnosis only | generic-2to4-sigma-offset-vs-madevent |

Further sessions (a fix, its validation, performance where it applies, then
hygiene) are added once the diagnosis says which side owns the defect.
