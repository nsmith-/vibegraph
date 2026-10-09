---
type: Backlog Item
title: MLM matching at fixed beams is refused
description: "Matching or xqcut on a fixed-beam card (e.g. e+ e- > jets) is refused as FixedBeamMatching: the fixed-beam integrand cannot zero-weight clustered-out points."
area: feature
state: open
priority: low
closes_when: "MLM matching on a fixed-beam card runs and is gated against a banked MadGraph reference row."
blocked_by: []
opened: 2026-09-28
tags: [mlm, beams, madgraph-parity]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-06}
sources:
  - {id: todo, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L594-L600", title: "TODO.md entry T052"}
---
`ickkw = 1` or `xqcut > 0` on a fixed-beam card is refused as
`FixedBeamMatching`. The fixed-beam integrand has no path to zero-weight a
point the clustering rejects, and no MadGraph reference is banked for it
([note 41-mlm §4 M1 "Landed (implementation)"](../../history/notes/41-mlm-feature-sprint-plan.md)).

Kept as low-priority parity work (user, 2026-10-09). Resolving it needs the
clustering veto wired into the fixed-beam integrand and a reference row to gate
against. Matching on a decay is refused for good, by decision
([mlm-not-on-decays](../../decisions/mlm-not-on-decays.md)); the same
`FixedBeamMatching` refusal covers it today.
