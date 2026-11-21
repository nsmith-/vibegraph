---
type: Backlog Item
title: "DY1jToLL_M-1to10_Pt-100to200 reads 2.7% above MadGraph under the ptj1 rank cut"
description: "CERN targeted check card 409 (ptj1min = 100, ptj1max = 200) gives 5.292 ± 0.013 pb against MadGraph's 5.151, a pull of +4.65 on one seed. Its neighbouring bins read +0.2–0.3%."
area: validation
state: open
priority: high
closes_when: "The card's σ is measured at ≥ 5 seeds against a MadGraph σ regenerated on the same run card, and either agrees by the multi-seed criterion or the cause is named and fixed or refused."
blocked_by: []
opened: 2026-11-21
tags: [rank-cuts, cuts, sigma, cms-gridpacks, silent-wrong-answer]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-21}
sources:
  - {id: cern, resource: "../../sprints/v022-smeft-and-refusals/log.md", title: "Sprint K log, CERN targeted check on 400c88f9"}
  - {id: kc2, resource: "../../sprints/v022-smeft-and-refusals/sessions/K-C2-report.md", title: "K-C2 report"}
---
Seed 1 of the CERN host's targeted check on the v0.2.2 release candidate
(400c88f9), at survey settings (1% target, neval 120000). The reference is
the gridpack's own MadGraph σ times the small-width ratio.

K-C2 measured the same family locally against MadGraph 3.5.7, with run cards
it rebuilt from the 600toInf card:
- Pt-70to100: +0.278%;
- Pt-200to400: +0.204%.

Both are the ℓℓj family's known offset (`llj-family-reads-high-vs-seed-records`).
So +2.7% in the bin between them is out of line. Several readings remain open:
- a VEGAS seed that missed a region, though a missed region reads *low*, not
  high;
- a gridpack run card that differs from K-C2's rebuilt bins in more than
  `ptj1min`/`ptj1max`;
- the gridpack's MadGraph version (2.x) treating the rank cut differently;
- a fault in the rank cut's upper edge.

The first step is to diff the gridpack's run card against K-C2's rebuilt
100to200 card, then run 5 seeds.
