---
type: Backlog Item
title: "ptj1min and ptj1max with real values are refused as unimplemented"
description: "CMS DY1jToLL_M-1to10_Pt-* bins (6 cards) set ptj1min/ptj1max to cut the leading jet's pT; vibegraph refuses them, so binned samples cannot run."
area: feature
state: open
priority: medium
closes_when: "ptj1min–ptj4min and ptj1max–ptj4max are implemented as MadGraph's cuts.f rank cuts (ordering by pT, applied in the τ floor where setcuts.f does), with σ on two DY1j pT bins against MadGraph by the multi-seed criterion, and the 6 cards integrate."
blocked_by: []
opened: 2026-11-17
tags: [cuts, run-card, refusal, cms-gridpacks]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-17}
---
The CERN host's survey (gridpacks 407–412, `DY1jToLL_M-1to10_Pt-*`) found these
refused. Its driver reset the cuts to the default and got the unbinned σ
(1.24e6 pb) in every bin, so the refusal was correct.

The non-positive "off" values compile to the default cuts. This item covers
real cut values.
