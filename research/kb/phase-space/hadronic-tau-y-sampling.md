---
type: Design Decision
title: Sample hadronic collisions in (τ, y), with channels rebuilt at each event energy
description: "A mass window is a thin diagonal band in (x₁,x₂) that VEGAS misses (6% low); in (τ,y) it is a 1-D bound on τ. ScaledMultiChannel then draws at each point's √ŝ."
status: draft
tags: [phase-space, hadronic, vegas, change-of-variables, multichannel]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: n18-design, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/18-hadronic-xsec-design.md#L283-L311", title: "Note 18 §2.5 (hadronic assembly, direct x-map)"}
  - {id: n18-h7, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/18-hadronic-xsec-design.md#L479-L911", title: "Note 18 §5 decision records (H7: the (τ, y) remap)"}
  - {id: n18-outcome, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/18-hadronic-xsec-design.md#L935-L1039", title: "Note 18 outcome"}
  - {id: n24-p2, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/24-user-distribution-and-proton-events-plan.md#L879-L899", title: "Note 24 P2 (ScaledChannel / ScaledMultiChannel)"}
  - {id: n27-b2, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/27-v3-backlog-plan.md#L212-L297", title: "Note 27 B2 (hadronic ŝ floor)"}
  - {id: mg-setcuts, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064/Template/LO/SubProcesses/setcuts.f#L527-L707", title: "MadGraph setcuts.f (smin derivation)"}
---

# Sample hadronic collisions in (τ, y)

## Decision

A proton–proton run integrates over the two momentum fractions through

```
τ = ŝ/s = x₁x₂,        y = ½ ln(x₁/x₂)
x₁ = √τ eʸ,   x₂ = √τ e⁻ʸ,   dx₁ dx₂ = dτ dy
```

with τ drawn over `[τ_min, 1]` and `y` uniform over `[−y_max, y_max]`,
`y_max = ½ ln(1/τ)`. The remaining `3n − 4` coordinates go to the
multichannel, evaluated at that point's own `√ŝ = √(τ s)`.

## Why: the direct x-map misses thin bands

The obvious map,[^n18-design] VEGAS on `(u₁, u₂) → x_i = x_min^(1−u_i)` with points outside
the mass window rejected, makes a dilepton mass window a **thin diagonal band**
`x₁x₂ ∈ [ŝ_min, ŝ_max]/s` in the unit square. VEGAS's separable grid cannot
resolve a diagonal band: the `p p → e⁺e⁻` run with `m_ℓℓ ∈ [60, 120]` came out
**6% low (5.8σ)** and the default-cut run mis-converged by +0.78%. In `(τ, y)`
the window is a one-dimensional bound on τ, which VEGAS resolves trivially:
both runs then landed at about 0.1% of MadGraph with about ten times smaller
MC error.[^n18-h7][^n18-outcome]

The general lesson: when a VEGAS integrand has a thin kinematic band that is
not aligned with the coordinate axes, change variables so the band becomes a
bound on one coordinate, before spending iterations or tuning adaptation.

## The Jacobian

`ProtonIntegrand::map_point_with` (`vibegraph-lib/src/proton.rs:2009`) draws τ
under the run's `TauMap` (`phasespace/maps.rs:47`):

| `TauMap` | Draw | `(dτ/du)/τ` |
|---|---|---|
| `Log` (density ∝ 1/τ) | `τ = τ_min^(1−u₀)` | `ln(1/τ_min)` |
| `InverseSquare` (∝ 1/τ², MadEvent's `transpole(pole = −2)`) | `τ = 1/(1/τ_min − u₀(1/τ_min − 1))` | `τ (1/τ_min − 1)` |

and `jac = (dτ/du)/τ · 2 y_max`. The PDF grids return `x·f(x)`; the `1/(x₁x₂)`
in `f = (x·f)/x` cancels one power of τ, so the luminosity is built from `x·f`
products and only `(dτ/du)/τ` enters. Which τ map a run uses, and why `Log` is
the default, is [phase-space/map-choices](map-choices.md).

The scale probe that checks whether any point clears the factorisation-scale
floor always uses the `Log` map (`proton.rs:1846`): a steeper map crowds draws
at `ŝ_min`, where a card only partly below the floor would read as wholly below
it.

## τ_min: the cut-implied ŝ floor

`τ_min = Cuts::shat_min() / s` (`proton.rs:1614`). `shat_min` is a provable lower
bound on the `ŝ` of any point that survives the cuts (`cuts.rs:1142`):

```
ŝ_min = max( dsqrt_shat²,  mmll²  (if a same-flavour opposite-sign lepton pair exists),
             (Σ_i pT_i^min)²,  (Σ_i m_i)² )
```

The last two come from the partonic centre of mass, where `√ŝ = Σ E_i`: a boost
along the beam leaves each leg's `pT` unchanged, and `E_i ≥ max(m_i, pT_i)`. Both
hold for any multiplicity, without a back-to-back argument. They are the bounds
MadGraph's `setcuts.f` derives (`smin_p²` per letter class, `:527-676`, and
`max(smin, (Σ pmass)², dsqrt_shat²)`, `:702-707`; `genps.f:274` passes
`smin/stot` as τ_min).[^mg-setcuts] Two deliberate departures, both in the
direction of the derivation:[^n27-b2]

- MadGraph sums the transverse term per letter class and *adds* the classes;
  here it is summed over all legs at once, equal when one class is cut and
  tighter when several are.
- MadGraph's per-leg term is `max(e_X, pt_X, …)`; only the transverse threshold
  is used here, since an energy cut is a lab-frame quantity and the sum is
  taken in the partonic CM.

A massless, uncut final state gets `ŝ_min = 0` and therefore `ln(1/τ_min) = ∞`;
`dsqrt_shat` exists for that case. Before these general bounds, `p p > b b~`
reached an infinite `ln(1/τ_min)` and a `NaN` PDF call; for its banked card
(`ptb = 20`) the floor is `(2·20)² = 1600 GeV²`, MadGraph's value, and σ agrees at
−0.011%. Tighter, xqcut-implied floors on τ are discussed (and the dropped
one explained) in [phase-space/cut-implied-timelike-floors](cut-implied-timelike-floors.md).

## Channels rebuilt at each event's energy

In a hadronic run `ŝ` changes every point, so the channel maps cannot be
`Channel`s built at one `√ŝ`. A channel's structure (masks, masses, propagator
poles) does not depend on the energy, so the seam has an energy-parametrised
variant (`phasespace/channel.rs`):[^n24-p2]

- `ScaledChannel` (`:230`), a subtrait of `PhaseSpaceMap` (one `ndim` in
  scope), with `sample_at(sqrt_s, u)` and `density_at(sqrt_s, momenta)`.
  `DiagramChannel` implements both `Channel` and `ScaledChannel`
  (`diagram_channel.rs:1194`, `:1270`).
- `ScaledMultiChannel` (`:749`), the combiner over `ScaledChannel`s:
  `density_at(√ŝ, p) = Σ_j α_j g_j(p)` at that energy, `sample_channel_at`,
  `draw_in_channel_at`, `channel_weight_at`.
- `DiagramChannel` stores the beams' **masses** (`beam_masses`), not built beam
  momenta, and builds the beams at the draw's energy. In a proton run the
  partons are massless.

`kleiss_pittau_step` and `select_channel` are free functions, so the α
reallocation rule is written once for both combiners. Because the hadronic
integrand prepends its own `(τ, y)` coordinates, the unit hypercube VEGAS sees
is not the combiner's, and the α-adaptation survey is driven by the integrand
that owns the outer map ([phase-space/multichannel](multichannel.md)). The
density contract the scaled channels obey is the same as for fixed-energy ones
([phase-space/channel-contract](channel-contract.md)).

## Frames

`|M|²` is evaluated in the partonic CM with the beams along `±z`, the frame the
helicity-pruned evaluator requires and the frame the channel maps generate in.
The cut filter and the scale prescription read the **laboratory** frame, so the
outgoing momenta are boosted along `z` by the partonic rapidity `y` first.
Rapidity, `η` and `ΔR` change under that boost; `Δφ` and `pT` do not. (For a
back-to-back LO lepton pair `Δφ = π`, so a `drll` cut never fires there.) The
fixed-beam analogue, with massive beams, is
[phase-space/fixed-beam-kinematics](fixed-beam-kinematics.md); the luminosity
and the rest of the hadronic σ are
[hadronic/proton-integrand](../hadronic/proton-integrand.md).

[^n18-design]: Note 18 §2.5: the direct x-map as first designed.
[^n18-h7]: Note 18 §5, the H7 decision record: the 6% miss and the Jacobian.
[^n18-outcome]: Note 18 outcome: the load-bearing finding and its lesson.
[^mg-setcuts]: `setcuts.f` at `b7687064`, the `smin` derivation.
[^n27-b2]: Note 27 B2: the general ŝ floor and the `pp_to_bb_fixed` measurement.
[^n24-p2]: Note 24 P2: `ScaledChannel`, `ScaledMultiChannel` and per-energy beams.
