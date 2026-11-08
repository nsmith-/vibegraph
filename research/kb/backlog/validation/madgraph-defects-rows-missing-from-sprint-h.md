---
type: Backlog Item
title: "Three MadGraph behaviours sprint H measured are missing from the madgraph-defects catalogue"
description: "Not catalogued: the ~1% iteration-combination shift on HT multi-jet cards, the aS-off-SMINPUTS-3 refusal on HAHM models, and setcuts.f's first-row cut-class rule."
area: validation
state: open
priority: low
closes_when: "madgraph-defects (or its linked concepts) records all three with their evidence, the iteration-weighting concept gives the HT-card size, and the HAHM lhacode swap is scripted where the survey runs MadGraph."
blocked_by: []
opened: 2026-11-07
tags: [madgraph-defect, catalogue, madevent, ufo, cuts]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-07}
sources:
  - {id: hg1, resource: "../../sprints/cms-gridpack-coverage/sessions/H-G1-report.md", title: "H-G1 report, Found 8"}
  - {id: hm2, resource: "../../sprints/cms-gridpack-coverage/sessions/H-M2-report.md", title: "H-M2 report, Found 1"}
  - {id: hk1, resource: "../../sprints/cms-gridpack-coverage/sessions/H-K1-report.md", title: "H-K1 report, Found: the first-row rule"}
---
Each is a MadGraph behaviour that moves a reference or blocks an oracle, and none
has a row in [madgraph-defects](../../validation/madgraph-defects.md):

1. **Iteration combination on HT cards** (H-G1 Found 8). Paired equal-weight
   re-combination of `G*/results.dat` moves σ +1.65, +0.41, +1.14, +1.01% on the
   HT-binned 2→4 card (4 seeds), about +0.2% on the 2→3 card. The catalogue's
   mean²/variance row and [its concept](../../validation/madevent-iteration-weighting-bias.md)
   give 0.1–0.2% at fixed beams. Those runs used MadEvent's default survey
   (H-Q1 Found 2), so record the size with that caveat, and keep `G*/results.dat`
   per seed on such references.
2. **UFO models with `aS` off SMINPUTS 3** (H-M2 Found 1). MadGraph 3.7.1 refuses
   them (`import_ufo.py:905`), and CMS's HAHM gridpacks ship one; with python
   LHAPDF present, `update_dependent` writes αs into SMINPUTS 3 whatever parameter
   is there (`Gf` on HAHM). Any oracle on such a model needs H-M2's lhacode swap.
3. **The first-row cut-class rule** (H-K1). With `group_subprocesses = False`, or
   one matrix element per P directory, a b row merged into a light-quark matrix
   element takes that element's cut classes (`setcuts.f:217`). vibegraph never
   does this; no banked card reaches it. A registered-deviation note covers it.
