---
type: Backlog Item
title: "The misset² term of MadGraph's lepton smin is transcribed, not measured"
description: "#156 refuses misset < 0 where misset² is the lepton smin term; its el, ptl<n>min, mmll, mmnl and ptllmin branches are transcribed from setcuts.f, not measured."
area: validation
state: open
priority: low
closes_when: "MadGraph's printed smin is read for p p > e+ e- j (ptj 20, ptl 0, mmll 0) at misset = -1 and 0 (401 vs 400 expected), and either the refusal is narrowed to MadEvent's class-by-class smin or the conservative cases are recorded."
blocked_by: []
opened: 2026-11-19
tags: [cuts, misset, smin, madgraph-parity]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-19}
---
`vibegraph run-card` reads the card without the process, so it reports
`refusal: null` for a card this refusal stops.

Found by K-N1 ([report](../../sprints/v022-smeft-and-refusals/sessions/K-N1-report.md), Found).
