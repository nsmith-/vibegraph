---
type: Backlog Item
title: "vibegraph σ reads 0.1–0.3% above local MadEvent on many cards, cut cards included"
description: "Across sprint H, vibegraph sits +0.07% to +0.18% above local MadEvent (LHAPDF 247000, hard_survey = 2) on most cut cards; an uncut quadrature check clears vibegraph's PDF, |M|², flux and map."
area: validation
state: open
priority: medium
closes_when: "A quadrature built on MadEvent's own PDG2PDF, SMATRIX1 and PASSCUTS for one cut card (e.g. u u~ > e+ e- or p p > a j j) settles which side the offset is on, and it is either fixed with a test or recorded as MadEvent's integration (madgraph-defects)."
blocked_by: []
opened: 2026-11-05
tags: [sigma, madgraph-parity, madevent, quadrature]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-05}
sources:
  - {id: hm2, resource: "../../sprints/cms-gridpack-coverage/sessions/H-M2-report.md", title: "H-M2 report, Found 2 (seven mirror rows)"}
  - {id: hq1, resource: "../../sprints/cms-gridpack-coverage/sessions/H-Q1-report.md", title: "H-Q1 report, the uncut quadrature and Found 3"}
  - {id: hn1, resource: "../../sprints/cms-gridpack-coverage/sessions/H-N1-report.md", title: "H-N1 report, p p > a j j σ"}
---
Measured in sprint H against local MadEvent, with LHAPDF 247000 on both sides,
`hard_survey = 2` and a fresh directory per seed, 5 seeds per side:

| rows | vibegraph above MadEvent | pulls |
|---|---|---|
| seven mirror rows (`u u~ > e+ e-`, `u g > u z`, `b b~ > mu+ mu-`, `g b > z b`, both orderings), H-M2 | +0.07% to +0.11% | +1.1 to +2.4, together about 4.7σ |
| `p p > a j j QCD=0`, H-N1 | +0.18% | +2.2 |
| `p p > z j` at `pt_min_pdg {23: 100}`, H-C1 | +0.10% | |

On an *uncut* `u u~ > e+ e-` card, H-Q1 found vibegraph exact to
+2.8e-5 ± 8.9e-5 against a Gauss–Legendre quadrature built on MadEvent's own
`PDG2PDF` and `SMATRIX1`. So the offset is not in vibegraph's PDF, |M|², flux or
(τ, y) map. H-Q1 also found MadEvent 0.2–0.45% low at large √ŝ on QCD 2→3, even
at `hard_survey = 2` (`madgraph-defects.md`). That is the leading suspect. The
check that settles it is the same quadrature with `PASSCUTS` on a cut card.
H-Q1's driver is `hq1quad_main.f`; its scratch path is in the H-Q1 report.
