---
type: Design Decision
title: Regulate massless t-channel transfers with a cut-implied spacelike floor
description: "Where cuts imply a floor ((max single-leg pT cut)²), bound each rung's t_max at −floor and keep a token pole; elsewhere use the flat fallback. With the before/after σ that decided it."
status: draft
tags: [t-channel, spine, cuts, conditioning, madgraph-agreement]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: n24-p0, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/24-user-distribution-and-proton-events-plan.md#L525-L618", title: "Note 24 P0, the three-body spine probe and what P2 must take from it"}
  - {id: n24-p2, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/24-user-distribution-and-proton-events-plan.md#L879-L1010", title: "Note 24 P2, per-energy channels, walk weight, the bias re-read, design decisions"}
  - {id: n24-p2b, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/24-user-distribution-and-proton-events-plan.md#L1049-L1092", title: "Note 24 P2b, Cuts::spacelike_floor(): the floor's scale is process data"}
  - {id: n28-dec, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/28-kt-spine-feature-sprint-plan.md#L254-L279", title: "Note 28 §6, decisions D2 and D3"}
  - {id: n28-d3, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/28-kt-spine-feature-sprint-plan.md#L1670-L1764", title: "Note 28 §S2.5, D3 decided by measurement"}
  - {id: n28-s4, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/28-kt-spine-feature-sprint-plan.md#L1827-L1988", title: "Note 28 §S4, spine in production: coverage, B1, B2, B4, C"}
  - {id: cuts-rs, resource: "vibegraph-lib/src/cuts.rs#L615-L653", title: "Cuts::spacelike_floor"}
  - {id: dc-rs, resource: "vibegraph-lib/src/phasespace/diagram_channel.rs#L50-L85", title: "diagram_channel.rs module doc, 'Regulating the spacelike pole'"}
  - {id: dc-chain, resource: "vibegraph-lib/src/phasespace/diagram_channel.rs#L1643-L1708", title: "POLE_FRACTION_OF_FIDUCIAL_SCALE, spine_chain"}
  - {id: dc-tmap, resource: "vibegraph-lib/src/phasespace/diagram_channel.rs#L2363-L2456", title: "t_pole_shapes, draw_t, t_measure, apply_fiducial_t_max"}
  - {id: hadronic-mc, resource: "vibegraph-lib/src/hadronic.rs#L2386-L2453", title: "FixedBeamIntegrand::use_multichannel"}
measured:
  - {command: "probe_fiducial_t_max_against_the_floored_pole_on_llj_cuts (tests/diagram_channel.rs), five seeds × 200 000 points per cut, √ŝ = 500 GeV"}
  - {command: "probe_qcd_seed_stability, probe_channel_map_degeneracy, every_bounded_channel_set_covers_its_own_fiducial_region (tests/validate_sigma.rs); probe_fiducial_bound_on_llj_fixed (tests/validate_hadronic.rs)"}
---

## The problem

A peripheral rung off a massless beam that exchanges a massless line has its
transfer's upper edge analytically on the pole, `t_max = m² = 0`, computed as
a cancelling difference of two large quantities ([t-channel spine](t-channel-spine.md)).
Two conditioning fixes pin that edge at exactly zero whenever the emitted
subsystem's invariant is *fixed* (a single on-shell leg): the Källén function
in its grouped form, and boosts out of a blob's rest frame with `γ = E/√s`.
The flat fallback then fires deterministically, which is why a `2 → 2` spine
is safe. A **composite** emitted subsystem leaves a drawn invariant cancelling
against `ŝ`, and the edge lands on either side of zero at rounding
scale[^dc-rs]. Over 20 000 recoil invariants at `s = 2.5e5` it landed below
zero 6 131 times, above 6 218 times and exactly zero 7 651 times, with
`|t_max| ≤ 4e-8`[^n24-p0]. (The test that recorded this is no longer in the
tree; `tests/diagram_channel.rs` still cites it by name.)

When it lands just below zero, `t_pole_shapes` switches the propagator draw on
with `N = ln(|t_min|/|t_max|) ≈ 30` e-folds reaching `|t| ~ 1e-11`, while the
density recomputes `t` from the momenta with a cancellation error of the same
size. The channel's own walk weight is fine — an unregulated three-body spine
weighting by its walk reproduces flat RAMBO's `V_3` to 1.003 — but a combiner
weights every point by `Σₖ αₖ gₖ` from the densities, and those no longer
describe the map that drew the point:[^n24-p2]

| spacelike pole, six llj cuts at √ŝ = 500 | worst walk-vs-density gap | non-positive self-densities |
|---|---|---|
| unregulated (`m = 0`) | 3.99e4 | 145 of 800 000 |
| floored at 5 GeV | 1.2e-8 | 0 |

A non-positive density at a point the channel generated is a zero in the
combiner's denominator. The statement is therefore "the unfloored spine breaks
the density contract a combiner rests on", pinned by
`an_unregulated_three_body_spine_breaks_the_density_a_combiner_weights_by` and
`an_unregulated_spine_breaks_the_positive_density_contract`
(`tests/diagram_channel.rs`) ([channel contract](channel-contract.md)).

## The floor's scale

The scale is process data, read off the compiled cuts:[^cuts-rs]

```rust
pub fn spacelike_floor(&self) -> f64   // cuts.rs:645
// (max over compiled single-leg cuts of pt_min)², or 0 when none is active
```

Reading it from the *compiled* cuts lets class membership decide which legs
count: a `pdg = 5` leg carries `ptj` at `maxjetflavor = 5` and nothing at 4,
where MadGraph makes it a b with `ptb = 0`. A peripheral emission off a
massless beam that puts a massless system at transverse momentum `pT`
transfers

```text
|t| = 2E_beam (m² + pT²) / (E + p_z) ≥ pT²
```

That is a bound wherever transverse balance ties the two sides of the rung
together — always, for a three-body final state, since the system opposite the
jet carries the jet's `pT` — and only a scale past three outgoing legs, where a
partition can balance internally. `a_transverse_momentum_threshold_bounds_the_transfer_it_implies`
(`cuts.rs`) computes `t` from momenta over 40 rapidities × 3 energies rather
than asserting the algebra[^n24-p2b]. The default card's `ptj = 20` gives
400 GeV², ten orders above the 4e-8 noise; `ptl = 10` alone gives
100 GeV². Under MLM matching the `ptj = xqcut` rewrite makes it `xqcut²`
([cut-implied timelike floors](cut-implied-timelike-floors.md)).
`ProtonIntegrand::spacelike_floor` (`proton.rs:1878`) and
`FixedBeamIntegrand::use_multichannel` both read it from their own cuts, so a
run cannot regulate at a scale its cuts do not have[^hadronic-mc].

## Decision

For every rung of a spine, when `Cuts::spacelike_floor() > 0`
(`spine_chain`, `diagram_channel.rs`):[^dc-chain]

- **bound the transfer** at `t_max ← −floor`. `apply_fiducial_t_max` applies it
  per rung and per configuration, and skips a rung whose kinematic window
  already lies below the bound or would be emptied by it, so a rung always
  keeps a window. The bound narrows the channel's *support*, and the density
  reports an exact zero above it;
- **floor the pole** at `t_mass² = max(m², 1e-3 · floor)`
  (`POLE_FRACTION_OF_FIDUCIAL_SCALE`), three orders below the bound and far
  above the cancellation noise, so a configuration the bound cannot narrow
  still draws against a well-posed pole. It enters `draw_t` and `t_measure`
  alike, so it reshapes the density and nothing else.

When the floor is 0 (no active single-leg `pT` cut) nothing is regulated: a
final state of more than two legs gets no spine, only the all-timelike tree,
and a `2 → 2` spine keeps the flat fallback at the collinear edge. Whether that
conservative fallback is still needed after the conditioning fixes is open:
[three-body spine requires a fiducial scale](../backlog/feature/three-body-spine-requires-fiducial-scale.md).

**Why the bound and not only the pole floor.** On the six single-spacelike-line
cuts of `u u~ > e+ e- g` and `g u > e+ e- u` at √ŝ = 500 with the default
cuts, integrand `cut · BW(s_ll) / t²` with the spacelike line left massless,
five seeds × 200 000 points:[^n28-d3]

| cut | var(all-timelike)/var(floored pole) | var(floored)/var(bounded) | cut efficiency floored → bounded |
|---|---|---|---|
| `u u~` 0 | 38.11× | 1.665× | 0.3843 → 0.4197 |
| `u u~` 1 | 39.07× | 1.833× | 0.7610 → 0.8381 |
| `u u~` 2 | 26.89× | 1.689× | 0.3880 → 0.4238 |
| `u u~` 3 | 41.74× | 1.834× | 0.7617 → 0.8378 |

(The `g u` cuts reproduce two of these exactly: four distinct configurations.)
All three arms agree within combined error on every cut, so no bias is bought.

**Coverage, because the bound narrows support.** A channel set whose members
each renounce part of phase space is unbiased only if together they reach
everywhere the integrand lives. Against flat RAMBO with the cut indicator as
the integrand, a combiner of bounded spines alone holds at the cut scale
(0.9σ at 400 GeV², 0.2σ at 4 000) and fails monotonically from 100× the cut
scale (4.6σ at 40 000, 34σ at 100 000), so the check fires and the installed
bound has an order of magnitude of margin. The banked gate
`every_bounded_channel_set_covers_its_own_fiducial_region` repeats this per
production row (100 000 flat draws): every accepted point is reachable on all
six affected rows. Five of them keep an unbounded member (an s-channel or
contact diagram), so their coverage is not a constraint; on
`ud_to_epemud_qcd0` all 35 channels are bounded chains, coverage is exact, and
pushing every bound out 100× loses 170 of 75 360 accepted points — the control
that shows the check sees the bound[^n28-s4].

The floor is provable but loose: on the llj configuration the accepted region
starts at `|t| ≈ 4 000–40 000 GeV²`, 10–100× above `pT_min²`, because central
leptons force the jet to recoil at large angle and the per-leg bound knows
nothing of that: [spacelike floor too loose](../backlog/performance/spacelike-floor-too-loose.md).

## Which rows it reaches

Every fixed-beam row whose cuts imply a floor *and* whose diagrams carry a
spacelike line changed its maps — six enforced partonic rows, not none:
`uux_to_uux`, `gg_to_gg` and `ud_to_epemud_qcd0` at 400 GeV² (`ptj 20`), and
`ee_to_ee`, `ee_to_mumua` and `ee_to_mumu_tata_qcd0` at 100 GeV²
(`ptl 10`)[^n28-s4]. Two disjoint control sets stayed bit-for-bit unchanged:
rows with peripheral diagrams but no floor (`gg_to_ttx`, `ee_to_wpwm`) and rows
with a floor but no spacelike line (`ee_to_mumu`, `uux_to_mumu`,
`ee_to_tatah`). Fixed seed, unchanged budgets:

| row | before: σ, pull, rel, χ²/dof | after |
|---|---|---|
| `uux_to_uux` | 2.818429e4 ± 5.129e1, −1.49, −3.00e-3, 1.76 | 2.825463e4 ± 2.172e1, −0.44, −5.08e-4, 0.76 |
| `gg_to_gg` | 1.427908e5 ± 3.660e2, +0.05, +1.45e-4, 1.16 | 1.427420e5 ± 1.400e2, −0.16, −1.96e-4, 0.56 |
| `ee_to_ee` | 1.556023e2 ± 9.323e-2, −0.83, −7.56e-4, 1.55 | 1.556415e2 ± 9.439e-2, −0.55, −5.04e-4, 1.64 |
| `ee_to_mumu_tata_qcd0` | 1.367003e-3 ± 2.685e-6, −1.45, −4.01e-3, 0.99 | 1.372287e-3 ± 2.078e-6, −0.06, −1.55e-4, 1.18 |
| `ee_to_mumua` | 1.007660e-1 ± 2.022e-4, +3.12, +9.67e-3, 0.97 | 1.006000e-1 ± 1.665e-4, +2.79, +8.01e-3, 0.72 |

Every moved row moved toward MadGraph and four of five shrank their error.
The comparison is the decision's evidence, which is why both columns stay.

- **`uux_to_uux`'s negative bias is gone.** Five seeds × two budgets
  (`probe_qcd_seed_stability`): the five-seed mean went from −0.30% to
  +0.019% (and −0.25% → +0.015% at 4× budget); worst |pull| 2.69 → 0.93. The
  bias was the spacelike collinear region drawn flat — every peripheral
  fixed-beam channel had been an isotropic 2-body split. The quoted error fell
  2.4× on `uux_to_uux` and 2.6× on `gg_to_gg`, 5.6× and 6.8× in variance at
  equal cost.
- **Degenerate maps differentiate.** `probe_channel_map_degeneracy`: the worst
  pairwise density difference went from 0 (bit-identical) to 1.000 on
  `uux_to_uux` and `gg_to_gg`; `α` moved to `[8.5e-6, 0.99999]` and
  `[3.2e-5, 3.2e-5, 0.496, 0.504]`. `gg_to_gg`'s two non-peripheral channels
  stay identical to each other, as expected with no spacelike line to act on.
  The control `gg_to_ttx` reproduced its earlier numbers digit for digit
  ([multichannel](multichannel.md)).
- **On `pp_to_llj_fixed`** (`probe_fiducial_bound_on_llj_fixed`, three seeds,
  300 000 × 10 each arm): bound on 423.3142 ± 0.2313 pb (−0.12% vs MadGraph),
  bound off (pole floor only) 422.5653 ± 0.2642 pb (−0.30%); 1.25–1.34× in
  variance per seed, less than on isolated cuts once VEGAS and 23 other
  channels sit between the map and the answer, and no bias bought.

`ee_to_mumua` stays the widest row, on an offset that is the reference's own
([gating exceptions](../validation/sigma-row-gating-exceptions.md)).

[^dc-rs]: `vibegraph-lib/src/phasespace/diagram_channel.rs`, module doc "Regulating the spacelike pole".
[^n24-p0]: Note 24 P0, the probe verdict, point 4.
[^n24-p2]: Note 24 P2, "the bias was an artefact of the weight definition".
[^cuts-rs]: `vibegraph-lib/src/cuts.rs`, `spacelike_floor` doc.
[^n24-p2b]: Note 24 P2b.
[^hadronic-mc]: `vibegraph-lib/src/hadronic.rs`, `use_multichannel` doc.
[^dc-chain]: `vibegraph-lib/src/phasespace/diagram_channel.rs`, `spine_chain` and `apply_fiducial_t_max`.
[^n28-d3]: Note 28 §S2.5.
[^n28-s4]: Note 28 §S4; the row floors checked against `validation/madgraph/scripts/ud_to_epemud_qcd0.mg5` (`ptj 20`).
