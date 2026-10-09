---
type: Validation Gate
title: "Kernel correctness and reach: property harness and op census"
description: "Each hand-written kernel is checked fused == generic on typed random inputs, and a per-model two-way census fails when a kernel no gated row reaches, or a listed-uncovered op that is reached."
status: draft
tags: [evaluator, kernels, coverage, property-tests, ufo]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-09}]
sources:
  - {id: n13-3a, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/13-typed-repr-conventions-design.md#L200-L222", title: "Note 13 §3a — fused(random) == generic(random) as the per-kernel oracle"}
  - {id: n13-7, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/13-typed-repr-conventions-design.md#L289-L354", title: "Note 13 §7 — the harness and the IEEE-exact sign shuffles"}
  - {id: n35-e1, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/35-ufo-lorentz-sprint-plan.md#L334-L431", title: "Note 35 E1 — new kernels, the fusion guard and the SMEFTsim census"}
  - {id: n35-l1, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/35-ufo-lorentz-sprint-plan.md#L604-L675", title: "Note 35 L1 — the per-model op census"}
  - {id: n35-toy, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/35-ufo-lorentz-sprint-plan.md#L1416-L1434", title: "Note 35 §10.4 — the toy models' census"}
  - {id: code-harness, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/vibegraph-lib/src/helas/eval/prop_harness.rs", title: "helas/eval/prop_harness.rs"}
  - {id: code-census, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/vibegraph-lib/src/helas/eval/compile.rs#L1053-L1160", title: "assert_op_coverage_across and the SM census"}
  - {id: code-smeft, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/vibegraph-lib/tests/smeftsim.rs#L700-L746", title: "tests/smeftsim.rs — SMEFTsim census"}
  - {id: code-toy, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/vibegraph-lib/tests/toy_models.rs#L195-L240", title: "tests/toy_models.rs — toy-model census"}
---

# Kernel correctness and reach: property harness and op census

Two complementary checks sit under the MadGraph amplitude gates. One says each
hand-written kernel computes what the generic path computes; the other says
every kernel is actually reached by some gated process, so the amplitude oracle
is evidence for it. Neither compares against MadGraph.

## Per-kernel equivalence on random inputs

`helas/eval/prop_harness.rs` is a toolbox, not a test: typed random-input
generators (ket and bra spinors, polarisation vectors, momenta, scalars, each
wrapped in the `WaveformSlot` the kernels take), a variant-strict comparator
`slots_approx_eq` (a `Vector` never equals a `Scalar`; within a variant every
stored component and the routed momentum are compared), and the driver
`check_agree(n, seed, tol, gen, lhs, rhs)`. The RNG is seeded, so a failure
reproduces from the reported seed.

The inputs are deliberately arbitrary: off shell, equation-of-motion violating.
The identities being certified are algebraic, and an identity that holds only on
shell is the wrong thing to certify. Both sides receive the same input vector, so
momentum routing stays consistent without physical constraints.

The design reason is in note 13: matching a vertex's term list to a fused
kernel happens at compile time, once per (vertex, output leg), and each
hand-coded kernel then has a mechanical definition of correct — equal to the
typed general path on random momenta — instead of "add a node when a validation
process breaks", which is how the old stopgap ops accreted. Note 12's lesson was
that every convention bug lived at a hand-coded duality boundary; this makes
those boundaries test-me rather than trust-me[^n13-3a]. A structure whose terms do
not lie in a kernel's basis falls through to the generic path.

Uses on record:

- Sign-shuffle rewrites certified bit-exact before any rewiring, e.g.
  `propagate(metric_vout(v), m>0) ≡ propagate(Vector(+v), m>0)` on 10k random
  inputs[^n13-7].
- `kernel::tests::ffv_vout_matches_generic_chiral_pair` and
  `ffv_fermion_out_matches_generic_chiral_pair`: the fused `Ffv*` forms against
  the generic chiral pair. These hermetic pins are the equivalence evidence for
  fusion, because the process-level fused-vs-generic comparison is vacuous on
  SMEFTsim (its restricted vertices keep one chirality, so no chiral pair
  survives to fuse). The fusion site `chiral_gamma_site` refuses a chain carrying
  a momentum slash or a `Gamma5`[^n35-e1].
- New kernels are pinned against an algebraic identity of MadGraph-covered ones,
  e.g. `EpsilonVout · d = EpsilonAmp`.

The peephole design these kernels come from is
[the intertwiner basis and peephole kernels](../amplitudes/intertwiner-basis-and-peephole.md);
the evaluator they live in is [evaluator architecture](../amplitudes/evaluator-architecture.md).

## The two-way op census

`assert_op_coverage_across(instances, known_uncovered)` (`helas/eval/compile.rs`,
under `test` or `extended-validation`) compiles every `(label, model, processes)`
instance, counts which `Op` variants appear in the compiled arenas, and asserts
that **the set of ops never reached equals the allowlist exactly**. It is
two-way: an op that stops being reached fails, and so does an allowlisted op that
becomes reached, which forces the allowlist down as coverage grows[^n35-l1]. A
model whose gated processes use several restrict cards is one instrument spread
over several loaded models; what it leaves unreached is what none reaches.

Three instances at `787070e`:

| census | processes | allowlisted (never reached) |
|---|---|---|
| `mg_validated_suite_exercises_every_op` (`compile.rs`, SM) | `MG_VALIDATED_PROCESSES` | 16: `Hels`, `IdentityAmp`, `Gamma5`, `Gamma5Amp`, `EpsilonVout`, `EpsilonAmp`, the three `Fierz*`, `MultivectorIout/Oout`, the five `Sigma*` |
| `tests/smeftsim.rs` | the gated SMEFTsim rows | 13: `Hels`, `ProjMAmp`, `ProjPAmp`, `Gamma5Amp`, `MultivectorIout/Oout`, the five `Sigma*`, `FfvIout`, `FfvOout` |
| `tests/toy_models.rs` | the gated toy rows | 20, including `Hels`, `GammaIout/Oout`, `ProjP`, `ProjM/PAmp`, `MetricVout`, the `Ffv*` forms, `PMom`, `Coupling` |

Why each instance leaves what it leaves:

- `Hels` is never emitted at compile time; it is the root the helicity expansion
  derives, read by every `eval_m2`.
- The SM writes γ⁵ as `ProjP − ProjM`, has no Levi-Civita vertex, no `Identity`
  bilinear (its Yukawas are `ProjM + ProjP`) and no cyclic index graph, so
  `Gamma5*`, `Epsilon*`, `IdentityAmp` and the tensor–tensor ops belong to the
  other censuses.
- FeynRules expands `σ^{μν}` into gamma chains before writing a UFO, so no
  SMEFTsim vertex has a literal `Sigma`. That is one reason the toy models exist
  ([toy UFO models](toy-ufo-models.md)): they cover `SigmaVout`, `SigmaOut` and
  the bare `Gamma5Amp`, and reach `FierzOut`/`FierzOutRev`/`FierzPair` and
  `IdentityAmp`. The three `Sigma` ops still unreached rest on
  `literal_sigma_currents_are_rooting_invariant` and the kernel identities
  `SigmaVoutRev = −SigmaVout`, `SigmaOutRev = −SigmaOut`[^n35-toy].
- `MultivectorIout/Oout` need a tensor contact on an internal line; the one
  SMEFTsim four-fermion tensor row saturates its four external legs and has only
  the amplitude-sink rooting. They rest on the identity
  `ψ̄ (M ψ) = (ψ̄ M) ψ = ⟨fierz(ψ̄, ψ), M⟩` and on
  `tensor_four_fermion_currents_are_rooting_invariant`.
- Every toy coupling scales a product beside a constant coefficient, so constant
  collection folds it into a composite pool entry; the bare `Coupling` leaf is
  the SM census's to cover.

Rows covering ops: [SMEFTsim and toy rows](non-sm-rows.md).

## What these checks cannot see

- The harness certifies that two kernels agree; a convention error shared by the
  fused and generic paths passes. The MadGraph amplitude gates cover that.
- The census says an op is *reached* by a gated process, not that the process
  would fail if the op were wrong — a reached op whose contribution is a global
  sign, or numerically negligible at the banked points, is reached but not
  pinned. [Convention-channel coverage](convention-channel-coverage.md) is the
  finer instrument for signs.
- An op reached only by an `info` row (the SMEFTsim census counts gated rows) is
  covered only by its hermetic identity.

[^n13-3a]: Note 13 §3a.
[^n13-7]: Note 13 §7, Stage B.
[^n35-e1]: Note 35 E1.
[^n35-l1]: Note 35 L1.
[^n35-toy]: Note 35 §10.4, which counts a 19-op allowlist; the list in `tests/toy_models.rs` at `787070e` has 20 entries.
