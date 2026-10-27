---
type: Sprint
title: "A channel floor that scales with the budget"
description: "Give floor-bound multichannel channels a floor that grows with neval, so budget reaches them, and measure it on bbx_to_ccx_emmm_qcd0 with 20 paired seeds at 600k and 1.2M under the plain-mean combine."
status: draft
active: true
tags: [multichannel, allocation, coverage-floor, two-to-six, heavy-tail, budget-ladder]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-27}
---
## Goal and exit criteria

On `bbx_to_ccx_emmm_qcd0`, 551–580 of 615 channels sit at the
accepted-point floor and draw identical points at 600k and 1.2M, and floor
points are about half of each 1.2M iteration under either allocation
([floor-bound-neyman P1](../floor-bound-neyman/sessions/P1-report.md)).
Neyman cannot reach them; #75 made channel terms combine as the plain mean,
so varying counts no longer read as a gain. The user chose the scaling floor
next (2026-10-27). Done when the item's `closes_when` is met: a 20-seed
paired ladder at 600k and 1.2M with the scaling floor shows the per-seed sd
falling about 1/√2 and χ²/dof near 1, or the record says why it fails.
Switching the default floor is the user's decision (D1), with the gate impact
measured.

## Scope

Backlog items claimed (one `Backlog: <slug>` line each in the draft PR):

- `floor-bound-channels-ignore-budget`

**Banking.** No new reference data is needed. Any banking need becomes one
comment on the PR listing exactly what to run on the M3 host.

## Sessions

| Session | Agent type | Depends on | Gate | Closes |
|---|---|---|---|---|
| S1 | performance-dev (Opus) | — | offline sizing, then 20 paired seeds × 2 rungs per arm; validate before/after for D1 | `floor-bound-channels-ignore-budget` |
