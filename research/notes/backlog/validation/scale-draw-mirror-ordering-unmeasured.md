---
type: Backlog Item
title: Scale draw uses only the direct ordering, untested against the mirror
description: The per-point scale configuration is drawn from the direct ordering's AMP2 alone; whether σ moves when the mirror is included is unmeasured.
area: validation
state: open
priority: medium
closes_when: σ on the dynamical-scale hadronic rows has been measured with the draw formed from the luminosity-weighted direct+mirror combination, and it agrees with the direct-only draw within Monte Carlo (≥5 seeds); otherwise a finding is filed.
blocked_by: []
opened: 2026-08-03
tags: [scales, proton, mirror, falsifier, pp-to-llj]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-06}
sources:
  - {id: todo, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L525-L531", title: "TODO.md entry T040"}
---
For hadronic integrands, `ProtonIntegrand::scale_channel` (`vibegraph-lib/src/proton.rs`,
doc at ~:2370) draws one scale configuration per group per point from the
*direct* ordering's momenta and `AMP2`, then applies that scale to both the direct
and the mirrored term. The falsifier was named when this was designed: σ must not
move outside Monte Carlo when the draw is formed from the luminosity-weighted
combination of both orderings. It has never been measured. If σ does move, the
ordering is a third partition axis, next to the channel partition and the draw,
and that is a finding about the dynamical-scale σ gates, not an inline fix.

Related caveat, recorded in [note 29](../../29-v01-validation-sprint-plan.md)
"Chain B results" (table at ~L5598, open items ~L5734): with the draw, five-seed
scatter on `pp_to_llj_dyn` is χ²/dof 6.38 at a 75k budget and clean from 150k up.
Any budget cut on that row would therefore bite. Design and falsifier:
[note 29](../../29-v01-validation-sprint-plan.md) §B.11 "The mirror ordering".
