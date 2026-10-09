---
type: Backlog Item
title: Same-flavour four-fermion contact processes are ungated
description: "`e+ e- > e+ e- NP<=1` makes one diagram per fermion pairing at a four-fermion contact where MadGraph draws one, and no banked row checks the count, the signs or |M|²."
area: validation
state: open
priority: medium
closes_when: An `ee_to_ee_4f` SMEFTsim row is banked and its |M|² (per diagram and per helicity) gates against MadGraph, with the diagram-count difference recorded as a convention cell.
blocked_by: []
opened: 2026-09-07
tags: [non-sm-ufo, smeft, four-fermion, diagram-count, madgraph-oracle]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-06}
sources:
  - {id: todo, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L935-L937", title: "TODO.md entry T083"}
---
Take a same-flavour four-fermion process such as `e+ e- > e+ e- NP<=1`. At the
`O_ll`-type contact, vibegraph enumerates one diagram per fermion pairing.
MadGraph draws one diagram per contact.

This is the counting-convention class of `gg_to_gg_cg`, which is an `info`
cell: 21 against 27 under `NGRAPHS` ([note 35 §10.1](../../history/notes/35-ufo-lorentz-sprint-plan.md)).
The difference should be harmless if the pairings' relative Fermi signs are
right. Nothing checks that, because the banked four-fermion rows
(`ee_to_mumu_4f`, `uux_to_ttx_4f`) all have distinct flavours. With distinct
flavours each contact has only one pairing.

A same-flavour row is the first where the relative sign between the two pairings
of one contact reaches |M|². [Note 35 §F1](../../history/notes/35-ufo-lorentz-sprint-plan.md)
describes the four-fermion pairing and permutation-sign design.

Area: this is a gate gap, so it is filed under validation, not feature.
