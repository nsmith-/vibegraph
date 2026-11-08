---
type: Design Decision
title: "Sprint I: refuse the equivalent-photon limit, retiring pp_to_epemjj_qcd0"
description: "D1: the initial-collinear guard covers emission through a massless vector exchange too, so the banked EWK e+ e- j j row at ptj = 0, divergent at tree level, is refused and retired."
decided: 2026-11-08
decided_by: human:nsmith-
status: stable
tags: [cuts, ir-safety, guards, banked-rows, sprint-i]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-08}
verified: [{by: "human:nsmith-", at: 2026-11-08}]
sources:
  - {id: ic2, resource: "../sessions/I-C2-report.md", title: "I-C2 report: the ladder and the pole probe"}
---
**The question.** I-C2's initial-collinear guard refuses a massless outgoing
leg at a vertex with a massless incoming leg and a massless spacelike
propagator, unless that leg has a pT minimum or a rapidity maximum. The banked
long-tier row `pp_to_epemjj_qcd0` (`u d > e+ e- u d QCD=0` and its family, at
`ptj = 0`, `etaj = -1`, `mmjj = 4`, `mmll = 50`) has such a limit through a
t-channel photon: the equivalent-photon logarithm off a massless quark.

**The evidence** ([I-C2](../sessions/I-C2-report.md)).
- **The pole probe.** |M|²·|t| reads 1.936969e-6, constant to 7 digits over 8
  decades of |t| as the outgoing u approaches its beam (√ŝ = 1 TeV, x = 0.5).
  The control with `/ a` falls as |t|². The measure goes as d|t|, so σ diverges
  as ∫dt/t.
- **The ptj ladder** is flat below 1 GeV (0.207333 ± 0.000139 pb at
  `ptj = 0`, χ²/dof 0.13), but only because every peripheral channel floors its
  pole at 1 GeV² (`DEFAULT_POLE_SCALE × POLE_FRACTION_OF_FIDUCIAL_SCALE`).
  MadEvent's `10/stot` floor plays the same role, so the banked 0.36% agreement
  is two samplers' cutoffs agreeing.

**The options.**
- (a) Refuse the limit through a vector exchange too, and retire the row.
- (b) Exempt it as a documented gap, keeping the row.

**Decided (a)** (the user, 2026-11-08). The row is retired from the manifest
and its long-tier gate removed in sprint I's PR 2. Re-banking a regulated
variant (a ptj or etaj cut) on the M3 is a follow-up item.
