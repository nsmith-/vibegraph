---
type: Paper
title: MadGraph (Stelzer and Long, 1994)
description: "hep-ph/9401258: the first MadGraph; topology-first diagram generation, particle insertion, and HELAS code emission for any tree-level SM process."
resource: "https://arxiv.org/abs/hep-ph/9401258"
status: draft
tags: [madgraph, diagram-enumeration, helas, paper, topologies]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: n01-mg, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/01-paper-summaries.md#L87-L119", title: "Note 01, original MadGraph summary"}
  - {id: mg5-beyond, resource: "https://arxiv.org/abs/1106.0522", title: "Alwall et al., MadGraph 5: Going Beyond (leg-combination generation, wavefunction reuse)"}
---

The first MadGraph (Stelzer and Long) generated the Feynman diagrams of any
tree-level Standard Model process and wrote [HELAS](helas.md) Fortran to
evaluate them[^n01-mg]. It is the clearest published description of
topology-first diagram generation.

## The algorithm

1. **Topologies.** Start from the single three-point topology and add one
   external leg at a time, to every existing line and to every vertex. This
   gives 1 topology for 3 particles, 4 for 4 and 25 for 5 (counting four-point
   vertices).
2. **Particle insertion.** Assign particle flavours to each topology's lines,
   keeping only assignments every vertex of the model allows and that conserve
   the quantum numbers.
3. **Colour and symmetry factors** per diagram.
4. **Code emission.** Walk each diagram from the external legs inward:
   external wavefunctions (`ixxxxx`, `oxxxxx`, `vxxxxx`), then a vertex call
   per internal line producing an off-shell current, then a final vertex
   returning the amplitude as a complex number.

MadGraph 5 replaced the topology-first step with leg combination over the
model's interactions, with `DiagramTag` deduplication, and reused identical
wavefunction calls across diagrams within one helicity configuration
(Alwall et al., arXiv:1106.0522[^mg5-beyond]; the code is in
[the MadGraph survey](../codebases/madgraph5-amcnlo.md)).

## Relevance to vibegraph

vibegraph enumerates diagrams through [FeynGraph](../codebases/feyngraph.md),
which implements this topology-first, then-insertion scheme in Rust, with
loops and parallelism added. The diagram set it produces must match MadGraph 5's
leg-combination output, which the structural censuses check
([diagram enumeration](../../process/diagram-enumeration.md),
[structural censuses](../../validation/structural-censuses.md)). The inward walk
of step 4 is the evaluation order of vibegraph's per-diagram programs, rooted
at a chosen vertex ([diagram rooting](../../performance/diagram-rooting.md)).

[^n01-mg]: Note 01, original MadGraph summary.
[^mg5-beyond]: arXiv:1106.0522; the notes cite it in a reference list and one sentence of note 15 §1.1.
