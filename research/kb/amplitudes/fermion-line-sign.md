---
type: Physics Convention
title: Fermion-line sign
description: "Diagram::fermion_line_sign: an uncrossed line takes one −1 per internal propagator whatever its vertices; a crossed line takes a single −1; why the fitted alternatives fail."
status: draft
tags: [fermion-flow, sign-convention, crossing, diagrams, madgraph-oracle]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: code-line-sign, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/vibegraph-lib/src/diagrams/diagram.rs#L526-L581", title: "Diagram::fermion_line_sign and its derivation doc"}
  - {id: code-closed-line, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/vibegraph-lib/src/helas/eval/root_diagram.rs#L583-L718", title: "spine_sign_from_flow / closed_line_sign: the rooted-tree cross-check"}
  - {id: n12-causes, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/12-helas-continuum-bugfix-journey.md#L37-L87", title: "Note 12, root causes 4–6 (initial-state spine sign, crossed-line conjugation, per-propagator parity)"}
  - {id: n28-s6, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/28-kt-spine-feature-sprint-plan.md#L2598-L2756", title: "Note 28 S6, the crossing sign rule (mixed lines)"}
  - {id: n35-t3, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/35-ufo-lorentz-sprint-plan.md#L1042-L1112", title: "Note 35 T3 (the Dirac-content exemption, since reverted)"}
  - {id: n35-closeout, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/35-ufo-lorentz-sprint-plan.md#L1281-L1358", title: "Note 35 §10.1 close-out (rule 5, since reverted)"}
  - {id: fact-line-sign, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/facts/fermion-line-sign-ignores-vertex-content.md#L12-L35", title: "Fact: the fermion-line sign ignores vertex content (replaced by this concept)"}
  - {id: pr13, resource: "https://github.com/nsmith-/vibegraph/commit/7f523ad", title: "7f523ad (PR #13): revert of the Dirac-content exemption, SSS1/SSSS1 scalar-sink −1"}
---

# Fermion-line sign

Every diagram carries a relative Fermi sign `Diagram::sign`, set once in
`Diagram::from_view` as the product of two graph properties
(`vibegraph-lib/src/diagrams/diagram.rs`):

- `fermion_pairing_sign` — the parity of the external fermions paired by line
  (feyngraph's `view.sign()`, the Wick sign);
- `fermion_line_sign` — one factor per fermion line, described here.

The line sign reads only the lines (`Diagram::fermion_lines`): their two end legs
and the number of internal fermion propagators `P` on each. It does not read the
vertices, the rooting or the numbering.[^code-line-sign]

| line class | ends | factor |
|---|---|---|
| initial–initial | both incoming | `(−1)^P` |
| mixed | one incoming, one outgoing | `(−1)^P` |
| crossed | both outgoing | `−1`, whatever `P` is |

```rust
let crossed = line.legs.iter().all(|l| l.0 >= self.n_in);
let propagators = line.vertices.len() - 1;
if crossed || propagators % 2 == 1 { sign = -sign; }
```

## Where it comes from

Diagram enumeration (feyngraph) binds external legs to vertex slots in the
*all-incoming* identity: an outgoing leg sits at its antiparticle's slot. The
HELAS bookkeeping MadGraph's amplitudes are defined against binds them in the
*all-outgoing* identity, where an incoming leg sits at its antiparticle's slot. A
UFO fermion slot fixes a spinor adjoint (pair-first slot takes the ket,
pair-second the bra), so the two bindings disagree on exactly the legs neither
side crosses the same way:[^n28-s6]

| leg | diagram enumeration | reference HELAS |
|---|---|---|
| initial | physical identity | anti identity |
| final, on a mixed line | anti identity, restored to physical by `mixed_line_final_legs` | physical |
| final, on a crossed line | anti identity, kept | physical |

A line with at least one initial-state end is therefore read *against* its slot
arrow at every vertex. Reading a bilinear backwards replaces the vertex structure
by `C Γᵀ C⁻¹`, which for `Γ = γ^μ P_χ` is `−γ^μ P_χ̄`. The chirality flip is applied
per vertex (`chiral_correction`, `root_lorentz.rs`). Of the `V` minus signs, one is
applied at the line's single vector-rooted sink by the reversed-bilinear parity
(see [convention-sign-inventory](convention-sign-inventory.md)); the remaining
`V − 1 = P`, one per internal propagator, are this factor.

A crossed line keeps the anti identity *and* the anti wavefunction, so it is read
*along* its arrow and takes no per-propagator factor. Its single −1 is the
operator reordering of the conjugated pair relative to the reference's physical
pair: a final-state pair evaluates `ū₁Γv₂` where the vertex is defined `ū₂Γv₁`, and
`ū₁Γv₂ = −ū₂(CΓᵀC⁻¹)v₁`.[^n12-causes] Crossing inverts slot identity and adjoint
together, which is why the evaluator carries an explicit per-leg `crossed` bit
(see [fermion-flow-and-crossing](fermion-flow-and-crossing.md)).

## The factor ignores vertex content

A chain of Yukawa-type vertices (`Identity`, `Gamma5`, bare projectors) takes
exactly the factor a gauge line does.[^fact-line-sign] The pins, all against
MadGraph per diagram or per helicity:

- `ta+ ta- > t t~ h` (14 diagrams) and `ta+ ta- > t t~ h h`: the tau line that
  radiates a Higgs and then annihilates is right only with the −1; standalone JAMP
  rows `tata_to_ttxh` and `tata_to_ttxhh` (`tests/standalone_jamps.rs`).
- `b b~ > c c~ e+ e- mu+ mu- QCD=0`: a mixed gauge/Yukawa `b` line (one photon,
  one `b b~ H` vertex, one propagator) is bit-for-bit only with the −1.
- `b b~ > h h` (`bbx_to_hh`, and the unit test `every_line_takes_the_propagator_sign`):
  the exchange diagrams take the line's −1, the triple-Higgs annihilation diagram
  takes its −1 from the all-scalar `HHH` vertex's scalar-sink sign instead, and the
  two interfere at MadGraph's relative sign.
- The toy row `qt qt~ > o8 o8` agrees because its s-channel runs through an
  all-scalar cubic vertex whose own scalar-sink −1 (`build_at_leg`, the `SSS1`/`SSSS1`
  arm) balances the t/u-channel propagator's.

## Alternatives that fit some data and are false

Each was a candidate at some point; each is refuted by a bit-exact row.[^n28-s6]

- **"−1 per propagator on a crossed final–final line."** On `u d > e+ e- u d
  QCD=0` alone it selects the complement of the right set and a global sign
  absorbs it, but it moves `e+ e- > mu+ mu- ta+ ta-` diagrams 0–15 against 17–24
  and `g g > t t~`'s t/u-channel diagrams against the s-channel.
- **"The WWγ/WWZ vertex takes no Yang–Mills sign."** Falsified by
  `e+ e- > W+ W-`, whose s-channel diagrams need the VVV source sign.
- **"−1 per spacelike boson propagator."** Fits `u d > e+ e- u d` and is false on
  `u u~ > u u~`.
- **"−1 per propagator only on lines whose vertices carry a Dirac matrix"**
  (`Identity`/`Gamma5`/projector lines exempt). It looked right on
  `qt qt~ > o8 o8` only because it cancelled a second bug, the missing
  scalar-sink −1 on `SSS1`/`SSSS1`. With the exemption, `ta+ ta- > t t~ h h` has
  24 wrong signs of 96 against MadGraph's `AMP()`; with both fixed it has none.
  Reverted in 7f523ad (PR #13); `carries_dirac_matrix` no longer exists.[^pr13]
  Note 35 §T3 and §10.1 rule 5 (archived) describe the reverted rule.[^n35-t3][^n35-closeout]

## How it is checked

- `debug_assert_eq!(spine_sign_from_flow(&tree), diagram.fermion_line_sign(model))`
  in `compile_single_diagram`: every compiled diagram's line sign is re-derived
  from the spinor adjoint baked into its rooted tree.[^code-closed-line]
- `fermion_line_sign_matches_the_rooted_tree_derivation` holds that agreement at
  every rooting; `spine_sign_separates_mixed_line_and_crossed_line_propagators`
  asserts the 24 / 9 / 2 split of `u d > e+ e- u d QCD=0` (propagator on a mixed
  line / on the crossed lepton line / none) with `g g > t t~` as the control that
  a crossed line's −1 does not count propagators.
- Per-diagram MadGraph rows in `tests/amplitude_oracle.rs`, notably
  `ud_to_epemud_qcd0` (the first row with two mixed quark lines), `ee_to_ee`
  (crossed s-channel vs uncrossed t-channel) and the uux/bbx 2→6 classes.

## Caveats

- Every test above is blind to a sign common to all diagrams of a process; the
  absolute per-diagram pattern of `u d > e+ e- u d` is pinned only by its
  `amplitude_oracle` row. If that row were demoted, the split test alone would
  pass with the line sign off by a global sign.
- On a mixed gauge/Yukawa line, which vertex "owns" which factor is not resolved
  by any oracle; only the product per line is pinned.
- "One −1 per propagator" and "one −1 per vertex read backwards, less the one at
  the sink" are the same function on every tree line (`P = V − 1`), so no
  tree-level dump can tell the two phrasings apart (see
  [charge-flow-phase-study](charge-flow-phase-study.md)).
- A line closed at a fermion-output *tensor-path* four-fermion current has its
  line sign divided back out (`fermion_current_line_sign`); see
  [four-fermion-vertices](four-fermion-vertices.md).
- Majorana lines are out of scope; see
  [majorana-fermions-unsupported](../backlog/feature/majorana-fermions-unsupported.md).

Because the factor reads only the graph, it is invariant under re-rooting and
renumbering; see [rooting-invariance-and-anchor](rooting-invariance-and-anchor.md).

[^code-line-sign]: `vibegraph-lib/src/diagrams/diagram.rs`, `fermion_line_sign` and its doc comment.
[^code-closed-line]: `vibegraph-lib/src/helas/eval/root_diagram.rs`, `spine_sign_from_flow`, `closed_line_sign`, `compile_single_diagram`.
[^n12-causes]: Note 12, root causes 4–6.
[^n28-s6]: Note 28 §S5–S6.
[^n35-t3]: Note 35 §T3.
[^n35-closeout]: Note 35 §10.1.
[^fact-line-sign]: The fact file this concept replaces.
[^pr13]: Commit 7f523ad.
