---
type: Algorithm
title: Diagram enumeration through feyngraph
description: "Topology-then-assignment generation via feyngraph and what vibegraph adds (aliases, s-channel filters, own model); comparison with MadGraph's leg combination, DiagramTag and NGRAPHS counting."
status: draft
tags: [diagrams, feyngraph, enumeration, madgraph, ngraphs]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: n01-sl, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/01-paper-summaries.md#L94-L106", title: "Note 01, Stelzer and Long: the diagram enumeration algorithm"}
  - {id: n02-fg, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/02-reference-implementations.md#L160-L283", title: "Note 02, FeynGraph Goal 2: diagram enumeration (read at 1dc4ea7)"}
  - {id: n02-mg, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/02-reference-implementations.md#L357-L409", title: "Note 02, MadGraph Goal 2: diagram enumeration"}
  - {id: n02-cross, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/02-reference-implementations.md#L528-L555", title: "Note 02, FeynGraph as a dependency; MadGraph vs FeynGraph"}
  - {id: n06-gap, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/06-process-grammar.md#L460-L544", title: "Note 06 §8, feyngraph gap analysis and model construction"}
  - {id: n25-findings, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/25-validation-layering-plan.md#L678-L719", title: "Note 25 §10, findings register (g g > g g counting decision)"}
  - {id: n35-ngraphs, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/35-ufo-lorentz-sprint-plan.md#L1494-L1515", title: "Note 35 §10.7, gg_to_gg_cg 21/27 under the NGRAPHS convention"}
  - {id: mg-generate-diagrams, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/madgraph/core/diagram_generation.py#L520", title: "MadGraph diagram_generation.py Amplitude.generate_diagrams"}
  - {id: mg-diagramtag, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/madgraph/core/diagram_generation.py#L46", title: "MadGraph diagram_generation.py DiagramTag"}
  - {id: code-enum, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/vibegraph-lib/src/diagrams/mod.rs#L690-L960", title: "vibegraph-lib/src/diagrams/mod.rs generation loop"}
---

vibegraph enumerates tree-level Feynman diagrams with the `feyngraph` crate
(`Cargo.toml`: git rev `fd5aa83`, default features off) and keeps everything around that
call: the model, the process grammar, alias expansion, the order search, the s-channel
filters and the decay-chain stitching. feyngraph's own API and source layout are surveyed
in [FeynGraph](../references/codebases/feyngraph.md); this concept is how vibegraph uses
it.

## Two ways to enumerate

**feyngraph: topology first, then assignment.** It generates every graph topology with
`n` external legs (valid node-degree partitions, an adjacency-matrix DFS, bridge finding
for momentum routing), then assigns particles to each topology by backtracking over the
model's vertices, in parallel over topologies with rayon. Symmetry factors come from the
topology stage, and loop topologies are available (`n_loops`). [^n02-fg]

**MadGraph: leg combination.** `Amplitude.generate_diagrams` (`diagram_generation.py:520`)
[^mg-generate-diagrams] [^n02-mg]:

1. build interaction dictionaries for n → 0 (amplitude) and n → 1 (propagator) vertices;
2. mark the external legs, flipping incoming particles to their antiparticles;
3. combine groups of legs through the n → 1 dictionary (`reduce_leglist`), replacing each
   group by a new off-shell leg (`merge_comb_legs`);
4. repeat until at most two legs remain, then close with an n → 0 vertex.

Every vertex is stored with the outgoing convention and incoming legs are flipped back on
use. Duplicates are removed by hashing a `DiagramTag` (L46) [^mg-diagramtag]; the
off-shell output of a vertex is found by removing the input PDG codes from the
interaction's particle list. The original MadGraph (Stelzer and Long) generated topologies
by adding one leg at a time, to each leg and each vertex (1, 4 and 25 topologies for 3, 4
and 5 particles), then inserted particles [^n01-sl]; see
[MadGraph (Stelzer and Long)](../references/papers/madgraph-stelzer-long.md).

| | MadGraph `Amplitude` | feyngraph |
|---|---|---|
| Approach | leg combination (Dyson–Schwinger style) | topology first, then particle assignment |
| Duplicate removal | `DiagramTag` hashing | unique by construction; symmetry factors at the topology stage |
| Loops | no (aMC@NLO is separate) | yes |
| Parallel | no | yes |

Note 02 judged the leg-combination approach simpler to write from scratch and the
topology-first one better for loops and high multiplicity. [^n02-cross] Whether
enumeration is ever on the critical path, which would decide whether a MadGraph-style
enumerator is worth building, is unmeasured
([backlog](../backlog/feature/diagram-enumeration-cost-unmeasured.md)); one known cost is
an allocation per candidate vertex in feyngraph's assignment
([backlog](../backlog/performance/feyngraph-assign-counts-allocation.md)).

## How vibegraph drives feyngraph

feyngraph takes particles **by name** and has no process-string parser, so everything from
the card to a list of names is vibegraph's ([proc-card grammar](proc-card-grammar.md)).
[^n06-gap] The model is built by vibegraph too: `ufo/topo.rs` constructs feyngraph's
`Model` through `Model::empty()` and its mutation API, and feyngraph's own UFO parser is
never called ([UFO parsing](../model/ufo-parsing.md)).

The generation path (`diagrams/mod.rs`) [^code-enum]:

1. **Topologies once.** `generate_topologies` builds every tree topology for the
   process's external-leg count with `TopologyGenerator` and caches it; topology search is
   factorial in the number of internal vertices, so it is not repeated per subprocess.
2. **Concrete subprocesses** from alias expansion, deduplicated and charge-filtered
   ([subprocess enumeration](subprocess-enumeration.md)).
3. **Selectors** (`diagrams/selector.rs`): a coupling constraint becomes
   `select_coupling_power` or `select_coupling_power_list` (`>` enumerates powers up to 20,
   since feyngraph has no unbounded selector); a forbidden particle `/ X` becomes
   `select_propagator_count(X, 0)`; a `WEIGHTED` bound is a custom function summing
   hierarchy × order over the diagram.
4. **Assignment.** `DiagramGenerator::new(in, out, 0, model.topo, selector)
   .assign_topologies(cached)` returns a container of views.
5. **Conversion at the module boundary.** Each view becomes an owned `Diagram`
   (`Diagram::from_view`) and the container is dropped; feyngraph views never leave
   `diagrams/`. The `Diagram` carries legs, propagators with their signed momentum
   combinations and on-shell flags, vertices with their interaction and fermion-flow group,
   the Fermi sign (feyngraph's `view.sign()` pairing parity times the per-line sign), the
   symmetry factor and provenance. Its canonical form and the sign conventions are in
   [rooting invariance and the anchor](../amplitudes/rooting-invariance-and-anchor.md).
6. **s-channel filters** (`>`, `$$`) run on the converted diagrams
   ([s-channel restrictions](s-channel-restrictions.md)).

Without explicit orders this whole pass runs inside the **automatic lowest-`WEIGHTED`
search**: from `(n_ext − 2) × min hierarchy` upward, one pass per bound, until some
subprocess has a diagram or the bound reaches `(n_ext − 2) × max hierarchy`. With explicit
orders it runs once ([coupling orders](../model/coupling-orders.md)).

What feyngraph lacked for MadGraph's process language is all handled in vibegraph now:
alias expansion and subprocess dedup, required and forbidden s-channels (diagram filters on
the converted diagrams, not feyngraph custom functions), `$` (a pointwise integrand, not a
filter), decay chains ([stitching](decay-chains.md)), polarization, and the order search.
A process string with `@N` is metadata that never reaches feyngraph.

## Counting diagrams against MadGraph

Diagram counts are compared with MadGraph's `NGRAPHS` in `matrix<N>_orig.f`
([structural censuses](../validation/structural-censuses.md)). `NGRAPHS` counts one `AMP()`
per **(diagram, colour-ordered contact structure)**: MadGraph writes the four-gluon contact
as one graph per colour structure, where vibegraph writes one diagram whose vertex carries
all three. So `g g > g g` is 4 here against MadGraph's 6 (3 + 1 against 3 + 3), and
SMEFTsim's `gg_to_gg_cg` is 21 against 27. [^n25-findings] [^n35-ngraphs]

**Decision** (note 25's findings register): report vibegraph's count in its own
convention and mark those `diagrams` cells informational (⚠️ in the report). Re-splitting
the enumeration to match a counting convention would change the thing being validated to
make a number match, and `g g > g g` is pinned per colour flow at 8.25e-14, far below what
any difference in diagram content could survive.

A second counting divergence: for a same-flavour process through a four-fermion contact
whose structures pair the legs both ways (`e+ e- > e+ e- NP<=1`), vibegraph emits one
diagram per pairing where MadGraph draws one; no banked row checks it
([backlog](../backlog/validation/same-flavour-four-fermion-diagram-count-ungated.md),
[four-fermion vertices](../amplitudes/four-fermion-vertices.md)).

There is no command to draw or summarise a card's diagrams before integrating
([backlog](../backlog/feature/diagram-inspection-command-missing.md)).

[^n01-sl]: Note 01, Stelzer and Long: topology generation by leg addition, particle insertion.
[^n02-fg]: Note 02, FeynGraph Goal 2: the two-stage algorithm (read at FeynGraph `1dc4ea7`).
[^n02-mg]: Note 02, MadGraph Goal 2: `Amplitude`, the leg-combination algorithm, `DiagramTag`.
[^n02-cross]: Note 02, cross-cutting notes: FeynGraph as a dependency and the comparison table (its `Model::from_ufo` example is not how vibegraph builds the model).
[^n06-gap]: Note 06 §8: feyngraph takes names, selector capabilities, bypassing feyngraph's UFO parser; its gap table predates the s-channel, decay-chain and polarization support.
[^n25-findings]: Note 25 §10 findings register, item 5: the `g g > g g` counting decision.
[^n35-ngraphs]: Note 35 §10.7: `gg_to_gg_cg` 21/27, `gg_to_gg` 4/6 under the `NGRAPHS` convention.
[^mg-generate-diagrams]: `madgraph/core/diagram_generation.py` `Amplitude.generate_diagrams`, L520 (class at L433).
[^mg-diagramtag]: `madgraph/core/diagram_generation.py` `DiagramTag`, L46.
[^code-enum]: `vibegraph-lib/src/diagrams/mod.rs`: `generate_topologies`, the WEIGHTED loop, `generate_sets_inner`.
