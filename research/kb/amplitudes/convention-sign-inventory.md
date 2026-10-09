---
type: Physics Convention
title: Convention signs in a diagram's sign and what pins each
description: "Every relative-sign arm folded into a diagram's fermi_sign (Wick parity, line, tensor line, VVV, contact, gluon-scalar, build, reversed bilinear), where each is read, and which oracle pins it."
status: draft
tags: [sign-convention, fermi-sign, rooting, amplitudes, madgraph-oracle]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: code-compile, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/vibegraph-lib/src/helas/eval/root_diagram.rs#L1020-L1250", title: "root_diagram.rs: yang_mills_vvv_sign, vector_contact_sign, gluon_scalar_current_sign, compile_single_diagram"}
  - {id: code-build, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/vibegraph-lib/src/helas/eval/root_lorentz.rs#L620-L830", title: "root_lorentz.rs build_at_leg: the per-vertex build-sign arms"}
  - {id: code-guard, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/vibegraph-lib/src/helas/eval/root_diagram.rs#L1393-L1470", title: "channel_counts and mg_guard_processes_exercise_every_convention_channel"}
  - {id: n24-p1, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/24-user-distribution-and-proton-events-plan.md#L665-L755", title: "Note 24 P1: rows enforced per channel; c_i·AMP(i) is the comparable object"}
  - {id: n28-s5s6, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/28-kt-spine-feature-sprint-plan.md#L2346-L2756", title: "Note 28 S5–S6: falsified candidates and the crossing sign rule"}
  - {id: n29-f, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/29-v01-validation-sprint-plan.md#L353-L1152", title: "Note 29 chain F: the pinned-convention inventory and what the dumps pin"}
  - {id: n38-s1, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/38-process-grammar-sprint-plan.md#L362-L432", title: "Note 38 S1: signs to the diagrams stage, the anchor, the tensor-path line rule"}
  - {id: pr13, resource: "https://github.com/nsmith-/vibegraph/commit/7f523ad", title: "7f523ad (PR #13): SSS1/SSSS1 scalar-sink −1, Yukawa standalone JAMP rows"}
---

# Convention signs in a diagram's sign

The evaluator's currents are built "honestly": each rooted current is the same
whatever vertex the diagram is rooted at. Every convention sign that does depend
on where a diagram is rooted, or that compensates a kernel convention, is lifted
into one per-diagram scalar, `fermi_sign`, assembled in
`compile_single_diagram` (`helas/eval/root_diagram.rs`):[^code-compile]

```rust
let fermi_sign = diagram.sign                       // Wick parity × fermion-line sign
    * fermion_current_line_sign(reference)          // tensor-path four-fermion lines
    * yang_mills_vvv_sign(diagram, model)           // colourless VVV sources
    * vector_contact_sign(diagram, model)           // all-vector contacts
    * gluon_scalar_current_sign(diagram, model)     // coloured VVS → off-shell scalar
    * reference.build_convention_sign()             // per-vertex build arms
    * reference.reversed_convention_sign()          // reversed-bilinear parity, anchor
    * tree.reversed_convention_sign();              // … cancelling the live tree's
```

`reference` is the tree rooted at the diagram's anchor (`Diagram::anchor`); `tree`
is the live tree, rooted at `canonical_root`. When the two roots coincide they are
one tree and the last two factors cancel. Why the anchor, and why two roots, is in
[rooting-invariance-and-anchor](rooting-invariance-and-anchor.md).

## The arms

| arm | read from | fires | class |
|---|---|---|---|
| Wick parity | `Diagram::fermion_pairing_sign` | parity of external fermions paired by line | graph |
| fermion-line sign | `Diagram::fermion_line_sign` | uncrossed line `(−1)^P`; crossed line `−1` | graph |
| tensor-path line sign | `fermion_current_line_sign` | divides the line sign back out for each line closed at a fermion-output tensor-path four-fermion current | anchor tree |
| Yang–Mills VVV source | `yang_mills_vvv_sign` | `−1` per *colourless* 3-leg all-vector `P`-carrying vertex other than the anchor | anchor |
| vector contact | `vector_contact_sign` | cancels the kernel's `−1` on every ≥4-leg all-vector vertex except a colourless one at the anchor | anchor |
| gluon–scalar current | `gluon_scalar_current_sign` | `−1` per coloured VVS vertex that produces an off-shell scalar in the anchor tree | anchor |
| build sign | `VertexInfo::build_sign` (from `build_at_leg`) | see below | anchor tree |
| reversed-bilinear parity | `VertexInfo::reversed_sign` (`term_reversed_parity`) | `−1` per fermion→vector sink (`GammaVout`, `SigmaVout`) whose UFO row index is a ket | anchor tree × live tree |

The fermion-line sign has its own concept, [fermion-line-sign](fermion-line-sign.md);
the vector-vertex arms (VVV, contact, gluon–scalar) are argued in
[vector-vertex-signs](vector-vertex-signs.md). A triple-gluon vertex and a gluon
contact take no sign in either role: their colour factors carry that half of the
antisymmetry, and a `−1` at a gluon source gives `u u~ > g g` the wrong relative
sign between gluon- and quark-exchange diagrams and breaks its Ward identity.

**Build-sign arms** (`root_lorentz.rs::build_at_leg`; one sign per vertex, asserted
uniform over the vertex's Lorentz terms):[^code-build]

- *scalar-sink bilinear*: `−1` when a `ProjM`/`ProjP`/`Identity`/`Gamma5` bilinear
  sinks into the amplitude or a scalar output (against the `−i/D` scalar
  propagator);
- *crossed pair*: a further `−1` on such a bilinear when either leg is on a crossed
  line (`pair_crossed`);
- *standalone projector on a crossed line*: `−1` when a standalone `ProjM`/`ProjP`/
  `Gamma5` is rooted at a fermion output and wraps a crossed leg
  (`standalone_projector_crossed`);
- *pure-metric vector–scalar*: `−1` once per term for a `Metric`-only VVS/VVSS
  structure, at an amplitude or scalar-output root;
- *all-vector contact*: `−1` for any vertex of ≥4 vectors, decided per vertex, not
  per term (then cancelled or kept by `vector_contact_sign`);
- *operator-free scalar contact*: `−1` for `SSS1`/`SSSS1`-type vertices (all
  scalars, no operator), pinned per diagram on `ta+ ta- > t t~ h(h)`;[^pr13]
- *tensor-path crossed line*: `−1` per crossed line of a cyclic four-fermion
  structure.

Two classes, measured on a census of every manifest process plus 23 others (2553
diagram–chain pairs, at every rooting):[^n38-s1] the Wick and line signs are graph
properties that never varied with the rooting; the VVV, build and reversed signs
are kernel compensations that vary with the reference root (on 214, 92 and 12
pairs, their product on 234). The choice of reference root carries physics: the
sign product differs on 53 pairs between feyngraph's vertex 0 and the vertex leg 0
attaches to, which is why the anchor is defined from the graph.

The colour side has one more convention, the `3`/`3̄` transpose of every `T` under
feyngraph's crossing; see [colour-crossing-epsilon-and-sextets](colour-crossing-epsilon-and-sextets.md).

## MadGraph puts the same sign elsewhere

MadGraph places the annihilation/exchange relative sign in the JAMP colour
coefficient `c_i`; vibegraph places it in the diagram root and keeps colour
coefficients uniformly `+1`. Neither is observable alone. The comparable object is
`c_i · AMP(i)`, never the bare `AMP(i)`:[^n24-p1] `e+ e- > e+ e-` carries
`c = (−1, −1, +1, +1)`, and `g u~ > e+ e- u~` flips MadGraph's colour structure
`T(1,5,2) → T(1,2,5)` and its coefficients `+1 → −1`. The same split forces the
per-configuration amplitude fit in `amplitude_oracle` to be per diagram, not
global (see [validation/amplitude-oracle](../validation/amplitude-oracle.md)).

## What the oracles can see

Write vibegraph's diagram amplitude as `A_d = φ_d · H_d`, with `φ_d = fermi_sign`
and `H_d` the sign-free evaluation. The per-diagram gate pins
`A_d = G · c_d · AMP_d`, so it sees only the *pattern* of `φ_d` up to one sign per
process, and never the split between `φ` and `H`: a sign moved from one to the
other is invisible to every banked oracle.[^n29-f] In `e+ e- > e+ e-` and
`u u~ > u u~` the Wick and line signs cancel exactly and the relative sign MadGraph
pins sits inside `H`. A uniform `φ` is still evidence: `g g > t t~` is uniform with
crossed top lines carrying 0, 1 and 1 propagators, which is what pins the crossed
line's propagator independence.

Which arm a gated row actually exercises has to be measured, not read off the
process. As of the chain-F measurement (note 29, 2026-08-03):

- `e+ e- > ta+ ta- H` exercises the *standalone-projector-crossed* arm, not the
  scalar-sink arm: its `−1` sits on the `ta ta H` (`FFS4`) vertex rooted at a
  fermion output, and its `ZZH` vertex is never a scalar-sink root at that
  rooting. The scalar-sink bilinear arm then had **no varying instance** in the
  banked set, so it was unchecked.
- The crossed-pair arm and the reversed-bilinear parity were reproduced but
  unchecked: neither produced a varying `φ` pattern in any probed process.

Since then the standalone JAMP rows `tata_to_ttxh`, `tata_to_ttxhh` and
`bbx_to_hh` gate Yukawa processes per helicity. Whether any of them puts a
*varying* scalar-sink bilinear sign in front of an oracle has not been
re-measured; until it is, treat that arm as possibly unchecked. The production
comment on the fourth assertion of `mg_guard_processes_exercise_every_convention_channel`
still attributes `e+ e- > ta+ ta- H`'s build sign to "ProjM/ProjP scalar-sink + the
crossed-τ standalone projector"; only the second half fires.

`mg_guard_processes_exercise_every_convention_channel` is the non-vacuity half:
each channel must fire on a named process (`e+ e- > W+ W-` VVV, Bhabha crossed
line, `g g > g g` contact build sign, `e+ e- > ta+ ta- H` build sign,
`e+ e- > mu+ mu-` reversed parity), so an enumeration change cannot silently
stop exercising a branch.[^code-guard] The per-channel row coverage is in
[validation/convention-channel-coverage](../validation/convention-channel-coverage.md).

## How a new sign gets found

The mixed-line arm of the line sign is the worked case.[^n28-s5s6]
`u d > e+ e- u d QCD=0`, the first row with two mixed quark lines, disagreed on a
subset of diagrams. Three candidate factors each fitted it and each was
falsified by an independent bit-exact row (listed in
[fermion-line-sign](fermion-line-sign.md)). The fix came from asking which slot
binding disagrees with the reference, and the account made a prediction that was
not the thing being fixed: the reversed-bilinear parity should equal
`(−1)^(#initial–initial + #mixed lines)` per diagram and never fire on a crossed
line. It did, on eight processes. Two diagrams (the triple-gauge ones) landed in
the right class without being named. Adding the process to the rooting sweep also
showed every diagram root-invariant, so the defect was not in the currents.

## Open

- [wpwm-to-epem-neutrino-exchange-sign](../backlog/validation/wpwm-to-epem-neutrino-exchange-sign.md):
  `w+ w- > e+ e-` has the wrong t-channel sign against the s-channel.
- [config-amp-phase-and-sign-unpinned](../backlog/validation/config-amp-phase-and-sign-unpinned.md):
  per-configuration phases are measured `±1` but asserted only in modulus.
- [mixed-dirac-and-matrix-free-vertex-terms](../backlog/feature/mixed-dirac-and-matrix-free-vertex-terms.md):
  a vertex mixing a `Gamma` term with an `Identity` term has no defined build sign.

Related: [global-phase-i-counting](global-phase-i-counting.md) (the `G = ±i`
constant), [charge-flow-phase-study](charge-flow-phase-study.md) (why these signs
do not reduce to one principle).

[^code-compile]: `vibegraph-lib/src/helas/eval/root_diagram.rs`.
[^code-build]: `vibegraph-lib/src/helas/eval/root_lorentz.rs`, `build_at_leg` and `RootedTerm::build_sign`.
[^code-guard]: `root_diagram.rs` tests.
[^n24-p1]: Note 24 §P1 outcome.
[^n28-s5s6]: Note 28 §S5–S6.
[^n29-f]: Note 29 §F.1, §F.8, §F.10, §F.13.
[^n38-s1]: Note 38 §S1.
[^pr13]: Commit 7f523ad.
