---
type: Backlog Item
title: Restrict-card parameters set to exactly 1 are not fixed
description: MadGraph fixes a restrict-card parameter set to exactly 1 (as it fixes zeros), but the loader locks only zeros, so a later param card could move a value MadGraph holds at 1.
area: feature
state: open
priority: low
closes_when: The loader fixes restrict-card parameters equal to 1 as MadGraph does, and a restrict card with a 1-valued parameter pins it, compared against MadGraph's generated param card.
blocked_by: []
opened: 2026-09-07
tags: [non-sm-ufo, ufo, restrict-card, madgraph-parity]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-06}
sources:
  - {id: todo, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L938-L939", title: "TODO.md entry T084"}
---
`ParameterSet::apply_restrict` (`vibegraph-lib/src/ufo/parameters.rs:115`) does
two things with a restrict card:

- it takes every value as the parameter's default;
- it locks a parameter only when its value is zero (`:125`).

MadGraph also fixes parameters the restrict card sets to exactly `1`.

The gap is latent. On 2026-10-06, no restrict card in the repository set a
non-`QNUMBERS` parameter to 1:

- the SM `sm_assets` cards;
- the SMEFTsim cards;
- `validation/madgraph/cards/smeft/restrict_vg_*`;
- the toy UFOs' cards.

A user-supplied card with a Wilson coefficient at 1 would diverge from MadGraph
if a later param card moved that coefficient.

Read MadGraph's exact rule from `import_ufo.py` (`RestrictModel`) before
porting. That includes whether it also merges identical values.

The zero-locking precedent and the falsifier pattern
(`restricted_defaults_are_madgraphs_generated_param_card`) are in
[note 35 §C](../../35-ufo-lorentz-sprint-plan.md).
