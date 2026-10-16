---
type: Physics Convention
title: Convention signs in a diagram's sign and what pins each
description: "fermi_sign = Diagram::sign × convention_sign × the live tree's reversed parity: the Wick and line signs, the propagator phases, each vertex's reversed-bilinear parity as the amplitude root, the tensor-path sign and the triple-gluon −1; what each reads and which oracle pins it."
status: draft
tags: [sign-convention, fermi-sign, rooting, amplitudes, madgraph-oracle]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-15}
sources:
  - {id: b-report, resource: "../sprints/amplitude-signs/sessions/B-report.md", title: "B′ report: the derivation, the census comparison and the per-factor mutations"}
  - {id: code-compile, resource: "https://github.com/nsmith-/vibegraph/blob/5d4a34672e784f8b5fc985e82bd656232ae07caa/vibegraph-lib/src/helas/eval/root_diagram.rs", title: "root_diagram.rs: convention_sign, propagator_phase_sign, vertex_root_signs, triple_gluon_sign, compile_single_diagram"}
  - {id: code-rootl, resource: "https://github.com/nsmith-/vibegraph/blob/5d4a34672e784f8b5fc985e82bd656232ae07caa/vibegraph-lib/src/helas/eval/root_lorentz.rs", title: "root_lorentz.rs: term_reversed_parity, RootedTerm::build_sign (the tensor path)"}
  - {id: code-kernel, resource: "https://github.com/nsmith-/vibegraph/blob/5d4a34672e784f8b5fc985e82bd656232ae07caa/vibegraph-lib/src/helas/eval/kernel.rs", title: "kernel.rs: propagate_core (every propagator −i)"}
  - {id: n24-p1, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/24-user-distribution-and-proton-events-plan.md#L665-L755", title: "Note 24 P1: rows enforced per channel; c_i·AMP(i) is the comparable object"}
  - {id: n28-s5s6, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/28-kt-spine-feature-sprint-plan.md#L2346-L2756", title: "Note 28 S5–S6: falsified candidates and the crossing sign rule"}
  - {id: n29-f, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/29-v01-validation-sprint-plan.md#L353-L1152", title: "Note 29 chain F: what the per-diagram dumps pin"}
---

# Convention signs in a diagram's sign

The evaluator's currents are built "honestly": no kernel applies a convention
sign, and each rooted current is the same whatever vertex the diagram is rooted
at. What separates a diagram's value from MadGraph's `AMP()` is one per-diagram
scalar, `fermi_sign`, assembled in `compile_single_diagram`
(`helas/eval/root_diagram.rs`)[^code-compile]:

```rust
let fermi_sign = diagram.sign                       // Wick parity × fermion-line sign
    * convention_sign(diagram, model, chain)?       // propagator phases × vertex signs × triple gluon
    * tree.live_reversed_parity()?;                 // cancels the runtime reversed parity
```

Every factor but the last reads the graph alone, so `fermi_sign` is the same at
every rooting, numbering and leg labelling
([rooting-invariance-and-canonical-form](rooting-invariance-and-canonical-form.md)).

## Why the convention is a product of local constants

MadGraph's `AMP()` is rooting-invariant: ALOHA evaluates every vertex as
`COUP · L`, closes the amplitude with `−i` whichever vertex that is, and gives
each propagator a fixed phase (`+i` for spin 0 and ½, `−i(g − qq/m²)` for spin
1). The kernels are rooting-invariant too. So the ratio between a diagram here
and its MadGraph graph is one constant per internal line times one per vertex,
and `convention_sign` is that product[^b-report].

## The arms

| arm | read from | value | pinned by |
|---|---|---|---|
| Wick parity | `Diagram::fermion_pairing_sign` | parity of the external fermions paired by line | every multi-line MadGraph row |
| fermion-line sign | `Diagram::fermion_line_sign` | a line with an initial-state end: `(−1)^P`, the fermion propagators' phase; a crossed line: `−1`, the reordering of the conjugated pair | [fermion-line-sign](fermion-line-sign.md) |
| propagator phase | `propagator_phase_sign` | `−1` per internal scalar line, and per internal fermion line of a crossed line | 25 standalone rows fail without the scalar term, 22 without the crossed-line term |
| vertex reading | `vertex_root_signs`, from `VertexInfo::reversed_sign` | each vertex's reversed-bilinear parity, read with the vertex as the amplitude root: `−1` per `γ^μ` or `σ^{μν}` bilinear whose row slot is bound to a ket | 17 standalone rows and 28 amplitude-oracle rows fail without it |
| tensor-path sign | `vertex_root_signs`, from `VertexInfo::build_sign` on a cyclic four-fermion term | the sign the tensor path gives its amplitude root: `−1` per crossed line of the cyclic structure | `tata_to_ttxa_tensor4f`, `ll_to_qqx_toy_tensor`, `tata_to_ttx_tensor4f` |
| triple gluon | `triple_gluon_sign` | `−1` per coloured three-vector vertex | `uux_to_ggg`, `ug_to_ug`, `gg_to_uux`, `ttx_to_gg_chg`, the five `t t~ + jet` rows, `uux_to_ttxg_4f`; oracle `gg_to_ttx` and its two SM-limit rows |
| live reversed parity | `DiagramEvalTree::live_reversed_parity` | the runtime parity of the live tree, applied again so that it cancels | `rooting_soundness` |

The "pinned by" column is the per-factor mutation of the B′ session: each factor
dropped on its own, against `standalone_jamps` and `amplitude_oracle`; every
listed failure is a per-diagram or per-flow disagreement with MadGraph[^b-report].
[validation/convention-channel-coverage](../validation/convention-channel-coverage.md)
has the full table.

**The propagator phase.** ALOHA's off-shell currents carry `+i` on a fermion or
scalar propagator (`i(q̸ + m)/D`, `i/D`) and `−i` on a vector one; every
propagator here carries `−i` (`kernel::propagate_core`)[^code-kernel]. So every
internal fermion and scalar line takes `−1` against MadGraph and a vector line
none. The fermion lines of a line with an initial-state end take theirs inside
`Diagram::fermion_line_sign`, which leaves a crossed line's to this arm.

**The vertex reading.** A vertex reads each fermion bilinear with the spinors
its lines bind to its slots: each fermion slot is bound to the external leg its
line reaches through that slot (`external_bindings`, `line_end`). Where a
bilinear's row slot is bound to a ket, the bilinear is read against its UFO
order, `ψ̄₁Γψ₂ = ψ̄₂(CΓᵀC⁻¹)ψ₁`, and `CΓᵀC⁻¹ = −Γ` for `γ^μ` and `σ^{μν}` while
`1`, `γ⁵` and `P_χ` keep their sign. That is `term_reversed_parity` with every
leg an input, a function of which external leg each slot leads to, and so of
the graph[^code-rootl]. A vertex whose terms disagree on the parity is refused
(`uniform_sign`): it has no single sign.

**The tensor path.** A cyclic four-fermion structure
(`VertexInfo::tensor`) cuts its index cycle at one line and reads the reversals
inside the kernel, so its terms carry no reversed parity of their own. They
carry instead the sign that path gives its amplitude root
(`RootedTerm::build_sign`, `+1` on every tree-rooted term). See
[four-fermion-vertices](four-fermion-vertices.md).

**The triple-gluon vertex.** Its Lorentz structure and coupling normalisation
are the colourless triple-vector vertices', which take no sign; the sign
belongs to how the colour-ordered decomposition of `f^{abc}` meets the vertex
slots. Only its ratio to the quark–gluon vertex is observable: a process's
external gluons fix the parity of the two counts together, so `−1` per
quark–gluon vertex is the same convention up to a sign common to the process.
The constant is calibrated, not derived (below). Why no other vector vertex
carries a sign is [vector-vertex-signs](vector-vertex-signs.md).

The colour side has one more convention, the `3`/`3̄` transpose of every `T`
under feyngraph's crossing; see
[colour-crossing-epsilon-and-sextets](colour-crossing-epsilon-and-sextets.md).

## How the factors were fixed

The factors are the minimal solution of a linear system over GF(2)[^b-report]:

- The targets were the MadGraph-verified per-diagram signs of every
  amplitude-oracle row, every standalone table and W1's 40 probes: 3495
  diagrams.
- The unknowns were a constant per vertex class and per line class, up to one
  sign per process, with the root-invariance equations added.
- The system is consistent once the vertex class carries its reversal parity.
  With the line constants fixed from the propagator phases, the unique minimal
  solution is one more constant: `−1` on the triple-gluon vertex. The next
  smallest has weight 9.

The convention was then compared, per (diagram, colour chain), with the
measured signs on 506 subprocesses (7347 signs) and differed on none.

## MadGraph puts the same sign elsewhere

MadGraph places the annihilation/exchange relative sign in the JAMP colour
coefficient `c_i`; vibegraph places it in `fermi_sign` and keeps colour
coefficients uniformly `+1`. Neither is observable alone. The comparable object
is `c_i · AMP(i)`, never the bare `AMP(i)`[^n24-p1]: `e+ e- > e+ e-` carries
`c = (−1, −1, +1, +1)`, and `g u~ > e+ e- u~` flips MadGraph's colour structure
`T(1,5,2) → T(1,2,5)` and its coefficients `+1 → −1`. The same split forces the
per-configuration amplitude fit in `amplitude_oracle` to be per diagram, not
global (see [validation/amplitude-oracle](../validation/amplitude-oracle.md)).

## What the oracles can see

Write vibegraph's diagram amplitude as `A_d = φ_d · H_d`, with `φ_d = fermi_sign`
and `H_d` the sign-free evaluation. The per-diagram gate pins
`A_d = G · c_d · AMP_d`, so it sees only the *pattern* of `φ_d` up to one sign per
process, and never the split between `φ` and `H`: a sign moved from one to the
other is invisible to every banked oracle[^n29-f]. In `e+ e- > e+ e-` and
`u u~ > u u~` the Wick and line signs cancel exactly and the relative sign
MadGraph pins sits inside `H`. A uniform `φ` is still evidence: `g g > t t~` is
uniform with crossed top lines carrying 0, 1 and 1 propagators, which is what
pins the crossed line's propagator independence.

**The per-process sign is free.** `|M|²`, every σ and the relabelling test are
blind to a sign shared by every diagram of a process, and the fitted `G`
absorbs it. No labelling-free rule reproduces one particular choice of it, so
the per-process sign the convention gives is its own; `CONFIG_AMP_SIGNS` banks
it as a tripwire, not as physics[^b-report].

## How a new sign gets found

The mixed-line arm of the line sign is the worked case[^n28-s5s6].
`u d > e+ e- u d QCD=0`, the first row with two mixed quark lines, disagreed on a
subset of diagrams. Three candidate factors each fitted it and each was
falsified by an independent bit-exact row (listed in
[fermion-line-sign](fermion-line-sign.md)). The fix came from asking which slot
binding disagrees with the reference, and the account made a prediction that was
not the thing being fixed. The method in general is
[bit-exact-amplitude-debugging](../validation/bit-exact-amplitude-debugging.md):
per-diagram × per-helicity complex dumps against `AMP()`, then a GF(2) solve for
the sign pattern over graph features, as above.

## Open

- **The triple-gluon `−1` has no derivation.** It is the one constant the
  propagator phases and the bilinear algebra do not fix; following the trace
  basis through `colorize` for one `u u~ > g g` diagram would place it.
- **Unmeasured classes.** No committed row reaches a scalar⊗scalar four-fermion
  contact rooted in a tree (`O_ledq`-type: the convention gives it no vertex
  sign), a cyclic tensor structure on two mixed lines (the tensor-path sign is
  pinned on (initial, crossed) line pairs only), or a VSS vertex (no vertex
  sign, and `−1` for the scalar lines beside it).
- [config-amp-signs-blind-to-multiflow-rows](../backlog/validation/config-amp-signs-blind-to-multiflow-rows.md):
  per-configuration phases and sign patterns are asserted on the rows with a
  per-diagram comparison (`CONFIG_AMP_SIGNS`), not yet on the multi-flow rows.
- [mixed-dirac-and-matrix-free-vertex-terms](../backlog/feature/mixed-dirac-and-matrix-free-vertex-terms.md):
  a vertex mixing a `Gamma` term with an `Identity` term has a sign only where none
  of its bilinears is read reversed. That is derived (B′ report, H11), not run, and
  no oracle checks it.

Related: [global-phase-i-counting](global-phase-i-counting.md) (the `G = ±i`
constant), [charge-flow-phase-study](charge-flow-phase-study.md) (why these signs
do not reduce to one principle).

[^b-report]: B′ report: the derivation table, the GF(2) solve, the (diagram, chain) comparison and the per-factor mutations.
[^code-compile]: `vibegraph-lib/src/helas/eval/root_diagram.rs`.
[^code-rootl]: `vibegraph-lib/src/helas/eval/root_lorentz.rs`, `term_reversed_parity` and `RootedTerm::build_sign`.
[^code-kernel]: `vibegraph-lib/src/helas/eval/kernel.rs`, `propagate_core`.
[^n24-p1]: Note 24 §P1 outcome.
[^n28-s5s6]: Note 28 §S5–S6.
[^n29-f]: Note 29 §F.1, §F.8, §F.10, §F.13.
