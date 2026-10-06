---
type: Backlog Item
title: MLM matching at fixed beams or on decays is refused
description: Matching or xqcut on a fixed-beam card or on a decay is refused (FixedBeamMatching); the fixed-beam integrand cannot zero-weight clustered-out points.
area: feature
state: open
priority: low
closes_when: Matching at fixed beams (and on decays, or a recorded decision to keep that refused) runs against a MadGraph reference row, or both refusals are recorded as permanent scope decisions.
blocked_by: []
opened: 2026-09-28
tags: [mlm, beams, decays]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-06}
sources:
  - {id: todo, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L594-L600", title: "TODO.md entry T052"}
---
`ickkw = 1` or `xqcut > 0` on a fixed-beam card, or matching on a decay, is
refused as `FixedBeamMatching`. The fixed-beam integrand has no path to
zero-weight a point the clustering rejects, and there is no MadGraph reference
for it ([note 41-mlm §4 M1 "Landed (implementation)"](../../41-mlm-feature-sprint-plan.md)).

Resolving it needs the clustering veto wired into the fixed-beam integrand and
a reference row to gate against; for decays, a decision on what matching a
decay should mean, since MadGraph's MLM acts on the production process.
