---
type: Sprint
title: "bbx's ladder excess: the tails in channels sized by their α share"
description: "Diagnose why channels 613, 614 and 590 of bbx_to_ccx_emmm_qcd0 carry tails whose variance does not fall as points double, locate the tail points in phase space, and propose a sampler fix or a heavy-tail error model."
status: draft
active: true
tags: [two-to-six, multichannel, heavy-tail, budget-ladder, vegas, diagnosis]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-28}
---
## Goal and exit criteria

On `bbx_to_ccx_emmm_qcd0` the per-seed sd does not fall with budget (ratio
1.22 from 600k to 1.2M over 20 paired seeds). Neither Neyman (#75) nor a
scaling floor (#79) moved it: the floor-held channels carry about 2% of the
variance, and seed 55's 1.18e-8 pb point in α-share channel 613 drives the
excess. The user chose this study next (2026-10-28). T1 is a diagnosis: where
the tail points sit, why their channels' variance does not fall with points,
and what would fix it. The fix, or a heavy-tail error model in place of the
√n reading, is a later session after the user decides.

## Scope

Backlog items claimed (one `Backlog: <slug>` line each in the draft PR):

- `bbx-ladder-excess-in-alpha-share-tails`

Related, not claimed: `two-to-six-tail-point-unlocated` (the floor-bound
tail on `784582d`); T1's per-point record may close part of it, which goes
in its Found section.

**Banking.** No new reference data is needed. Any banking need becomes one
comment on the PR listing exactly what to run on the M3 host.

## Sessions

| Session | Agent type | Depends on | Gate | Closes |
|---|---|---|---|---|
| T1 | performance-dev (Opus) | — | diagnosis; candidate fix verified arithmetically on dumps | — (diagnosis for `bbx-ladder-excess-in-alpha-share-tails`) |
