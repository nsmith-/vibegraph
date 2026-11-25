---
type: Backlog Item
title: "Frixione photon isolation (ptgmin > 0) is refused as an unimplemented cut"
description: "CMS aajj cards 425 and 427 parse their squared orders on PR 1, then stop at the run card: ptgmin = 15 turns on MadGraph's Frixione isolation, which cuts.rs lists in UNIMPLEMENTED_CUTS."
area: feature
state: open
priority: medium
closes_when: "ptgmin ≠ 0 compiles to MadGraph's isolation (cuts.f:1104-1222: R0gamma, xn, epsgamma, isoEM), with draj dropped as banner.py:4520 drops it. A per-event cut-decision comparison against MadEvent on one isolated-photon card agrees. Cards 425 and 427 run through gpdb with a 5-seed σ against MG."
blocked_by: []
opened: 2026-11-25
tags: [cuts, photon, isolation, run-card, cms, refusal]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-25}
sources:
  - {id: mg-cuts-iso, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/Template/LO/SubProcesses/cuts.f#L1104", title: "MadGraph cuts.f photon isolation (ptgmin, R0gamma, xn, epsgamma, isoEM)"}
  - {id: mg-banner-draj, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/madgraph/various/banner.py#L4520", title: "MadGraph banner.py: draj discarded when photon isolation is used"}
  - {id: h-g2, resource: "../../sprints/cms-gridpack-coverage/sessions/H-G2-report.md", title: "H-G2 report: photon collinear refusal and the isolation rule it defers to"}
---
`cuts.rs` lists `ptgmin` in `UNIMPLEMENTED_CUTS`. A card with `ptgmin ≠ 0` is
refused as `UnimplementedCutActive` before any pair cut is read.

In MadGraph, a nonzero `ptgmin` turns on Frixione isolation for every photon
(`cuts.f:1104-1222`). The `pT` of jet-class partons within `ΔR ≤ R0gamma` must
stay below `epsgamma·pT_γ·((1 − cos ΔR)/(1 − cos R0gamma))^xn`. `banner.py:4520`
then discards `draj`. The [H-G2 report](../../sprints/cms-gridpack-coverage/sessions/H-G2-report.md)
worked out the rule and deferred it.

Found by the sprint L validator on PR 1's request A (bin-pr1 9f196713). In
v022-1009 the aajj cards 425 and 427 (`QCD^2==2`) were refused at parse, for
squared orders. They now parse, enumerate and reach the run card, and they stop
on `ptgmin = 15`. PR 1 does not touch this.

The isolation sum runs over the final-state partons of each event. So it is a
cut on the full event, not a per-leg or pair minimum like the existing compiled
cuts.
