---
type: Derivation
title: Rooting invariance, the graph-read signs and canonical form
description: "Every sign a diagram carries reads the graph alone (Diagram::sign and convention_sign) and the live tree's reversed-bilinear parity is cancelled, so every rooting and every leg labelling gives one amplitude; canonical form and container equality."
status: draft
tags: [rooting, sign-conventions, diagrams, relabelling, canonical-form]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-15}
sources:
  - {id: b-report, resource: "../sprints/amplitude-signs/sessions/B-report.md", title: "B′ report: the convention sign is a product of local constants, read off the graph"}
  - {id: code-rootdiag, resource: "https://github.com/nsmith-/vibegraph/blob/5d4a34672e784f8b5fc985e82bd656232ae07caa/vibegraph-lib/src/helas/eval/root_diagram.rs", title: "compile_single_diagram, convention_sign, canonical_root, choose_root"}
  - {id: code-diagram, resource: "https://github.com/nsmith-/vibegraph/blob/5d4a34672e784f8b5fc985e82bd656232ae07caa/vibegraph-lib/src/diagrams/diagram.rs", title: "Diagram::canonical, fermion_line_sign, fermion_pairing_sign"}
  - {id: code-soundness, resource: "https://github.com/nsmith-/vibegraph/blob/5d4a34672e784f8b5fc985e82bd656232ae07caa/vibegraph-lib/src/helas/eval/rooting_soundness.rs", title: "all_rootings_preserve_amplitude"}
  - {id: code-renumber, resource: "https://github.com/nsmith-/vibegraph/blob/5d4a34672e784f8b5fc985e82bd656232ae07caa/vibegraph-lib/src/helas/eval/renumbering.rs", title: "renumbering_preserves_signs_and_amplitudes"}
  - {id: code-relabel, resource: "https://github.com/nsmith-/vibegraph/blob/5d4a34672e784f8b5fc985e82bd656232ae07caa/vibegraph-lib/src/helas/eval/relabelling.rs", title: "relabelling_preserves_per_diagram_amplitudes"}
  - {id: n19-v5, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/19-validation-pass-plan.md#L148-L710", title: "Note 19 §V5 (rooting soundness: factorability and the reversed-parity design)"}
  - {id: n38-container, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/38-process-grammar-sprint-plan.md#L254-L291", title: "Note 38 §3.3 (the diagram container is the oracle boundary)"}
  - {id: n38-s1, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/38-process-grammar-sprint-plan.md#L362-L432", title: "Note 38 §4 S1 (signs to the diagrams stage; the census)"}
  - {id: n38-d2, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/38-process-grammar-sprint-plan.md#L599-L688", title: "Note 38 §4 D2 (decay-chain stitching; the canonical-form conjugation)"}
  - {id: n38-decisions, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/38-process-grammar-sprint-plan.md#L1144-L1201", title: "Note 38 §5 (decisions)"}
---

# Rooting invariance, the graph-read signs and canonical form

A tree diagram is evaluated by rooting it at one vertex: that vertex closes the
amplitude, and every other vertex produces an off-shell current toward it. The
choice of root orients the internal lines but must not change the physics, and
neither may the order a process lists its legs in. Vibegraph makes both true by
reading every sign a diagram carries off the graph alone:

| sign | what it reads | where it lives |
|---|---|---|
| `Diagram::sign` | the diagram's fermion lines | the diagram (diagrams stage) |
| `convention_sign` | each internal line's spin and each vertex's Lorentz structure and spinor bindings | `helas/eval/root_diagram.rs` |
| the live tree's reversed-bilinear parity | the rooted tree production evaluates | cancelled in `fermi_sign` |
| anything keyed to a vertex or propagator index | — | none |

```rust
let fermi_sign =
    diagram.sign * convention_sign(diagram, model, chain)? * tree.live_reversed_parity()?;
```

The kernels build honest currents: no kernel applies a convention sign, so a
rooted current is a rooting-invariant tensor. The one rooting-dependent sign
left at run time is the reversed-bilinear parity `resolve_bra_ket` applies at
the live tree's fermion→vector sinks; the last factor above applies it a
second time, which cancels it[^code-rootdiag].

## Why a graph-read sign is enough

MadGraph's per-graph `AMP()` is rooting-invariant: ALOHA evaluates every vertex
as `COUP · L`, closes the amplitude with `−i` whichever vertex closes it, and
gives each propagator a fixed phase. The kernels here are rooting-invariant
too. So the ratio of the two, diagram by diagram, is a product of one constant
per internal line and one per vertex, and nothing in it refers to a root. The
three factors of `convention_sign` are that product
([convention-sign-inventory](convention-sign-inventory.md)). Each reads only
the graph, so the product is the same at every rooting, under every
renumbering and under every relabelling of the legs[^b-report].

Two measured facts carry the design:

1. **The kernels are rooting-invariant.** Re-rooting changes a diagram's
   amplitude by at most a global ±1, never by a value: swept over every diagram
   of every process of at most 60 diagrams, at every rooting and every helicity
   (1583 checks), and over `b b~ > c c~ e+ e- mu+ mu-` (96 480 checks), with no
   violation. The remaining ±1 is the live reversed parity, which
   `fermi_sign` cancels; `rooting_soundness` measures the product, 270
   re-rootings with 0 failures[^n19-v5][^b-report].
2. **The reversed parity is a per-diagram scalar, not a per-node flag.** A
   vertex is a `GammaVout` sink (which carries `reversed`) under one rooting and
   a fermion-continuing `GammaIout`/`GammaOout` (which does not) under another,
   so a node has no rooting-free counterpart. The parity is therefore cancelled
   at the diagram, and the convention carries each vertex's parity read with
   that vertex as the amplitude root instead (`vertex_root_signs`)[^n19-v5].

Multiplying a diagram by ±1 does not round, so a sign carried on `fermi_sign`
rather than on a term moves no amplitude bit. What re-rooting does change is
the association order of momentum sums (`POut = −Σ inputs`): a correct
re-rooting differs at the 1e-11 level (worst 2.2e-11 on the census), which is
why the rooting gates compare at 1e-10.

## The signs on the diagram

`Diagram::sign` is `fermion_pairing_sign × fermion_line_sign`, both functions
of the graph[^code-diagram]:

- `fermion_pairing_sign`: the parity of the external fermions paired by line
  (feyngraph's `view.sign()`).
- `fermion_line_sign`: one factor per fermion line, from the line's ends and its
  internal-propagator count. A line with an initial-state end takes a −1 per
  internal fermion propagator; a crossed (final–final) line takes one −1
  regardless of propagators. See [fermion-line-sign](fermion-line-sign.md).

A stitched decay-chain diagram rebuilds both from the stitched graph rather
than multiplying the parts' signs. The product of the parts' signs fails on
`g b > w- t, t > w+ b`, where the initial `b` line gains the `t` propagator;
that is a global sign, visible only to container equality and per-diagram
amplitudes, never to |M|² ([process/decay-chains](../process/decay-chains.md))[^n38-d2].

`compile_single_diagram` keeps the rooted-tree derivation of the line sign
(`spine_sign_from_flow`) as a debug-build cross-check on every live tree:
`debug_assert_eq!(spine_sign_from_flow(&tree), diagram.fermion_line_sign(model))`.
On a census of every rooting of 2553 (diagram, chain) pairs the tree derivation
never varied with the rooting. It counts every closed line, including one
closed at a four-fermion current rooted at a fermion leg[^n38-s1].

## Production rooting

Production roots each diagram at `canonical_root`: the vertex with the fewest
directly attached external legs, ties toward the lowest vertex index. A deep,
few-external root keeps sub-currents closer to the `(edge, direction)`
signatures that cross-diagram CSE deduplicates, while rooting at a
high-external hub duplicates them[^code-rootdiag]. The node and traffic savings,
and what the rejected alternatives cost, are
[performance/diagram-rooting](../performance/diagram-rooting.md).

The tie-break reads vertex indices, so two numberings of one diagram can root
differently and agree to rounding (≤ 1e-10), not bit for bit.

## Canonical form and container equality

`Diagram::canonical(model)` renumbers internal vertices in depth-first preorder
from leg 0's vertex, descending through each vertex's rays in slot order. Each
propagator takes the number of the vertex it leads to, less one, and is
oriented with `endpoints[0]` at the end the walk reaches first: its momentum is
negated **and its particle conjugated** where that reverses it.
`Prop::particle` is the particle of the slot at `endpoints[1]`, so a flipped
endpoint pair without the conjugation would make two equal graphs compare
unequal; this is pinned on 2656 census diagrams and 3397 charged
lines[^n38-d2].

`CanonicalDiagram` is `Eq + Hash` on every graph field: legs, propagators with
particles, endpoints, slots, momenta and on-shell flags, vertices with their
interaction, flow group and slot-ordered rays, the sign and the symmetry factor.
`provenance` (which `@N` process and which decay a line came from) is not
compared. Two canonical forms are equal exactly when the diagrams are equal up
to renumbering. The form is idempotent and keeps every census subprocess's
diagrams distinct[^code-diagram].

The diagram container is the oracle boundary between diagram generation and
`helas`: decay-chain stitching is checked by container equality against the
filtered full final state, not by amplitudes. That works because the fermion
signs live on the container, and the only sign `helas` adds, the convention
sign, is a function of the same graph and the model[^n38-container][^n38-decisions].

**Blind spots of container equality:**

- Slot order is part of the identity, so two containers binding identical
  particles to a symmetric vertex's slots in a different order compare
  unequal. This is a false inequality, never a false equality. It occurs on
  `e+ e- > w+ w- z, z > mu+ mu-`, where the forced `Z` and an s-channel `Z` sit
  in the two `Z` slots of `W W Z Z` / `H Z Z` in the other order; the stitching
  test pairs those by a slot-order-free description.
- A sign convention that is a wrong *function of the graph* passes
  renumbering, relabelling and container equality alike. Only the amplitude
  oracles see it.

## Gates

| gate | checks | cannot see |
|---|---|---|
| `helas::eval::rooting_soundness::all_rootings_preserve_amplitude` (`#[ignore]`, slow full sweep) | re-roots diagrams (per diagram for ≤ 40-diagram processes, whole-process for the 2→6 rows) and asserts \|M\|² equals the unoverridden baseline at `REL_TOL = 1e-10`: the kernels' rooting invariance and the cancellation of the live reversed parity | a sign that is wrong but rooting-invariant, which includes every convention-sign error; the baseline is pinned to MadGraph separately |
| `helas::eval::renumbering::renumbering_preserves_signs_and_amplitudes` | renumbers every census diagram twice (once at random, once with leg 0's vertex forced off index 0); exact `fermi_sign` per chain and the same canonical form; per-helicity per-flow amplitudes to 1e-10 on subprocesses of at most 40 diagrams (`AMP_MAX_DIAGRAMS`) | a wrong function of the graph |
| `helas::eval::relabelling::relabelling_preserves_per_diagram_amplitudes` | twelve processes against the same process with its legs listed in another order: every diagram equals its partner (paired by a label-free graph signature) at the permuted momenta and helicities, up to one sign per process | a wrong function of the graph; a dependence on a vertex no listed relabelling moves; multi-flow processes |
| the debug-build `spine_sign_from_flow == fermion_line_sign` assertion | the graph line sign against the tree derivation, on every compiled diagram | release builds (it is a `debug_assert`) |
| `tests/amplitude_oracle.rs`, `tests/standalone_jamps.rs` | the signs against MadGraph, per diagram and per flow | see [validation/amplitude-oracle](../validation/amplitude-oracle.md) |

Run the soundness sweep with

```
RUST_MIN_STACK=134217728 cargo test -p vibegraph-lib --lib \
  helas::eval::rooting_soundness::all_rootings_preserve_amplitude \
  -- --ignored --nocapture --test-threads=1
```

Which gate pins which factor of the convention sign is
[validation/convention-channel-coverage](../validation/convention-channel-coverage.md);
how the whole compile pipeline is laid out is
[evaluator-architecture](evaluator-architecture.md)[^code-soundness][^code-renumber][^code-relabel].

[^b-report]: B′ report: the derivation, the relabelling test's before and after, and the acceptance runs.
[^n19-v5]: Note 19 §V5, locus (b): factorability and the reversed-parity design.
[^n38-container]: Note 38 §3.3.
[^n38-s1]: Note 38 §4 S1: the line-sign census and the renumbering gate.
[^n38-d2]: Note 38 §4 D2: stitched signs rebuilt from the graph; the `renumbered`/`canonical` particle-conjugation correction.
[^n38-decisions]: Note 38 §5: "the stitching oracle is diagram-container equality; sign resolution moves to the diagrams stage".
[^code-rootdiag]: `vibegraph-lib/src/helas/eval/root_diagram.rs`: `compile_single_diagram`, `convention_sign`, `canonical_root`, `choose_root` (and the test-only `set_root_override`).
[^code-diagram]: `vibegraph-lib/src/diagrams/diagram.rs`: `canonical`, `CanonicalDiagram`, `fermion_pairing_sign`, `fermion_line_sign`.
[^code-soundness]: `vibegraph-lib/src/helas/eval/rooting_soundness.rs`.
[^code-renumber]: `vibegraph-lib/src/helas/eval/renumbering.rs`.
[^code-relabel]: `vibegraph-lib/src/helas/eval/relabelling.rs`.
