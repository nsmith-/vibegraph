---
type: Derivation
title: Rooting invariance, the diagram anchor and canonical form
description: "Graph-property signs live on Diagram; kernel-compensation signs are read at Diagram::anchor (first lowest-arity vertex), so currents are rooting-invariant; canonical form and container equality."
status: draft
tags: [rooting, sign-conventions, diagrams, anchor, canonical-form]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-09}]
sources:
  - {id: n19-v5, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/19-validation-pass-plan.md#L148-L710", title: "Note 19 §V5 (rooting-soundness: the sign derivation)"}
  - {id: n38-container, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/38-process-grammar-sprint-plan.md#L254-L291", title: "Note 38 §3.3 (the diagram container is the oracle boundary)"}
  - {id: n38-s1, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/38-process-grammar-sprint-plan.md#L362-L432", title: "Note 38 §4 S1 (signs to the diagrams stage)"}
  - {id: n38-d2, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/38-process-grammar-sprint-plan.md#L599-L688", title: "Note 38 §4 D2 (decay-chain stitching; the canonical-form correction)"}
  - {id: n38-decisions, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/38-process-grammar-sprint-plan.md#L1144-L1201", title: "Note 38 §5 (decisions)"}
  - {id: rooting-study, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/rooting-study-results.md#L233-L266", title: "Rooting-exploration study, findings"}
  - {id: code-diagram, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/vibegraph-lib/src/diagrams/diagram.rs", title: "Diagram::anchor, canonical, fermion_line_sign, fermion_pairing_sign"}
  - {id: code-rootdiag, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/vibegraph-lib/src/helas/eval/root_diagram.rs", title: "compile_single_diagram, canonical_root, choose_root"}
  - {id: code-soundness, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/vibegraph-lib/src/helas/eval/rooting_soundness.rs", title: "all_rootings_preserve_amplitude"}
  - {id: code-renumber, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/vibegraph-lib/src/helas/eval/renumbering.rs", title: "renumbering_preserves_signs_and_amplitudes"}
---

# Rooting invariance, the diagram anchor and canonical form

A tree diagram is evaluated by rooting it at one vertex: that vertex closes the
amplitude, and every other vertex produces an off-shell current toward it. The
choice of root orients the internal lines but must not change the physics.
Vibegraph makes that true by splitting every sign a diagram carries into
three kinds and putting each where it belongs:

| class | what it depends on | where it lives |
|---|---|---|
| **a. graph property** | the diagram's lines only | `Diagram::sign` (diagrams stage) |
| **b. kernel compensation** | (diagram, the rooting at which the kernels are read) | read at `Diagram::anchor`, folded into `fermi_sign` |
| **c. numbering** | vertex indices | none: eliminated |

The honest currents carry no convention sign, so they are rooting-invariant
tensors. Every class-b sign is read off one fixed rooting, the anchor's, so
the product is the same whichever vertex production actually roots at.
Production roots elsewhere for performance (below).

## The anchor

`Diagram::anchor`: of the vertices with the fewest rays, the first in canonical
order[^code-diagram]. On a diagram built only from three-point vertices it is
the vertex external leg 0 attaches to; a lower-arity vertex elsewhere
displaces a four-point vertex that holds leg 0. It reads only the labelled
external legs and slot order, so every numbering of a diagram has the same
anchor.

The rule reproduces feyngraph's own vertex 0, which the convention signs were
originally calibrated against. "The vertex leg 0 attaches to" does **not**: on
a census of 2517 diagrams it agreed in 2475, and the other 42
(`g g > g g g` 6, `g g > t t~ g` 1, `W+ W- > W+ W- Z NP<=1` 35) put leg 0 on a
four-point contact while feyngraph numbers a three-point vertex first. The sign
product differs between the two choices on 53 (diagram, chain) pairs, so the
choice carries physics and leg 0's vertex was not adopted[^n38-s1].

## Class a: signs on the diagram

`Diagram::sign` is `fermion_pairing_sign × fermion_line_sign`, both functions
of the graph[^code-diagram]:

- `fermion_pairing_sign`: the parity of the external fermions paired by line
  (feyngraph's `view.sign()`).
- `fermion_line_sign`: the relative sign the pairing parity leaves out, one
  factor per fermion line, from the line's ends and its internal-propagator
  count. A line with an initial-state end takes a −1 per internal fermion
  propagator; a crossed (final–final) line takes one −1 regardless of
  propagators. The derivation and its MadGraph pins are
  [fermion-line-sign](../amplitudes/fermion-line-sign.md).

A stitched decay-chain diagram rebuilds both from the stitched graph rather
than multiplying the parts' signs. The naive product passes every case where
insertion keeps the pairing parity and fails on `g b > w- t, t > w+ b`, where
the initial `b` line gains the `t` propagator; that is a global sign, visible
only to container equality and per-diagram amplitudes, never to |M|²
([process/decay-chains](../process/decay-chains.md))[^n38-d2].

The line sign used to be read off the rooted tree (`spine_sign_from_flow`). It
is now a graph function, and `compile_single_diagram` keeps the tree
derivation as a **debug-build cross-check on the live tree**:
`debug_assert_eq!(spine_sign_from_flow(&tree), diagram.fermion_line_sign(model))`.
Measured on a census of every rooting of 2553 (diagram, chain) pairs, the tree
derivation never varied with the rooting. It does count every closed line,
including one closed at a four-fermion current rooted at a fermion leg, which
it once skipped and where the cross-check first fired[^n38-s1].

## Class b: kernel compensations, read at the anchor

`compile_single_diagram` (`helas/eval/root_diagram.rs`) assembles[^code-rootdiag]:

```rust
let fermi_sign = diagram.sign
    * fermion_current_line_sign(reference)
    * yang_mills_vvv_sign(diagram, model)
    * vector_contact_sign(diagram, model)
    * gluon_scalar_current_sign(diagram, model)
    * reference.build_convention_sign()
    * reference.reversed_convention_sign()
    * tree.reversed_convention_sign();
```

`reference` is the tree rooted at the anchor: the live tree itself when the
production root is the anchor, otherwise a second tree built only for its
signs.

- **`build_convention_sign`**: per vertex, the ±1 that `build_at_leg` picks up for
  the role the vertex plays at that rooting (the VVS `pure_metric` −1, the FFS
  scalar-sink −1, the crossed-pair −1, `standalone_projector_crossed`). It is
  kept on `RootedTerm::build_sign`, not folded into the term's `coeff`.
- **`reversed_convention_sign`**: per vertex, the parity of the runtime
  `reversed` flag at a fermion→vector sink (`GammaVout`/`FfvVout`). The runtime
  `resolve_bra_ket` still applies the **live** parity, which keeps `ffv_vout`'s
  `g_L ↔ g_R` swap; multiplying by the live parity and then by the reference
  parity leaves the reference one. In production, when live = reference, the
  factor is `+1` and nothing changes.
- **The vector-vertex signs** (`yang_mills_vvv_sign`, `vector_contact_sign`,
  `gluon_scalar_current_sign`) read the anchor and the anchor-rooted output of
  each vertex directly from the graph. See
  [vector-vertex-signs](../amplitudes/vector-vertex-signs.md).
- **`fermion_current_line_sign`** divides line signs back out where a line
  closes at a cyclic four-fermion (tensor-path) current at the anchor rooting.
  See [four-fermion-vertices](../amplitudes/four-fermion-vertices.md).

On the census, three class-b factors vary with which vertex is the reference
root: the Yang–Mills source sign on 214 of 2553 pairs, the build sign on 92,
the reversed sign on 12 (their product on 234). That variation is exactly what
the anchor pins. The full inventory, with which gate covers which sign, is
[convention-sign-inventory](../amplitudes/convention-sign-inventory.md).

## Why reading at one fixed rooting is enough

This is the derivation behind the design[^n19-v5].

1. **Factorability.** Every re-rooting changes each diagram's amplitude by
   exactly a global ±1, never by a value change. This was swept over every
   diagram of every ≤ 60-diagram process at every rooting and every helicity
   (1583 checks, 0 shape violations), and over `b b~ > c c~ e+ e- mu+ mu-`
   (96 480 checks, 0 violations). So the honest tensor value is
   rooting-invariant and a per-diagram scalar can absorb the rest.
2. **The sign is per vertex current, not per term.** A chiral Yukawa has two
   Lorentz terms (`ProjM` + `ProjP`). Re-rooting it from an off-shell fermion
   current to a scalar sink flips *both*, so a product over terms reads
   `(−1)² = +1` and misses the flip. The sign is applied once per vertex, after
   asserting that all of the vertex's terms agree (`VertexInfo::build_sign`
   panics otherwise). No non-uniform vertex was found.
3. **The sign is per vertex *role*, not per propagator.** In
   `b b~ > c c~ e+ e- mu+ mu-` the same `H` propagator is VVS-produced in two
   diagrams. In one it is consumed as a fermion current (net −1, the VVS
   scalar-out sign alone); in the other it is consumed by the `bbH` Yukawa at
   the amplitude root, which fires its own scalar-sink −1 on top (net +1). No
   `(−1)^{#scalar propagators}` rule, at either end, reproduces both. The
   `u u~` analogue hides this because its `H` diagrams are ~0 (tiny u Yukawa),
   which is why the massive-`b` process is the sharp oracle.
4. **The reversed parity is a per-diagram scalar, not a per-node flag.** A
   vertex is a `GammaVout` sink (which carries `reversed`) under one rooting and
   a fermion-continuing `GammaIout`/`GammaOout` (which does not) under another,
   so a live node has no canonical counterpart to copy a flag from.
5. **Moving a ±1 off a term onto `fermi_sign` is bit-exact.** Multiplying by ±1
   does not round, and `(−a) + (−b) = −(a + b)` holds exactly in IEEE
   arithmetic. The extraction changed no amplitude bit.

What re-rooting does still change is the association order of momentum sums
(`POut = −Σ inputs`), so a correct re-rooting differs at the 1e-11 level (worst
2.2e-11 on the census), which is why the gate tolerance is 1e-10.

## Production rooting

Production roots each diagram at `canonical_root`: the vertex with the fewest
directly attached external legs, ties toward the lowest vertex index. This is
not the anchor. A deep, few-external root keeps sub-currents closer to the
`(edge, direction)` signatures that cross-diagram CSE deduplicates, while
rooting at a high-external hub duplicates them[^code-rootdiag]. Its node and
traffic savings, and what the rejected alternatives cost, are
[performance/diagram-rooting](../performance/diagram-rooting.md).

Before the class-b signs were lifted to the anchor, every node-reducing
rooting gave a silently wrong amplitude (max_rel up to 1.7e3 on
`e+ e- > w+ w-`, `e+ e- > ta+ ta- h` and every ≥ 6-point QCD=0 process), and
the rule was "never re-root". That rule is retired: the soundness gate passes,
and production roots off the anchor[^rooting-study].

The root's tie-break still reads vertex indices, so two numberings of one
diagram agree to rounding (≤ 1e-10), not bit for bit.

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
filtered full final state, not by amplitudes. That works because every sign
that is a property of the diagram now lives on the container, and `helas` keeps
only signs that compensate its own kernels[^n38-container][^n38-decisions].

**Blind spots of container equality:**

- Slot order is part of the identity, so two containers binding identical
  particles to a symmetric vertex's slots in a different order compare
  unequal. This is a false inequality, never a false equality. It occurs on
  `e+ e- > w+ w- z, z > mu+ mu-`, where the forced `Z` and an s-channel `Z` sit
  in the two `Z` slots of `W W Z Z` / `H Z Z` in the other order; the stitching
  test pairs those by a slot-order-free description.
- A sign convention that is a wrong *function of the graph* passes
  renumbering and container equality alike. Only the amplitude oracles see it.

## Gates

| gate | checks | cannot see |
|---|---|---|
| `helas::eval::rooting_soundness::all_rootings_preserve_amplitude` (`#[ignore]`, slow full sweep) | re-roots diagrams (per diagram for ≤ 40-diagram processes, whole-process for the 2→6 rows) and asserts \|M\|² equals the unoverridden baseline at `REL_TOL = 1e-10` | a sign that is wrong but rooting-invariant; the baseline is pinned to MadGraph separately |
| `helas::eval::renumbering::renumbering_preserves_signs_and_amplitudes` | renumbers every census diagram twice (once at random, once with the anchor's vertex forced off index 0); exact `fermi_sign` per chain, same anchor and canonical form; per-helicity per-flow amplitudes to 1e-10 on subprocesses of at most 40 diagrams (`AMP_MAX_DIAGRAMS`) | a wrong function of the graph |
| the debug-build `spine_sign_from_flow == fermion_line_sign` assertion | the graph line sign against the tree derivation, on every compiled diagram | release builds (it is a `debug_assert`) |
| `tests/amplitude_oracle.rs` | the anchor-rooted signs against MadGraph, per diagram and per flow | see [validation/amplitude-oracle](../validation/amplitude-oracle.md) |

Run the soundness sweep with

```
RUST_MIN_STACK=134217728 cargo test -p vibegraph-lib --lib \
  helas::eval::rooting_soundness::all_rootings_preserve_amplitude \
  -- --ignored --nocapture --test-threads=1
```

Mutation: dropping the line sign from `Diagram::sign` fails nine
`amplitude_oracle` rows[^n38-s1][^code-soundness][^code-renumber]. How the
whole compile pipeline is laid out is
[evaluator-architecture](../amplitudes/evaluator-architecture.md).

[^n19-v5]: Note 19 §V5, locus (b): factorability, the per-vertex-current refinement, the `b b~` counter-example and the reversed-parity design. V5 read the signs at feyngraph's `VtxIdx(0)`; they are now read at `Diagram::anchor`, and V5's `σ_V` rule is superseded by the vector-vertex signs. Its later remark that `spine_sign_from_flow` "can flip on re-rooting" describes the tree derivation as it then stood; the current derivation is debug-asserted equal to the graph's line sign on every live tree.
[^n38-container]: Note 38 §3.3.
[^n38-s1]: Note 38 §4 S1: the census, the class a/b/c split, the anchor rule, the canonical form, and the renumbering gate.
[^n38-d2]: Note 38 §4 D2: stitched signs rebuilt from the graph; the `renumbered`/`canonical` particle-conjugation correction.
[^n38-decisions]: Note 38 §5: "the stitching oracle is diagram-container equality; sign resolution moves to the diagrams stage".
[^rooting-study]: Rooting-exploration study, findings 1, 2 and 4. Its "every node-reducing rooting breaks the amplitude; do not promote a rooting pass" is superseded.
[^code-diagram]: `vibegraph-lib/src/diagrams/diagram.rs`: `anchor`, `canonical`, `CanonicalDiagram`, `fermion_pairing_sign`, `fermion_line_sign`.
[^code-rootdiag]: `vibegraph-lib/src/helas/eval/root_diagram.rs`: `compile_single_diagram`, `canonical_root`, `choose_root` (and the test-only `set_root_override`).
[^code-soundness]: `vibegraph-lib/src/helas/eval/rooting_soundness.rs`.
[^code-renumber]: `vibegraph-lib/src/helas/eval/renumbering.rs`.
