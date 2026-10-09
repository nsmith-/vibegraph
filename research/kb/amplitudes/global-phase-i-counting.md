---
type: Derivation
title: "The global amplitude phase: i-counting and the contravariant vector current"
description: "Vertex i and propagator −i multiply to i for any tree, so |G|=1 and Re G=0 against MadGraph; every vector producer emits the physical contravariant current and carries no sign of its own."
status: draft
tags: [phase-conventions, propagators, vector-current, madgraph-oracle, derivation]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: code-kernel, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/vibegraph-lib/src/helas/eval/kernel.rs#L405-L500", title: "kernel.rs: propagate_core and the −i/D propagators"}
  - {id: code-metricvout, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/vibegraph-lib/src/helas/eval/kernel.rs#L890-L910", title: "kernel.rs: metric_vout, the contravariant current"}
  - {id: code-oracle, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/vibegraph-lib/tests/amplitude_oracle.rs#L1-L100", title: "amplitude_oracle.rs module doc: G, |G| = 1, Re G = 0, known blind spots"}
  - {id: n13-s7, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/13-typed-repr-conventions-design.md#L289-L354", title: "Note 13 §7: one physical contravariant vector convention (Stage B)"}
  - {id: n29-f1, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/29-v01-validation-sprint-plan.md#L353-L437", title: "Note 29 F.1: the i-counting observation"}
  - {id: n29-f6, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/29-v01-validation-sprint-plan.md#L664-L682", title: "Note 29 F.6: the VVVV phase was a real −1 (b62ac17)"}
  - {id: n29-f12, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/29-v01-validation-sprint-plan.md#L1037-L1095", title: "Note 29 F.12: G tracks MadGraph's colour-coefficient sign"}
---

# The global amplitude phase

## i-counting

Two conventions fix the phase of every tree diagram:[^code-kernel]

- the UFO coupling carries the vertex factor `i`;
- every propagator carries `−i/D`, whatever it propagates: vector
  `−i(g − qq/m²)/D` (massive) or `−i g/q²` (massless), Dirac `−i(q̸ + m)/D`, scalar
  `−i/D`.

A tree diagram with `V` vertices has `V − 1` internal propagators, so its phase
is[^n29-f1]

```
i^V · (−i)^(V−1) = (−1)^(V−1) · i^(2V−1) = i      for every V.
```

Every diagram therefore carries exactly one factor of `i` that MadGraph's `AMP()`
does not. The oracle fits one complex constant `G` per process with
`A_d^vg = G · c_d · AMP_d^mg` (diagrams) and `J_f^vg = G · JAMP_f^mg` (flows), and
asserts `|G| = 1` and `Re G = 0` at `LINEAR_REL_TOL = 1e-12`
(`tests/amplitude_oracle.rs`).[^code-oracle] Both follow from i-counting with no
free parameter, and both held on every banked process when last measured.

Uniform propagator phases are load-bearing, not cosmetic. If one chain type
(fermion, scalar, vector) carried a different phase from the others, a process
mixing chain contents would interfere wrongly: `u u~ > c c~ e+ e- mu+ mu- QCD=0`
mixes two-fermion-propagator continuum diagrams with one-scalar-propagator Higgs
diagrams, and is the per-diagram pin. The scalar propagator's `−i/D` is matched
by the scalar-sink `−1`s of [convention-sign-inventory](convention-sign-inventory.md).

**The propagator-free diagram.** A four-point contact diagram (`V = 1`, `P = 0`)
gets `i` directly. Any extra imaginary factor there is an uncancelled 90° rotation
against the exchange diagrams, which is why the VVVV contact's convention is a
real `−1` and not `−i` (commit `b62ac17`, which also deleted
`Op::MetricNegI`).[^n29-f6] How that `−1` is kept or cancelled per role is in
[vector-vertex-signs](vector-vertex-signs.md).

## Only the sign of G is a convention

`G`'s quadrant is derived; its sign is not asserted (it is printed). The sign
tracks MadGraph's own colour-coefficient convention, not anything on vibegraph's
side:[^n29-f12] `e+ e- > mu+ mu-` and `u u~ > mu+ mu-` have identical diagram and
fermion-line structure and `G = −i` vs `+i`, and MadGraph's banked coefficients
differ in exactly the compensating way (`c = (−1,−1)` vs `(+1,+1)`), so `G·c₀ = +i`
for both. Over the 16 coefficient-banked processes of 2026-08-03, `G·c₀ = +i` for
13 and `−i` for `ee_to_mumu_tata_qcd0`, `ee_to_wpwm` and `ee_to_zh`.

A phase common to every diagram and flow is genuine convention freedom —
MadGraph chooses the overall sign of `JAMP(1)` per colour structure — and is
invisible to `|M|²` and `JAMP2`. `Re G = 0` narrows it to a sign and cannot remove
it. Deriving that sign would derive a quantity no output of this generator
depends on.

The same `G` serves diagrams and flows, which is what ties the diagram convention
to the colour convention. That includes `g g > g g` at `NCOLOR = 6`: vibegraph's
six sorted basis keys are MadGraph's six structures in MadGraph's order, and the
JAMPs agree flow by flow under the process's single `G` (see
[colour-flow-lines-and-conjugation](colour-flow-lines-and-conjugation.md)).

`tests/standalone_jamps.rs` compares against MadGraph *standalone* output, where
the fitted constant's modulus is asserted but its phase is left free: external
wavefunction phase conventions there differ by a process-dependent `±1`/`±i`.

## The vector current is contravariant everywhere

Every vector producer emits the physical contravariant current `V^μ`, and every
propagator outputs a contravariant current.[^code-metricvout]

- `MetricVout` is `g^{μν}V_ν = V^μ`, an identity on contravariant storage. No
  phase lives there: ALOHA's `VVS1P1N_1 = −i·g·V` folds the propagator's `−i` into
  the vertex routine, while here the coupling carries the `i` and the propagator
  the `−i`, so vibegraph's VVS current is `+i × VVS1P1N_1`.[^n13-s7]
- A `P`-carrying term rooted at its vector leg (`P(1,2)·Metric(2,3)` at leg 1)
  emits the bare momentum wrapped in the same `MetricVout`. The output-leg momentum
  is `PMomOut = −Σ inputs`, ALOHA's `P1 = −(V2+V3)`.
- The Yang–Mills VVV current is built honestly (`+V^μ`) at every rooting. The
  `−1` a colourless VVV vertex needs as a *source* is a diagram-level sign,
  `yang_mills_vvv_sign`, not a producer phase.
- There is no covariant vector slot and no index-lowering op. Variance is a type
  parameter on `VectorWf`; the evaluator's register carries contravariant vectors
  only (see [flat-op-ir](flat-op-ir.md)).

The single case where the physical convention and an index-relabelling
convention differ is the massless propagated current's time component. At the
validation points the conserved sink current's `J⁰` and the assembled VVV current's
time component are exactly `0.0`, so amplitudes there cannot distinguish the two;
a t-channel massless VVV rooting gets the physical time component.

External wavefunctions and propagator details:
[wavefunctions-and-propagators](wavefunctions-and-propagators.md). The oracle's
fit and blind spots: [validation/amplitude-oracle](../validation/amplitude-oracle.md).

[^code-kernel]: `vibegraph-lib/src/helas/eval/kernel.rs`, `propagate_core` and the `propagate_*_bare` kernels.
[^code-metricvout]: `kernel.rs`, `metric_vout`; `root_lorentz.rs`, `vector_out_node`.
[^code-oracle]: `vibegraph-lib/tests/amplitude_oracle.rs`, module documentation.
[^n13-s7]: Note 13 §7, step 5b.
[^n29-f1]: Note 29 §F.1, "the i-counting observation".
[^n29-f6]: Note 29 §F.6.
[^n29-f12]: Note 29 §F.12, witness pair W2.
