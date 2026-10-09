---
type: Design
title: "Subprocess enumeration: initial-state orderings and outgoing dedup"
description: "The enumerator emits one ordering per unordered initial state (the mirror term supplies the other) and deduplicates on (sorted initial, sorted final), matching MadGraph's subprocess set."
status: draft
tags: [subprocesses, diagrams, alias-expansion, hadronic, madgraph-parity]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: n24-p2, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/24-user-distribution-and-proton-events-plan.md#L943-L1010", title: "Note 24 P2, design decisions: one ordering per initial state, the mirror term"}
  - {id: n28-c1, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/28-kt-spine-feature-sprint-plan.md#L3538-L3624", title: "Note 28 §C.1–§C.2, the p p > j j surplus counted against leshouche.inc and its σ cost"}
  - {id: n28-c2, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/28-kt-spine-feature-sprint-plan.md#L3830-L3917", title: "Note 28 §C2, the dedup key repaired, the set at zero tolerance, blast radius"}
  - {id: code-dedup, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/vibegraph-lib/src/diagrams/mod.rs#L800-L900", title: "vibegraph-lib/src/diagrams/mod.rs generate_sets_inner"}
---

A process line with multiparticle labels (`p p > j j`, `p p > l+ l- j`) stands for many
concrete subprocesses. Which concrete assignments are enumerated, and which count as the
same subprocess, decides the cross section directly: an assignment enumerated twice is
summed twice.

## The expansion

`generate_sets_inner` (`vibegraph-lib/src/diagrams/mod.rs`) takes the Cartesian product of
each leg's particle list, initial side outermost; `p p > e+ e-` yields 9 × 9 = 81
candidates. [^code-dedup] Each candidate then passes through, in order:

1. **Polarization**: a polarized leg is read into helicities for the concrete particle; a
   leg left with none drops the candidate ([polarization](polarization.md)).
2. **Deduplication** on the subprocess key (below).
3. **Charge conservation**, an O(n) check that prunes most alias-expanded candidates before
   the expensive assignment step (about 90% of `p p > q q~ 4l`'s).
4. **feyngraph enumeration** with the coupling-order, forbidden-particle and `WEIGHTED`
   selectors, then the s-channel filters on the converted diagrams
   ([diagram enumeration](diagram-enumeration.md)).

Every surviving candidate is returned as a `DiagramSet`, including those with no
diagrams.

## The subprocess key

```rust
// subprocess_key: per side, the (name, polarization) legs, sorted
(sorted(initial legs), sorted(final legs))
```

The key sorts **both** sides, and only the key: the surviving `DiagramSet` keeps the leg
order the expansion emitted. Three consequences:

**One ordering per unordered initial state.** `g u`, `g u~` and `u u~` are enumerated;
`u g`, `u~ g` and `u~ u` are not. The other beam ordering is not a separate subprocess but
the **mirror term** of the hadronic integrand, which is mandatory and is an identity, not
a symmetry assumption: the mirrored ordering's |M|² is the representative's evaluated with
the **outgoing legs** reflected (`(E, px, py, pz) ↦ (E, px, −py, −pz)`), beams left on
their axis. A group whose beams carry the same parton has no mirror term. Both terms share
one cut indicator, since the final state is the same. The identity, its test and its
visibility bound are in [the beam-mirror identity](../hadronic/beam-mirror-identity.md).
[^n24-p2]

**Outgoing permutations are one subprocess.** `g u > g u` and `g u > u g` are the same
subprocess, as in MadGraph, which keeps one representative per unordered outgoing
assignment. This is sound because dΦ_n is integrated over the whole labelled region and
every run-card cut acts per particle class, so permuting outgoing legs relabels the
integral without moving it. The representative is the first ordering the expansion emits,
which is **MadGraph's own outgoing order**: measured on `p p > j j`, all 65 survivors
list their outgoing legs as `leshouche.inc` does, so no rule for choosing a
representative is needed. [^n28-c2]

**Distinct final-state content never collapses.** A key on the initial state alone would
silently drop subprocesses such as `g d > e+ e- d` whenever the first final state tried for
that initial state (`g d > e+ e- g`) has no diagrams at the active `WEIGHTED` bound.

Polarizations are part of each leg in the key, so `z{0} z{T}` and `z{T} z{0}` are one
subprocess while `z{0} z{0}` and `z{T} z{T}` are two, matching MadGraph's
`tag = zip(prod, polids)`.

## What the key protects, measured

Sorting the final state matters only where a card's final-state slots draw on
**intersecting** alias sets; `p p > j j` is the only manifest row that does (every other
row spells its final state with concrete particles or with pairwise-disjoint labels such as
`l+`, `l-`, `j`). [^n28-c1]

- With the final state keyed **as written**, `p p > j j` enumerates 117 subprocesses:
  MadGraph's 65 (52 with unequal outgoing flavours, 13 equal) plus 52 surplus, every one an
  outgoing swap of a MadGraph assignment, `13 + 2 × 52 = 117`. Each swap is summed over the
  same full-sphere dΦ₂, so σ comes out as σ_MG plus σ_MG restricted to unequal outgoing
  flavours: predicted ratio 1.3354–1.3651 from the banked run's subprocess cross sections,
  measured 1.3628 (pull +155). Collapsing the permutations gave 65 subprocesses and
  σ/σ_MG = 1.0015 (pull +0.66).
- MadGraph's own side: of its 10 000 banked events, 77 distinct emitted flavour
  assignments, none with its outgoing swap also emitted; inside the unequal-outgoing ones
  the two legs are more-forward-first 1808 times and more-forward-second 1814, the even
  split one representative covering the whole sphere predicts.

The standing gate is `jj_subprocesses_are_madgraphs_own` (`validate_hadronic`, enforced):
set equality, at zero tolerance, against the banked run's own `leshouche.inc`, the only
place a run declares which concrete assignments its σ sums over. It also asserts that 52 of
MadGraph's 65 have unequal outgoing flavours and 0 of those have the swap listed, so the
equality cannot hold because there was never a choice to make. A negative control,
`p p > l+ l- j` (`pp_to_llj_fixed`), keeps all 212 enumerated sets under the same key, 24 of
them with diagrams: the rule merges where a label repeats and nowhere else. The `pp_to_jj`
σ cell is gated at `JJ_MAX_REL = 0.005`. [^n28-c2]

**Caveat.** Most `diagrams` cells compare only MadGraph's summed `NGRAPHS` per process, so
outside `p p > j j` no gate checks that the concrete subprocess set is MadGraph's exactly
([backlog](../backlog/validation/per-flavour-diagram-union-unmatched.md)).

## After enumeration

- `p p > l+ l- j QCD=2 QED=2` on `sm-default` gives 24 non-empty subprocesses, 4 diagrams
  each: `l+ l-` expands to e and μ (opposite-flavour pairs have no diagrams), so each initial
  state appears twice. [^n24-p2]
- Subprocesses are grouped for integration by measured |M|² equality, not by a hand-written
  flavour table ([flavour groups](../hadronic/flavour-groups.md)). Before grouping,
  `DiagramSet::with_final_order` puts each subprocess's outgoing legs in the first's order of
  (mass, cut class), which is how `p p > w+ j` and `add process p p > j w-` group.
- Each subprocess is divided by its identical-particle factor, keyed on `(id, polarization)`
  ([identical-particle factor](../phase-space/identical-particle-factor.md)).
- A concrete subprocess reached from two process lines is refused
  ([proc-card grammar](proc-card-grammar.md)).

[^n24-p2]: Note 24, the proton-events design decisions: one ordering per unordered initial state, the mirror term as an identity, grouping by measured |M|², the `llj` subprocess count (the mirror's refined, outgoing-only form is in the beam-mirror concept).
[^n28-c1]: Note 28 §C.1–§C.2: the surplus counted against `leshouche.inc`, MadGraph's sample, the predicted and measured σ ratio.
[^n28-c2]: Note 28, the enumeration repaired: the key that sorts the final state, the set test at zero tolerance, the negative control.
[^code-dedup]: `vibegraph-lib/src/diagrams/mod.rs` `generate_sets_inner` and `subprocess_key`.
