---
type: Backlog Item
title: Attribute every amplitude partial sum to exactly one Feynman diagram
description: "Each evaluator partial sum (a four-fermion pairing, a four-gluon colour-structure piece) should carry the Feynman diagram it belongs to, so any value traces to one diagram."
area: feature
state: open
priority: low
closes_when: "Every per-diagram contribution the evaluator computes, per-pairing and per-colour-structure sub-sums included, carries a Feynman-diagram id; the sum over one diagram's contributions is exposed by the evaluator; and a gate compares those per-Feynman-diagram sums against MadGraph through a declared, machine-checked diagram mapping."
blocked_by: []
opened: 2026-10-15
tags: [evaluator, diagrams, attribution, four-fermion, contact-vertex]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-15}
sources:
  - {id: user, resource: "https://github.com/nsmith-/vibegraph/pull/35", title: "PR #35: the user's direction on diagram attribution (2026-10-15)"}
  - {id: q1, resource: "../../sprints/amplitude-signs/sessions/Q1-report.md", title: "Q1 report: 22 against 19 on ee_to_ee_4f"}
  - {id: topo, resource: "../../../../vibegraph-lib/src/ufo/topo.rs", title: "ufo/topo.rs build_feyngraph_model: one feyngraph vertex per pairing (~:127-176)"}
---
The goal is attribution, not MadGraph's grouping. Every amplitude partial sum
should be traceable to the single Feynman diagram it belongs to.

Today the unit is neither MadGraph's nor a Feynman diagram:

- **Four-fermion contacts** (`ee_to_ee_4f`, PR #35). vibegraph emits one diagram
  per spinor pairing, 22 against MadGraph's 19 graphs.
  - The split is vibegraph's own: `ufo/topo.rs::build_feyngraph_model` registers
    one feyngraph vertex (`@0`/`@1`) per pairing, so it does not come from
    feyngraph's enumeration.
  - feyngraph's one-`spin_map`-per-vertex `add_vertex` made the split the easy
    route.
  - The evaluator relies on it: each diagram has one set of fermion lines, so one
    Fermi sign, one wavefunction typing (`crossed` vs mixed) and one rooting.
  - The two pairings of one contact are one Feynman diagram, but nothing marks
    them as such.
- **Four-gluon contacts** (`gg_to_gg_cg`, 21 against 27). vibegraph keeps one
  diagram per contact. Its colour-structure pieces are summed inside it with no
  per-structure attribution, where MadGraph writes one `AMP()` per structure.

**The change.** Keep the per-pairing and per-structure pieces as the evaluator's
units, since they carry the signs and rooting, but tag each with the id of the
Feynman diagram it belongs to, and expose the per-diagram sum. The pieces then
remain addressable for signs, and every value has an owner.

**Diagram counts.** Exact count parity with MadGraph is not achievable in general
(user's conclusion, 2026-10-15). MadGraph splits colour flows into separate
`AMP()`s but folds spin flows into one, and vibegraph does the opposite on both.
How the count gates change accordingly is
[diagram-count-oracle-is-raw-ngraphs](../validation/diagram-count-oracle-is-raw-ngraphs.md).

Related, but about subprocess sets rather than diagrams within one:
[per-flavour-diagram-union-unmatched](../validation/per-flavour-diagram-union-unmatched.md).
