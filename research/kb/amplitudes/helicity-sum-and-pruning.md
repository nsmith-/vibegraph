---
type: Design
title: Helicity sum and zero-helicity pruning
description: "|M|² sums (not averages) helicities and colours, coherent over diagrams; prune_zero_helicities matches MadGraph's NHEL filter bit-for-bit and holds only in the partonic CM with ±z beams."
status: draft
tags: [helicity, pruning, evaluator, madgraph-parity, amp2]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: n10-hel, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/10-lorentz-runtime-eval-plan.md#L383-L395", title: "Note 10 §5.5: helicity iteration"}
  - {id: n10-open, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/10-lorentz-runtime-eval-plan.md#L579-L605", title: "Note 10 §11: coherent sum, gauge choice"}
  - {id: n15-23, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/15-eval-optimization-plan.md#L466-L545", title: "Note 15 §2.3: helicity filtering (prune_zero_helicities)"}
  - {id: n19-v1v2, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/19-validation-pass-plan.md#L79-L100", title: "Note 19 V1–V2: the frame guard; NHEL pinning 14/14"}
  - {id: n27-b6, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/27-v3-backlog-plan.md#L912-L1038", title: "Note 27 B6: per-diagram AMP2, and what pruning does to it"}
  - {id: code-prune, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/vibegraph-lib/src/helas/eval/compile.rs#L630-L780", title: "compile.rs: prune_zero_helicities, generic_probe_points, prune_zero_amplitudes"}
  - {id: code-guard, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/vibegraph-lib/src/helas/eval/run.rs#L939-L990", title: "run.rs: assert_partonic_cm_beams_along_z"}
  - {id: code-pin, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/vibegraph-lib/src/helas/eval/run.rs#L4890-L4970", title: "run.rs: prune_zero_helicities_matches_madgraph_filter_bitwise"}
---

# Helicity sum and zero-helicity pruning

## What `eval_m2` sums

`eval_m2` returns `Σ_hel Σ_ij CF_ij J_i J_j*`: the sum over every helicity
combination of the colour-contracted square of the JAMPs, each JAMP a **coherent**
sum of diagram amplitudes. Diagrams are added before squaring
(`|M_γ + M_Z|²`, never `|M_γ|² + |M_Z|²`).[^n10-open] Helicities and colours are
**summed, not averaged** — MadGraph's `MATRIX1` convention; the `1/4`, `1/Nc²`,
`1/(Nc²−1)²` averages and identical-particle factors belong to the cross-section
layer.[^n10-hel] Massless vector propagators are in Feynman gauge (`−i g/q²`),
massive ones in unitary gauge (`−i(g − qq/m²)/D`).

All combinations are baked into one helicity-expanded arena under an `Op::Hels`
root, hash-consed so a current shared between combinations is computed once per
point ([performance/helicity-expansion](../performance/helicity-expansion.md)).
Polarized external legs restrict the combinations up front
([process/polarization](../process/polarization.md)).

## Pruning: MadGraph's filter, reproduced exactly

MadGraph never evaluates combinations that vanish identically:[^n15-23]

- standalone `SMATRIX`: evaluates all `NCOMB` for the first 20 calls and latches
  `GOODHEL` by the exact test `T .NE. 0D0`; disabled for `NEXTERNAL ≤ 3`;
- madevent: the same loop with a relative threshold
  `DABS(TS(I)) .GT. ANS*LIMHEL/NCOMB`, `LIMHEL = 1e-8`;
- helicity recycling (MadGraph ≥ 3): an init-mode survey bakes only the surviving
  `NHEL` rows into the generated `matrix1_optim.f`, plus per-(helicity, diagram)
  `ZEROAMP` skipping.

`AmplitudeEvaluator::prune_zero_helicities(&EvaluatedModel)` reproduces that
filter for the bound card:[^code-prune]

1. Probe the full expansion at 10 deterministic generic partonic-CM points: beams
   along ±z at two energy scales (3.7× and 11.3× the larger of the incoming and
   outgoing mass thresholds), five seeded massive-RAMBO final states each.
2. Keep every combination whose contribution exceeds `HEL_PRUNE_REL = 1e-24` of
   the helicity sum at any point; re-expand the arena over the survivors.
3. Within the survivors, remove the per-diagram operands that are still
   structurally zero (`prune_zero_amplitudes`, MadGraph's `ZEROAMP` layer),
   byte-for-byte with the full expansion.

It skips (returns 0) when `n_ext ≤ 3`, when the process is not 2 → n, and when no
combination or every combination survives — a card that zeroes the whole
amplitude stays visible instead of being pruned away.

**Why the threshold is safe.** The per-combination spectrum is bimodal.
Chirality-forbidden combinations are exactly `0.0` (structural zeros of massless
spinors propagate through the kernels); MHV-type zeros (all-plus gluons) cancel
across diagrams to ≲1e-30 of the sum; genuine contributions observed are ≳1e-12
even when doubly mass-suppressed. `1e-24` sits in the gap, and every dropped term
is below half an ulp of any partial sum, so **the pruned sum is bit-for-bit the
unpruned one** — a guarantee MadGraph's own `LIMHEL = 1e-8` does not give. A
symbolic zero test was rejected: it needs a hand-written zero-mask transfer
function per op (about twenty new convention hypotheses, each needing a pin),
while the numeric probe detects the same zeros (a rational function of the momenta
that vanishes at generic random points vanishes on the manifold, almost surely).

**The frame contract.** Some zeros are frame-bound, not identities. In
`g g > t t~`, same-helicity gluons with opposite-helicity tops vanish by `J_z`
conservation about the beam axis in the partonic CM only; massive-particle
helicity is not boost invariant, and a z-boost raises them from 1e-32 to 3e-3 of
the sum. MadGraph prunes them too, so a pruned evaluator takes **partonic-CM
momenta with beams along ±z** — the frame madevent, the VEGAS driver and the
validation samples evaluate in. `assert_partonic_cm_beams_along_z` checks it on
every pruned entry point (`eval_m2`, `eval_jamp2`, `eval_amp2`, …) in debug builds
and under the `extended-validation` feature; release builds pay nothing.[^code-guard][^n19-v1v2]

**Pins.** `prune_zero_helicities_matches_madgraph_filter_bitwise` asserts the
survivor counts of MadGraph's generated sources and bitwise pruned = unpruned
`eval_m2` on fresh points for 14 processes:[^code-pin]

| process | kept / all | process | kept / all |
|---|---|---|---|
| `e+ e- > mu+ mu-` | 4/16 | `e+ e- > mu+ mu- ta+ ta- QCD=0` | 16/64 |
| `u u~ > mu+ mu-` | 4/16 | `u u~ > c c~ e+ e- mu+ mu- QCD=0` | 16/256 |
| `e+ e- > e+ e-` | 6/16 | `b b~ > c c~ e+ e- mu+ mu- QCD=0` | 32/256 |
| `e+ e- > mu+ mu- a` | 8/32 | `u u~ > u u~` | 6/16 |
| `e+ e- > t t~` | 8/16 | `g g > t t~` | 12/16 |
| `e+ e- > w+ w-` | 16/36 | `g g > g g` | 6/16 |
| `e+ e- > z h` | 6/12 | `e+ e- > ta+ ta- h` | 8/16 |

The massive `b` keeps helicity-flip combinations (32 against the `u` class's 16).
`tests/amplitude_oracle.rs` additionally asserts per row that the helicity set is
MadGraph's own `NHEL` table and that the pruned evaluator matches the unpruned one
bit-for-bit at every point.

## Pruning moves the per-diagram AMP2

`|M|²` is protected because the dropped combinations are ~1e-30 of the *coherent*
sum. The *incoherent* per-diagram sums `AMP2_d = Σ_hel |A_d|²` have no such
protection: a combination dropped because its diagrams cancel still has non-zero
individual diagram amplitudes. Measured: `gg_to_ttx` **39.5%**, `gg_to_gg` **3.2%**,
every other row exactly 0.[^n27-b6] The production path draws configurations from
the pruned evaluator, the analogue of MadEvent's own `GOODHEL`-filtered
accumulation, and the measured `ICOLUP` frequencies agree with MadGraph's
(`gg_to_ttx` at p 0.46–0.71). `amplitude_oracle` measures the gap on every run
rather than assuming it away. How AMP2 feeds the configuration and colour-flow
draw: [per-diagram-amp2](per-diagram-amp2.md),
[events/colour-and-helicity-selection](../events/colour-and-helicity-selection.md).

The structure of the compiled program these sums run over:
[evaluator-architecture](evaluator-architecture.md).

[^n10-hel]: Note 10 §5.5.
[^n10-open]: Note 10 §11, items 2–3.
[^n15-23]: Note 15 §2.3.
[^n19-v1v2]: Note 19 §V1–V2.
[^n27-b6]: Note 27 §B6 outcome (2026-08-01), finding 2.
[^code-prune]: `vibegraph-lib/src/helas/eval/compile.rs`, `prune_zero_helicities`.
[^code-guard]: `vibegraph-lib/src/helas/eval/run.rs`, `assert_partonic_cm_beams_along_z`.
[^code-pin]: `run.rs` test module.
