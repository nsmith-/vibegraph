---
type: Backlog Item
title: Stochastic-rounding output refuses mixed-multiplicity cards
description: --strategy stochastic-rounding is refused on cards of several parts (@N multiplicities, or outgoing-mass sets) because unit weights leave each part's share of the file to the realised sample.
area: feature
state: open
priority: medium
closes_when: generate --strategy stochastic-rounding accepts a card of several parts (multiplicities or outgoing-mass sets), and each part's event count follows its integrated σ (checked against the buffered strategy's per-part normalisation).
blocked_by: []
opened: 2026-10-01
tags: [mlm, lhef, unweighting, mixed-multiplicity]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-06}
sources:
  - {id: todo, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L249-L254", title: "TODO.md entry T007"}
---
`StochasticRounding` keeps unit weights (`IDWTUP = +3`), so each `@N` part's
share of the file is whatever the realised sample gives. `Buffer` instead
normalises each part to its own integrated σ. A plan of several parts is
therefore refused in the CLI (`refuse_rounding_on_mixed_multiplicity`,
`vibegraph-lib/src/generate.rs:330`) and in the emitter
(`vibegraph-lib/src/lhef/emit.rs`, the per-part check near L228).

Lifting it needs a per-part event count drawn from the parts' integrated σ
A mixed-mass card (`p p > l+ l-` with `l = e, ta`) has two such parts under one
`@0`, split by outgoing masses.

(MadEvent's `unwgt.f` scales by `xsecabs/xsum`). Each part is then rounded to
its own target count.

Detail: [note 41 P12](../../history/notes/41-mlm-feature-sprint-plan.md) ("P12 Landed: both F-A policies").
