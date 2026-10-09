---
type: Backlog Item
title: MLM cards with a resolved ptj below xqcut are refused
description: With auto_ptj_mjj = F or ptj < 0, MadEvent's xqcut tau floor becomes a channel-dependent cut, and vibegraph refuses the card instead.
area: feature
state: open
priority: low
closes_when: "A matched card with a resolved ptj < xqcut integrates with MadEvent's per-channel tau floor and matches MadEvent's sigma on a banked row."
blocked_by: []
opened: 2026-09-28
tags: [mlm, cuts, madgraph-parity]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-06}
sources:
  - {id: todo, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L594-L600", title: "TODO.md entry T052"}
---
MadEvent applies a tau minimum `(Σ xe)²/s` under `xqcut` (`myamp.f:337-560`,
`setxqcuts` at `setcuts.f:892-955`). With the default resolved `ptj = xqcut`
it is implied by the rewritten cuts and cuts nothing (pinned on 200k points by
`cuts::madevents_xqcut_tau_floor_is_implied_by_the_rewritten_cuts`). With a
resolved `ptj < xqcut` (`auto_ptj_mjj = F`, or `ptj < 0`) it changes sigma
and differs between integration channels, so the card is refused
(`XqcutAboveJetThreshold`), a scoped refusal rather than an implementation.

MadGraph accepts this card, so this is a parity gap. Detail:
[note 41-mlm §4 M1 "Landed (implementation)"](../../41-mlm-feature-sprint-plan.md).

Kept as low-priority parity work (user, 2026-10-09). Only a non-default card
reaches it, and the behaviour it would reproduce is a cut that depends on the
integration channel, an artefact of MadEvent's sampling rather than physics.
