---
type: Backlog Item
title: "The clustering scale is refused when every diagram has a four-line vertex"
description: "MadGraph's clustering splits a contact vertex with fake propagators; vibegraph's channel forests do not, so a four-fermion contact process cannot use the clustering scale."
area: feature
state: open
priority: low
closes_when: "Channel forests for the clustering scale split n ≥ 4 vertices as MadGraph does, pinned on per-event scales against MadEvent, and LFV_ST_TUMuTau_Vector integrates with σ matching MadGraph."
blocked_by: []
opened: 2026-11-07
tags: [scales, clustering, madgraph-parity, cms-gridpacks]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-07}
---
`channel forests for the clustering scale: every diagram of this process
carries a vertex of 4 lines, so MadGraph's generator would split them with
fake propagators rather than drop the diagrams`.

Refused in `main-202a540-200-pc` (1): LFV_ST_TUMuTau_Vector. Census:
[refusals at 202a540](../../validation/cms-gridpack-refusals-202a540.md).
