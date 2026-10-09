---
type: Paper
title: COMIX
description: "arXiv:0808.3674 (Gleisberg, Höche): colour-dressed Berends-Giele recursion, exponential rather than factorial in the number of legs; Sherpa's ME generator and the alternative to per-diagram HELAS."
resource: "https://arxiv.org/abs/0808.3674"
status: draft
tags: [comix, berends-giele, sherpa, paper, matrix-elements]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-09}]
sources:
  - {id: n01-comix, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/01-paper-summaries.md#L253-L295", title: "Note 01, COMIX summary"}
  - {id: n00-refs, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/00-overview.md#L57-L69", title: "Note 00, references"}
---

COMIX (Gleisberg and Höche, JHEP 0812 (2008) 039) is the high-multiplicity
tree-level matrix-element generator inside Sherpa. ar5iv does not render this
paper; read the arXiv PDF[^n01-comix].

## Berends–Giele recursion

Instead of enumerating Feynman diagrams, COMIX builds off-shell currents. The
current `J(S)` for a set `S` of external legs is the sum of every sub-diagram
joining those legs to one off-shell leg, and it is built from smaller ones:

```text
J(S) = Σ over partitions S = A ∪ B of  V · J(A) · J(B)
```

with `V` the vertex factor (and a three-way split for four-point vertices).
The amplitude is the current for all legs but one, contracted with the last
external wavefunction. The number of recursion steps grows like `3ⁿ`, against
the `n!`-like growth of the number of diagrams, which pays off from about six
or more external legs.

Currents are **colour-dressed**: they carry explicit colour indices, so the
colour sum is done over external colour assignments (summed or sampled) at the
end rather than through a colour matrix. Helicities are handled by computing
currents per helicity configuration.

| | HELAS / MadGraph | COMIX / Berends–Giele |
|---|---|---|
| unit | one Feynman diagram | one recursive current |
| growth | `n!`-like (diagrams) | `3ⁿ` (recursion steps) |
| code | per-diagram routines | one recursive engine |
| suits | low multiplicity (2 → 2 to 2 → 4) | high multiplicity (2 → 6 and up) |

## Relevance to vibegraph

vibegraph follows HELAS and MadGraph: one amplitude per diagram, colour through
MadGraph's partial amplitudes and colour-factor matrix
([MadGraph colour factorization](../../amplitudes/madgraph-colour-factorization.md)).
That choice is what lets every amplitude be checked diagram by diagram against
MadGraph ([amplitude oracle](../../validation/amplitude-oracle.md)). The
evaluator does share sub-currents across diagrams through hash-consing, which
recovers part of what the recursion shares
([evaluator architecture](../../amplitudes/evaluator-architecture.md)). Sherpa's
implementation is [the Sherpa survey](../codebases/sherpa.md); the side-by-side
is [generators compared](../codebases/generator-architectures-compared.md).

[^n01-comix]: Note 01, COMIX summary; `research/refs/fetch-papers.sh` records the ar5iv failure.
