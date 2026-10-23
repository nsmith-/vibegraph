---
type: Sprint
title: "Floor-bound channels: Neyman allocation on the 2 → 6 ladder"
description: "Expose the block allocation in the 2 → 6 ladder probe and measure Neyman against ByAlpha on bbx_to_ccx_emmm_qcd0, 20 paired seeds at 600k and 1.2M, to see whether budget reaches the floor-bound channels."
status: draft
active: true
tags: [multichannel, allocation, vegas, two-to-six, heavy-tail, budget-ladder]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-23}
---
## Goal and exit criteria

On `bbx_to_ccx_emmm_qcd0`, 563–585 of 615 channels sit at or under their
accepted-point floor under `ByAlpha` and draw the same points at 600k and 1.2M
for a given seed, so the ladder's per-seed sd falls only ×0.93 when the budget
doubles ([B1's report](../two-to-six-gate/sessions/B1-report.md)). The user
chose Neyman allocation as the first fix (2026-10-23). Done when a 20-seed
ladder at 600k and 1.2M under Neyman, read as paired per-seed differences
against a `ByAlpha` control on the same tree, shows the per-seed sd falling
about 1/√2 with budget and χ²/dof near 1, or the record says why Neyman does
not do it. A floor that scales with `neval` is the item's second step and is
not started here without the user's decision.

## Scope

Backlog items claimed (one `Backlog: <slug>` line each in the draft PR):

- `floor-bound-channels-ignore-budget`

**Banking.** No new reference data is needed. Any banking need becomes one
comment on the PR listing exactly what to run on the M3 host.

## Sessions

| Session | Agent type | Depends on | Gate | Closes |
|---|---|---|---|---|
| P1 | performance-dev (Opus) | — | paired 20-seed ladder, both allocations | `floor-bound-channels-ignore-budget` |
