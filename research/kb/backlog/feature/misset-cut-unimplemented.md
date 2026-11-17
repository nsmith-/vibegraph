---
type: Backlog Item
title: "A positive misset cut is refused as unimplemented"
description: "bbDM_2HDMa cards (7) set misset = 150; vibegraph refuses it, so the missing-ET-cut DM samples cannot run."
area: feature
state: open
priority: medium
closes_when: "misset is implemented as cuts.f's missing-ET cut (the vector sum of the neutrinos' and the model's invisible particles' pT, as MadGraph defines invisible), with σ on a bbDM card against MadGraph by the multi-seed criterion, and the 7 cards integrate."
blocked_by: []
opened: 2026-11-17
tags: [cuts, run-card, refusal, dark-matter, cms-gridpacks]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-17}
---
The CERN host's survey refused gridpacks 154, 155 and 870–874 (bbDM_2HDMa) for
`misset = 150`. Its driver reset the cut to 0 and read σ 2.7–454× MadGraph's,
so the refusal was correct.

Read how cuts.f decides which particles are invisible before implementing it.
For a BSM model that probably depends on the PDG code. The −1 "off" value is
[`run-card-off-sentinel-values-refused`](run-card-off-sentinel-values-refused.md).
