---
type: Validation Methodology
title: Oracles below sigma for samplers and the hadronic integrand
description: "Samplers gate bit-for-bit where order allows and on analytic distributions, with sigma as backstop; the hadronic integrand has a pointwise re-derivation, fixed-s-hat slices and a DY cross-check."
status: draft
tags: [phase-space, sampling, hadronic, oracle, distributions]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-09}]
sources:
  - {id: n18-h, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/18-hadronic-xsec-design.md#L479-L911", title: "Note 18 §5 — RAMBO replay oracle, flat-MC normalisation, the pointwise DY oracle"}
  - {id: n21-regime, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/21-resonance-sampling-and-events-plan.md#L99-L118", title: "Note 21 — the sampler validation regime"}
  - {id: n21-close, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/21-resonance-sampling-and-events-plan.md#L231-L299", title: "Note 21 — Sprint A close-out and the hazard firing tests"}
  - {id: n24-p2d, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/24-user-distribution-and-proton-events-plan.md#L1447-L1501", title: "Note 24 P2d — in-session validation of the hadronic integrand"}
  - {id: code-proton, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/vibegraph-lib/src/proton.rs#L4332-L4480", title: "proton.rs — a_point_reproduces_an_independently_assembled_integrand"}
  - {id: code-slice, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/vibegraph-lib/src/proton.rs#L5258-L5370", title: "proton.rs — the fixed-energy slice test"}
  - {id: code-dy, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/vibegraph-lib/tests/validate_hadronic.rs#L588-L640", title: "validate_hadronic.rs — pointwise_integrand_oracle"}
  - {id: code-rambo, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/vibegraph-lib/tests/rambo_flat_mc.rs", title: "tests/rambo_flat_mc.rs"}
---

# Oracles below sigma for samplers and the hadronic integrand

σ agreement is a weak oracle for a sampler. A mis-sampled region of small
measure shifts σ smoothly instead of tripping a gate — MadGraph's own sampler
defects stayed latent for years that way — and VEGAS's iteration combination can
turn a missed region into a confidently wrong σ with a small error bar (see
[`AGENTS.md`](../../../AGENTS.md), "Samplers gate statistically"). So samplers
and the hadronic integrand are gated at finer levels first, and σ
([the σ gates](sigma-gate.md)) is the backstop.

## Samplers: three levels, finest first

1. **Bit-for-bit** where a pinned seed and an unchanged sampling order allow it —
   refactors of the phase-space seam, survey restructurings, thread-count
   invariance. Only then is bit-for-bit evidence at all.
2. **Distribution level** against analytic shapes: sampled invariant-mass and
   angular histograms against the analytic Breit–Wigner and t-channel oracles
   (the resonant line shape and an overlapping double peak both at χ²/dof ≈ 0.6
   when built). Comparison against MadGraph's own events is
   [the samples gate](samples-gate.md).
3. **σ within quoted MC uncertainty**, multi-seed — `validate_vegas.rs` and the
   σ rows[^n21-regime].

Each known sampler hazard gets a test that would fire if the map were wrong
(a passing σ is never accepted as confirmation of a convention)[^n21-close]:

| hazard | firing test |
|---|---|
| Breit–Wigner denominator / `ds/dθ` | `bw_map_is_measure_preserving`, `bw_map_zero_variance_on_bw_integrand` |
| t-channel invariant ordering | `spine_transfer_pairs_emitted_with_beam0`, `spine_emitted_is_forward_biased` (a silent swap flips the bias), `spine_built_for_real_t_channel_process` |
| threshold kinematics `s → (m₁+m₂)²` | `t_channel_threshold_window_collapses`, `t_bounds_include_initial_state_mass` |
| overlapping resonances | `overlapping_resonances_double_peak_resolved` (dropping the second channel collapses its coverage ~1000×) |

Each claimed variance win (Breit–Wigner map, multichannel combiner,
t-channel spine, α-adaptation) has its own strict-inequality test, so a win is
never mistaken for a convention check; the tests pin the direction, not the
synthetic integrands' magnitudes. The maps are
[resonance and pole maps](../phase-space/resonance-and-pole-maps.md) and
[multichannel](../phase-space/multichannel.md).

**Normalisation at the finest analytic level.** The massive RAMBO map is pinned
by a uniforms-replay oracle (`rambo_oracle.rs`, eight cases dumped by pure-stdlib
Python under `validation/rambo/`: worst momentum 1.3e-15, weight 3.4e-16 relative)
plus conversion goldens for the bits → uniform rule, which freeze stream addressing
and draw order. `flat_mc_two_body_normalization` integrates `σ(e⁺e⁻ → μ⁺μ⁻)` at
√s = 10 GeV by flat RAMBO against `4πα²/(3s)` (929.4 ± 0.5 against 928.9 pb), which
pins the phase-space volume and the `(2π)^{4−3n}` measure factor[^n18-h]. The
`#[ignore]`d `flat_mc_partonic_sigma` on the 2 → 6 row is an order-of-magnitude
check only: flat sampling of that collinear-peaked integrand is heavy-tailed and
its naive error understates the truth. See [RAMBO](../phase-space/rambo.md).

## The hadronic integrand

`ProtonIntegrand` ([the proton integrand](../hadronic/proton-integrand.md))
convolves partonic matrix elements with parton luminosities over the `(τ, y)` map,
with both beam orderings. Three oracles, each blind somewhere the others are not:

**The pointwise re-derivation** — `proton.rs`
`a_point_reproduces_an_independently_assembled_integrand`. On the ℓℓj card it
re-derives the `(τ, y)` map, both frames, the flux and the `2π` measure, forms
each member's `x·f` product directly from the PDF, and takes the mirrored
ordering from an **explicitly enumerated `b a > …` subprocess evaluated at the
unreflected point**. It shares neither `FlavorGroup::luminosity` nor
`FlavorGroup::mirror_into` with the integrand, so a dropped mirror, a mirror at
the wrong argument, a swapped beam ordering or a lost spin/colour average all
move it. It asserts non-vacuity: some points inside the cuts, some cut-rejected,
and the mirror carrying more than 10 % of a group's term. Bound `2e-12`; worst
`2.9e-13` over a twelve-seed sweep (the two sides multiply the same factors in a
different order). *Blind to* the phase-space weight (taken from its own copy of
the same channel construction) and to an error the cut filter and the PDF share.
The polarized-group variant asserts the matrix element is evaluated in the
partonic CM frame: the same assembly at the lab-frame point must disagree.

**The fixed-ŝ slice** —
`at_fixed_energy_the_integrand_is_the_partonic_cross_section_times_luminosity`.
At frozen `τ` and `y = 0` (the lab frame is then the partonic CM, so one cut
filter applies to both sides) the integrand's inner integral must equal
`Σ_g (L^direct + L^mirror)_g · σ̂_g(ŝ)`, with `σ̂_g` from `FixedBeamIntegrand`
through its *own* all-timelike per-diagram map, at √ŝ = 200 and 500 GeV so the
energy dependence is a shape. It compares flux, measure, spin/colour average and
symmetry factor across two independent phase-space maps (`rel < 3 %`,
`|pull| < 4`, set above a measured four-seed sweep and far below the factor of two
the smallest normalisation slip would produce). *Blind to* the rapidity boost
(switched off) and to the mirror's *argument*: at `y = 0` both orderings carry
equal luminosity and the reflection preserves measure and every cut observable,
so a mirror evaluated at the wrong point still integrates correctly[^n24-p2d].
Flat RAMBO is not usable as the partonic reference there (it misses the Z pole
inside the `mmll` window by 8–18 %), hence the multichannel side.

**Drell–Yan against MadGraph** — `validate_hadronic`'s
`pointwise_integrand_oracle` compares, at ~10 pinned `(x₁, x₂, cos θ)` points
(two straddling the `pT_ℓ = 10` GeV cut), the physics factors of the general path
— the `(τ, y)` map's `x₁, x₂, √ŝ` and Jacobian, each group's summed luminosity
over both orderings, each group's `|M(q)|²`, and the lab-frame cut indicator —
against an independent Python oracle (LHAPDF `xfxQ2` × MadGraph standalone
`|M|²`, `dy_integrand_oracle.json`, `pixi run -e madgraph generate-dy-oracle`)
at ≤ 1e-9, with MadGraph's exact param card bound. It does not compare the
assembled value (the oracle carries a flat-cos θ two-body measure the general
path does not use) and is blind to the mirror's pointwise argument, since the
oracle sums both luminosities against one `|M(q)|²`. The σ rows
`sigma_default_cuts_vs_mg` and `sigma_mmll_window_vs_mg` close the loop after
integration. The bespoke Drell–Yan integrand these once compared against no
longer exists; Drell–Yan runs through `ProtonIntegrand` like every other
hadronic process, and `generate` produces it.

What none of these can see: anything specific to a coloured initial state or a
three-body final state on the Drell–Yan rows, and frequencies of flavour or
colour labels (see [the event-output gates](event-output-gates.md)). The
phase-space seam and the hadronic sampling map are
[the channel contract](../phase-space/channel-contract.md) and
[hadronic (τ, y) sampling](../phase-space/hadronic-tau-y-sampling.md).

[^n18-h]: Note 18 §5, H3. The same section's H7 introduced the pointwise DY oracle.
[^n21-regime]: Note 21, "Validation regime".
[^n21-close]: Note 21, Sprint A close-out.
[^n24-p2d]: Note 24 P2d. Its slice bounds and the pointwise oracle's original 8.12e-14 have since become the code's `2e-12` bound over a twelve-seed sweep.
