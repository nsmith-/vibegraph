---
type: Sprint
title: "ee → μμγ radiative-return excess"
description: "Attribute the ~1 % ours-high excess of ee_to_mumua in the radiative-return pt(γ) windows to one side with a measurement, and re-arm the row's gates on attribution."
status: draft
active: true
tags: [ee-to-mumua, sigma, madgraph-coverage, info-cell]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-15}
---
## Goal and exit criteria

σ sits a fixed +1.04 % above the bank, localised to `pt(γ) ∈ [39.4, 144)`;
MadGraph's own `m(μμ)` partition recovers this side's total to +0.016 % and its
two `dummy_cuts` partitions disagree at 16.7σ. Done when the excess is
attributed to one side with a measurement; on attribution the row's σ pull and
samples cell re-arm as enforced gates (against a re-banked reference if the
reference is at fault).

## Scope

Backlog items claimed (one `Backlog: <slug>` line each in the draft PR):

- [ee-mumua-radiative-return-sigma-high](../../backlog/validation/ee-mumua-radiative-return-sigma-high.md)

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

Further sessions (a fix, its validation, performance where it applies, then
hygiene) are added once the diagnosis says which side owns the defect.
