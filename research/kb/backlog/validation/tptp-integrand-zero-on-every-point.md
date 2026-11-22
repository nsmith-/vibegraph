---
type: Backlog Item
title: "TpTpTo2T2STo2Mu2B_MS30 has zero weight on every point that passes the cuts"
description: "On TpTp 2Mu2B cards (MS30; 108 and 510–512 in v022-1009), every point that passes the cuts has |M|² × luminosity = 0, up to 46 M points, while MadGraph gives a σ: a possible silent wrong answer."
area: validation
state: open
priority: high
closes_when: "The zero is traced to a parameter, coupling or restriction, and either fixed (σ matching MadGraph) or recorded as a card defect that MadGraph also has."
blocked_by: []
opened: 2026-11-07
tags: [model-loading, param-card, cms-gridpacks]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-07}
---
`every one of the 36384 points the integration evaluated has zero weight,
though 17971 passed the cuts`. Compare the bound couplings with MadGraph's
`param_card` and restriction for this model.

Refused in `main-202a540-200-pc` (1): TpTpTo2T2STo2Mu2B_MS30. Census:
[refusals at 202a540](../../validation/cms-gridpack-refusals-202a540.md).

**Raised to high (sprint K close-out, 2026-11-22).** This is a possible
silent wrong answer: vibegraph returns a named zero while MadGraph returns a
σ. On the v0.2.2 release candidate and in the post-release campaign
`v022-1009`, CMS cards 108 and 510–512 (TpTpTo2T2STo2Mu2B) give "every one of
the 60000000 points … zero weight, though 28–46 M passed the cuts". In v0.2.1
those cards timed out instead.
