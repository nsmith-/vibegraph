---
type: Backlog Item
title: Group every amplitude contribution of one Feynman diagram together, as MadGraph does
description: "A four-fermion contact is split by spinor pairing and a four-gluon contact kept whole, both unlike MadGraph, so diagram counts and per-diagram tables differ in opposite directions."
area: feature
state: open
priority: low
closes_when: "A diagram is one Feynman graph with every contribution attributable to it (all spinor pairings of a contact, all colour structures) grouped under it, so vibegraph's per-diagram list and count match MadGraph's graphs, and the per-pairing and per-colour-structure pieces remain addressable for rooting and signs."
blocked_by: []
opened: 2026-10-15
tags: [diagrams, enumeration, four-fermion, contact-vertex, madgraph-parity]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-15}
sources:
  - {id: user, resource: "https://github.com/nsmith-/vibegraph/pull/35", title: "PR #35: the same-flavour four-fermion counting convention, user request for this item (2026-10-15)"}
  - {id: q1, resource: "../../sprints/amplitude-signs/sessions/Q1-report.md", title: "Q1 report: 22 against 19 on ee_to_ee_4f"}
  - {id: topo, resource: "../../../../vibegraph-lib/src/ufo/topo.rs", title: "ufo/topo.rs build_feyngraph_model: one feyngraph vertex per pairing (~:127-176)"}
---
Two counting conventions, in opposite directions:

- **Four-fermion contacts** (`ee_to_ee_4f`, PR #35): vibegraph emits one diagram
  per spinor pairing, 22 against MadGraph's 19. The split is vibegraph's own,
  made in `ufo/topo.rs::build_feyngraph_model` (one feyngraph vertex `@0`/`@1`
  per pairing), not feyngraph's enumeration. feyngraph's one-`spin_map`-per-vertex
  `add_vertex` made the split the easy route; the reason it is kept is that each
  diagram then has one set of fermion lines, one Fermi sign, one wavefunction
  typing (`crossed` vs mixed) and one rooting. MadGraph instead builds on a
  canonical pairing and folds the other into the same `AMP()` with the
  `get_sign_flow` sign (note 35 §F1).
- **Four-gluon contacts** (`gg_to_gg_cg`, 21 against 27; `gg_to_gg`, 4 against
  6): vibegraph keeps one diagram per contact where MadGraph writes one `AMP()`
  per colour-ordered structure (manifest notes).

Grouping by Feynman graph would need either MadGraph's canonical-pairing-plus-
coupling-sign construction, or per-pairing sub-sums inside one diagram that keep
each sub-sum's own fermion lines and sign. Either is an evaluator change. Today
the standalone per-graph gate (`MG_GRAPH_DIAGRAMS`) checks that the declared
groups partition vibegraph's diagrams into exactly MadGraph's graphs, which
pins the mapping without changing the convention.

Related, but about subprocess sets rather than diagrams within one:
[per-flavour-diagram-union-unmatched](../validation/per-flavour-diagram-union-unmatched.md).
