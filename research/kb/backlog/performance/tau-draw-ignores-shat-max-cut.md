---
type: Backlog Item
title: "The hadronic tau draw ignores the run card's own shat ceiling"
description: "dsqrt_shatmax is applied as a cut but does not bound the tau draw, unlike a decay chain's whole-state window, so a run with it set spends points above the ceiling."
area: performance
state: open
priority: low
closes_when: "The run card's sqrt(shat) ceiling folds into the tau bound (forced_shat_ceiling), with the artifact format bump and a measured efficiency gain on a card that sets it, and the banked sigma gates unchanged."
blocked_by: []
opened: 2026-11-05
tags: [phase-space, tau, cuts, sprint-g]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-05}
---
G-HY2 Found 1 ([report](../../sprints/v02-diagnoses/sessions/G-HY2-report.md)). The bound from a decay window is G-V1's `Cuts::shat_max`.
