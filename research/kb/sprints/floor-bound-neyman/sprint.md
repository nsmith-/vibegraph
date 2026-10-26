---
type: Sprint
title: "Neyman on the 2 → 6 ladder, and an outcome-independent channel combine"
description: "P1 measured Neyman against ByAlpha on bbx's floor-bound channels and found the point-weighted iteration combine biased low; C1 replaces it with an outcome-independent combine and re-measures Neyman's target-mode figures."
status: draft
active: true
tags: [multichannel, allocation, neyman, iteration-combination, two-to-six, budget-ladder]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-26}
---
## Goal and exit criteria

P1 (done) tested the user's first fix for `floor-bound-channels-ignore-budget`:
Neyman allocation on `bbx_to_ccx_emmm_qcd0`, 20 paired seeds at 600k and
1.2M ([report](sessions/P1-report.md)). Neyman moves no points above the
floors, and its apparent gain is a bias in `ChannelHistory::combine_kept`,
which weights a channel's iterations by point counts chosen from earlier
iterations' variances. The user chose to fix that combine first (2026-10-26,
relayed by the planner); the floor-bound item stays open, blocked on it, for
the scaling floor in its own PR.

Done when every channel combine whose counts can vary uses weights fixed
independently of the run's outcomes, the stop rule reads the same
combination, the fixed-budget path's transformation class is stated (bit
identical, or statistically gated with every moved cell listed), and the CLI's
Neyman target mode and the `BlockAllocation::Neyman` doc figures are
re-measured against a paired outcome-independent arm.

## Scope

Backlog items claimed (one `Backlog: <slug>` line each in the draft PR):

- `point-weighted-combine-biased-by-earlier-iterations`

Filed from P1 and not claimed: `convergence-report-lacks-iteration-split`,
`bbx-drifts-from-madevent-with-budget`; `floor-bound-channels-ignore-budget`
is updated with the Neyman result.

**Banking.** No new reference data is needed. Any banking need becomes one
comment on the PR listing exactly what to run on the M3 host.

## Sessions

| Session | Agent type | Depends on | Gate | Closes |
|---|---|---|---|---|
| P1 | performance-dev (Opus) | — | paired 20-seed ladder, both allocations | — (measured Neyman; filed the combine bias) |
| C1 | performance-dev (Opus) | P1 | offline replay, then ≥5-seed target mode and 20 paired seeds on any deciding pair a default moves | `point-weighted-combine-biased-by-earlier-iterations` |
