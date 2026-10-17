---
type: Sprint
title: "t t~ MLM @1 σ excess"
description: "Attribute the +1.28 % excess of pp_to_ttx_0j1j_mlm's one-jet part over MadEvent to the reference, H1, or this side, starting from the reference-side and per-event steps."
status: draft
active: false
tags: [mlm, sigma, ttx, h1, madevent-reference]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-15}
---
## Goal and exit criteria

`@1` reads 583.19 ± 0.30 pb against MadEvent's 575.84 ± 0.78 (+8.85σ),
every seed above MadEvent's whole range, while `@0` agrees. Done when the
excess is attributed (reference, H1, or this side) with a measurement, fixed
if it is this side's, and the integrals cell gates on a five-seed agreement
with |pull| < 3.

The parallel sprint that owns the evaluator is pinning which colour structure
multiplies which Lorentz structure on `g g > t t~ g`, which feeds `@1`. This
PR therefore takes the reference-side and per-event steps first and stops if
the diagnosis points into `helas/`.

## Scope

Backlog items claimed (one `Backlog: <slug>` line each in the draft PR):

- `ttx-mlm-at1-sigma-high` (closed in [refdata10-cell-flips](../refdata10-cell-flips/closeout.md))

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
| [F1](sessions/F1.md) | feature-dev (Opus) | V2 | fmt, clippy, hermetic, `--skip-deps validate`; moved cells reported | — |
| [V3](sessions/V3.md) | validation-dev (Opus) | F1 | measurement only | — |
| [V4](sessions/V4.md) | validation-dev (Opus) | V3 | diagnosis only | — |
| [HY1](sessions/HY1.md) | hygiene-dev (Opus) | F1, V3, V4 | fmt, clippy, hermetic, scale/hadronic tests | — |

Further sessions (a fix, its validation, performance where it applies, then
hygiene) are added once the diagnosis says which side owns the defect.
