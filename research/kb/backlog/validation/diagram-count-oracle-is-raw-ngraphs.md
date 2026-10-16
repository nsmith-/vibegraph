---
type: Backlog Item
title: The diagram gates compare raw counts with MadGraph's NGRAPHS
description: "Diagram cells gate on count equality with NGRAPHS, which cannot hold in general; they should compare under a declared, machine-checked diagram mapping instead."
area: validation
state: open
priority: medium
closes_when: "Every diagram-count comparison against MadGraph holds under a declared, machine-checked mapping (our diagrams grouped into MadGraph graphs, or MadGraph AMP()s grouped into our diagrams, as each convention requires) rather than by count equality, and gg_to_gg / gg_to_gg_cg's diagrams cells are gated under that mapping instead of informational."
blocked_by: []
opened: 2026-10-15
tags: [diagrams, oracle, madgraph-parity, counting-convention]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-15}
sources:
  - {id: user, resource: "https://github.com/nsmith-/vibegraph/pull/35", title: "PR #35: the user's direction on the diagram-count oracle (2026-10-15)"}
  - {id: diagrams, resource: "../../../../vibegraph-lib/tests/validate_madgraph_diagrams.rs", title: "validate_madgraph_diagrams: summed NGRAPHS per P-class"}
---
Exact diagram-count parity with MadGraph is not achievable in general (user's
conclusion, 2026-10-15). Four-gluon contacts give 27 against our 21: MadGraph
writes one `AMP()` per colour structure. Four-fermion pairings give 19 against
our 22: MadGraph folds them into one `AMP()`. Raw `NGRAPHS` equality is
therefore the wrong oracle. Replacing it is a change of oracle, not a widened
tolerance: the comparison becomes stricter, a one-to-one account of every
diagram on both sides.

**Where raw counts are compared today:**
- `tests/validate_madgraph_diagrams.rs`: the 64 manifest `diagrams` cells,
  against `diagrams.json`'s `NGRAPHS` summed per P-class. `gg_to_gg` (6 against
  4) and `gg_to_gg_cg` (27 against 21) are `info` for exactly this reason.
- `tests/smeftsim.rs::gated_row_diagram_counts_match_madgraph`, which follows
  the same manifest modes.
- `tests/amplitude_oracle.rs` (~:1011), which reports a count mismatch against
  `NGRAPHS` as its own kind of disagreement.

**The replacement.** A per-row declared mapping, checked by machine, in whichever
direction the row's convention needs:
- our diagrams grouped into MadGraph graphs (spin flows: four-fermion pairings);
- MadGraph `AMP()`s grouped into our diagrams (colour flows: four-gluon contacts).

The check is that the mapping is a partition of both sides and, where per-diagram
values exist, that each group's sum matches. Two such mappings already exist and
show the shape: `standalone_jamps`' `MG_GRAPH_DIAGRAMS` (PR #35, diagram → graph,
checked both ways) and `amplitude_oracle`'s `MG_DIAGRAM_ORDER`. A row with no
per-diagram values can still check that the declared mapping is a partition
consistent with both counts. Declaring each pairing once, for every gate that
reads it, is [mg-diagram-pairing-declared-twice](mg-diagram-pairing-declared-twice.md).

The evaluator-side counterpart, attributing every partial sum to one Feynman
diagram, is
[amplitude-partial-sums-attributed-to-diagrams](../feature/amplitude-partial-sums-attributed-to-diagrams.md).
