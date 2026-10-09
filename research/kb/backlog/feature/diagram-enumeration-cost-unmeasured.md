---
type: Backlog Item
title: Diagram enumeration's share of run time is unmeasured
description: Nobody has measured whether feyngraph's topology-first enumeration is ever on the critical path, which decides whether a MadGraph-style enumerator is worth building.
area: feature
state: open
priority: low
closes_when: feyngraph's share of integrate wall time on the widest cards is measured against MadGraph's generation time for the same cards, and either a leg-combination enumerator passes the census and amplitude-oracle gates or the study is recorded as not worth pursuing.
blocked_by: []
opened: 2026-09-06
tags: [research, diagrams, feyngraph, enumeration]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-06}
sources:
  - {id: todo, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L615-L631", title: "TODO.md entry T056"}
---
feyngraph enumerates topology-first (QGRAF-style orderly generation, then
particle assignment by backtracking). MadGraph 5 recursively combines
external-leg subsets through the vertex table, so it never visits a shape the
model cannot fill and prunes coupling orders in the recursion (arXiv:1106.0522).
`docs/src/guide/03-diagrams.md` records the contrast.

**Answer before any code:** measure feyngraph's share of `integrate` wall
time on the widest cards against MadGraph's own generation time.

Only if enumeration is on the critical path: a leg-combination enumerator over
`ufo::topo`'s vertex table producing `diagrams::Diagram` unchanged (same
slot-ordered rays, routing, Fermi sign, symmetry factor), a canonical diagram
tag for duplicate elimination, the WEIGHTED bound applied in the recursion, and
subprocess reuse across flavour relabellings. Gate: an identical census against
`validation/madgraph/diagrams.json` and byte-identical amplitude-oracle rows,
since a reordered diagram set changes the rooting and so the arithmetic.
