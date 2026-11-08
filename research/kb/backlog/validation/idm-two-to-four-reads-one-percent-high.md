---
type: Backlog Item
title: "idm's p p > h2 h2 mu+ mu- part reads +1.0% (2.4σ) above local MadGraph"
description: "At lhaid 247000, vibegraph 0.0023366 ± 0.0000024 pb (5 seeds) against MadGraph 0.002313 ± 0.0000095 (2 seeds); pre-existing, unconfirmed. The full card reads 0.962 ± 0.019 on one seed."
area: validation
state: open
priority: medium
closes_when: "The 2 → 4 part is compared at ≥ 5 MadGraph seeds; if the offset holds, it is localised by per-diagram dumps and fixed or explained; and the full idm card at its own PDF has a multi-seed σ/MG on a host where it finishes."
blocked_by: []
opened: 2026-11-08
tags: [sigma, madgraph-parity, idm, bsm, cms]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-08}
sources:
  - {id: hg2, resource: "../../sprints/cms-gridpack-coverage/sessions/H-G2-report.md", title: "H-G2 report, Found 4 and 5"}
  - {id: closeout, resource: "../../sprints/cms-gridpack-coverage/closeout.md", title: "Sprint H close-out draft: the closing rerun"}
---
H-G2 measured idm's `p p > h2 h2 mu+ mu-` part at LHAPDF 247000. vibegraph gives
0.0023366 ± 0.0000024 pb (5 seeds, χ²/dof 0.65). MadGraph gives
0.002313 ± 0.0000095 (2 seeds, `generate_events`). That is +1.0% at 2.4σ. Two
MadGraph seeds cannot calibrate this. The base binary was bit-identical, so
the offset predates sprint H.

The full idm card cannot settle it yet:
- In the closing rerun, at its own PDF, it reads 0.962 ± 0.019 on seed 1 only,
  and it exceeds gpdb's 600 s limit.
- Locally, one 2 → 6 flavour's survey did 687 of 1436 jobs in 56 minutes
  (H-G2 Found 5).

Medium, not high: the offset is not confirmed. If ≥ 5 MadGraph seeds keep it,
it is a silent wrong σ and the priority rises. The timeout is in
[multijet-mlm-cards-time-out-in-the-gridpack-campaign](../performance/multijet-mlm-cards-time-out-in-the-gridpack-campaign.md),
whose census lists idm_dilepton_mH110. idm_mH80, a panic in that campaign, now
times out too.
