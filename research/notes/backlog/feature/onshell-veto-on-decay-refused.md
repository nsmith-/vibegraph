---
type: Backlog Item
title: A $ veto on a decay of a chain is refused
description: A $ on-shell veto inside a decay chain's decay (h > e+ e- mu+ mu- $ z) is refused as DecayOnShellVeto, where MadGraph accepts it.
area: feature
state: open
priority: medium
closes_when: A chain card with $ on a decay runs, and its σ matches MadEvent's on one such card (e.g. e+ e- > z h, h > e+ e- mu+ mu- $ z) under the seed policy.
blocked_by: []
opened: 2026-09-26
tags: [process-grammar, onshell-veto, decay-chain, madgraph-parity, refusal]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-06}
sources:
  - {id: todo, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L317-L332", title: "TODO.md entry T019"}
---
`Unsupported::DecayOnShellVeto` (`vibegraph-lib/src/diagrams/check.rs:233`,
raised at `:682`) refuses a `$` on a decay of a chain, e.g.
`e+ e- > z h, h > e+ e- mu+ mu- $ z`. MadGraph accepts it.

The check's own table names the work: marking a decay's own propagators, per
decay, so that `onshell.rs`'s windowed zeroing applies to the stitched
decay's s-channel lines rather than to the core process's. D2's Prop-level
marking is the natural carrier. Detail:
[note 38 §4 S3, D2, E1](../../38-process-grammar-sprint-plan.md).
