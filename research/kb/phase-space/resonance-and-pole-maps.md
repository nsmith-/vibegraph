---
type: Algorithm
title: "Breit–Wigner, massless-pole and forced-resonance maps for timelike invariants"
description: "BW tan substitution, the logarithmic map for zero-width poles with its floor, ForcedResonances windows for decay chains (union over pairings), and how MadEvent's setgrid/cut_bw differ."
status: draft
tags: [breit-wigner, resonance, decay-chains, phase-space-map, madevent-parity]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-09}]
sources:
  - {id: n21-closeout, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/21-resonance-sampling-and-events-plan.md#L231-L299", title: "Note 21, resonance-sampling close-out (BW map, firing-test inventory)"}
  - {id: n21-production, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/21-resonance-sampling-and-events-plan.md#L300-L381", title: "Note 21, putting the sampler into production (massless pole, log map)"}
  - {id: n38-d3, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/38-process-grammar-sprint-plan.md#L689-L801", title: "Note 38 D3, decay-chain phase space, σ and the sampler ladder"}
  - {id: dc-rs, resource: "vibegraph-lib/src/phasespace/diagram_channel.rs#L1773-L1960", title: "bw_scale, LogMap, log_scale, draw_lo, windowed, draw_invariant, invariant_measure"}
  - {id: cuts-rs, resource: "vibegraph-lib/src/cuts.rs#L76-L140", title: "SMALL_WIDTH_TREATMENT, ForcedLine::mass_window, ForcedResonances"}
  - {id: maps-rs, resource: "vibegraph-lib/src/phasespace/maps.rs#L143-L170", title: "MapChoices::channel installs floors and forced windows"}
  - {id: mg-myamp, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/Template/LO/SubProcesses/myamp.f", title: "MadEvent myamp.f: cut_bw, set_peaks"}
  - {id: mg-setgrid, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/Template/LO/Source/dsample.f#L938-L1007", title: "MadEvent dsample.f, setgrid"}
measured:
  - {landed_in: 1539abc, pr: 12, command: "cli_decay_chain.rs σ rows, ten seeds at --target-rel 2e-3, against MadEvent 3.7.1 (decay_chain_sigma_reference.json)"}
  - {landed_in: 1539abc, pr: 12, host: "4 shared cores", command: "tests/decay_chain_ladder.rs (ignored), one seed, integration to 5e-3, 5000 unweighted events"}
---

A per-diagram channel draws each composite subsystem's invariant mass² `s`
over `[lo, hi]` with one of three maps, chosen by the propagator that sits on
it ([diagram channels](diagram-channels.md)). `draw_invariant` maps `x ∈ [0,1]`
to `s`; `invariant_measure` returns `ds/dx` at a realised `s`, so the channel's
density at a foreign point is the same function the draw used
(`phasespace/diagram_channel.rs`)[^dc-rs].

## The three invariant maps

| propagator on the invariant | map | measure `ds/dx` |
|---|---|---|
| finite width, `mΓ > 0` | Breit–Wigner: `s = m² + mΓ·tan θ`, `θ` uniform over `[atan((lo−m²)/mΓ), atan((hi−m²)/mΓ)]` | `[(s−m²)²/(mΓ) + mΓ]·(θ_hi − θ_lo)` |
| zero width with `m² ≤ lo` (the massless `γ*` of a lepton pair) | two-piece log map in `t = s − m²` (below) | `(t₀ − t_lo)/frac` below the floor, `t·ln(t_hi/t₀)/(1 − frac)` above |
| none, or zero width *inside* `[lo, hi]` | flat | `hi − lo` |

The BW map's density is `∝ 1/((s−m²)² + (mΓ)²)`, so it flattens a resonance
exactly; `bw_map_is_measure_preserving`, `bw_map_zero_variance_on_bw_integrand`
and `z_pole_histogram_matches_breit_wigner` pin the measure, the
zero-variance property and the line shape[^n21-closeout]. A zero-width pole
inside the range is a genuine singularity the log map does not regulate, and
the flat draw stands (`log_map_claims_only_zero_width_poles_below_threshold`).

**The log map.** A zero-width propagator contributes `1/(s − m²)²`, rising
without bound toward the lower edge. `log_scale` puts the log piece's start at

```text
floor = min(LOG_MAP_FLOOR_GEV2 = 10 GeV², t_hi/50),   t₀ = max(t_lo, floor)
```

mirroring the `10/stot` and `stot/50` terms of MadEvent's floor. The last
90% of `x` is uniform in `ln t` over `[t₀, t_hi]`; the first
`LOG_MAP_TAIL_FRACTION = 10%` covers `[t_lo, t₀]` linearly, so the map keeps
full support and stays unbiased. When the kinematic edge already sits at or
above the floor there is no sub-floor region, and `frac = 0`: a linear piece
over a zero-width interval would carry zero measure, giving infinite weights
and a zero density for any channel evaluating a foreign point
(`log_map_without_subfloor_region_stays_finite`)[^n21-production].

**The lower edge.** `draw_lo` raises the edge from the threshold `(Σmᵢ)²` to a
cut-implied floor where `with_timelike_floors` installed one, clamped below
`hi` ([cut-implied timelike floors](cut-implied-timelike-floors.md)).

## Why a massless pole needs its own map

A flat draw against the `1/(s−m²)²` rise is not merely inefficient. The
estimator acquires a tail heavy enough that a run either misses the region
(σ collapses) or catches it (σ inflates), and when iterations were combined by
`1/σ²` the missing iterations reported a small integral and a small variance
and dominated: one run read 25× low with a 5% error bar. A fixed-seed run of
`ee_to_mumu_tata_qcd0` showed an ordinary-looking pull of +3.19; only sweeping
seeds exposed it (`probe_resonant_seed_stability`). Diagnosis by
elimination:[^n21-production]

| hypothesis | test | result |
|---|---|---|
| α-adaptation collapsed a channel | `probe_alpha_collapse` | refuted: no channel at the floor, and uniform α collapses too |
| survey budget too small | 30k → 300k survey | refuted: it rescues one seed and breaks another; the failure moves |
| the low-`m_ll` photon pole | `probe_photon_pole_is_the_instability` | confirmed: an `mmll` cut 0 → 20 GeV takes the five-seed spread from 24.96× to 1.01×, χ²/dof 1159 → 1.18 |

A second defect sat on top: with the log map in, VEGAS at Lepage's damping
over-adapted the near-flat hypercube, which is why mapped integrands run at
`VEGAS_ALPHA_MAPPED = 0.5` ([VEGAS integrator](vegas-integrator.md)).

## How MadEvent does it

MadEvent shapes the **grid**, not the map. For an s-channel invariant with no
Breit–Wigner (zero width), `set_peaks` sets the grid floor at the invariant's
lower edge `xo = xm²/stot`, and only where that edge is zero at
`xo = MIN(10d0/stot, stot/50d0, 0.5)` (`myamp.f:454-455`, `:465-481`). The
`small_width_treatment` floor on `prwidth_tmp` applies to positive widths only
(`myamp.f:131-135`), so a zero-width line always takes this branch. It then
calls `setgrid`, which lays the bins out
logarithmically, `grid = xo**(1 − i/ngu)` over 90% of the bins, and reserves
the other ~10% to reach below `xo` (`dsample.f:938-1007`)[^mg-setgrid][^mg-myamp].
Its `gen_s` draw is flat for a zero pole. Doing it as an analytic map here
does not depend on having a per-channel grid, and the channel density stays
grid-free, which the mixture needs ([multichannel](multichannel.md)).

## Forced resonances in decay chains

A decay chain (`p p > t t~, t > w+ b, …`) forces its decay lines on shell
within `bwcutoff` widths ([decay chains](../process/decay-chains.md)).

**MadEvent's semantics, as read**[^mg-myamp][^n38-d3]. `cut_bw` loops over the
configuration's s-channel propagators; for a forced one (`gForceBW = 1`) it
tests `onshell = |√p² − M| < bwcutoff·prwidth_tmp` (`myamp.f:136-139`), with
`prwidth_tmp = max(Γ, M·small_width_treatment)` for `Γ > 0`
(`myamp.f:131-135`; the loop only visits propagators with `prwidth > 0`), and a point outside fails
`passcuts` before its matrix element is evaluated. On the phase-space side
`set_peaks` raises the forced invariant's lower edge to `M − bwcutoff·Γ`
(`myamp.f:403`) and draws it with `transpole` over its range. `cut_decays = F`
marks every descendant of a forced line `do_cuts = .false.`, removing its
single-leg cuts and pairwise `ΔR`/`mm` cuts; `ptll` and `mmnl` do not read
`do_cuts`. The window is evaluated per configuration.

**Here.** `cuts::ForcedResonances::of(diagrams)` holds each diagram's set of
forced lines (legs, mass, width), each distinct set once
(`cuts.rs`)[^cuts-rs]. `Cuts::compile_with` turns them into windows: strict
`<`, the width floored at `SMALL_WIDTH_TREATMENT · M = 1e-6·M`, a zero-width
line never cut. A point passes when **every line of some set** is inside its
window. With identical particles across decays (`e+ e- > z z, z > e+ e-`)
different diagrams force different leg pairings, and the union over pairings
is symmetric under the permutation and independent of the sampler, which
MadEvent's per-configuration reading is not. `cut_decays = F` uncuts the legs
a forced line produces in every set, and the windows also raise the `τ` floor.

On the channel side `MapChoices::channel` installs
`DiagramChannel::with_forced_windows`:[^maps-rs] each forced invariant draws
its Breit–Wigner over window ∩ kinematic range, the density reports exactly
zero outside (with a `WINDOW_EDGE_SLACK = 1e-9` relative slack so a point the
channel drew never reads zero after recomputation from momenta), and where the
remaining kinematic range cannot reach the window the full range is kept,
identically in draw and density. Non-chain cards are byte-identical with and
without this code.

**σ against MadEvent.** The comparison is with MadEvent's decay-chain σ, not
σ × BR: the window keeps `(2/π)·atan(2·bwcutoff)` of the Breit–Wigner, about
0.979 at `bwcutoff = 15`. MadEvent 3.7.1, 10k events per seed; here ten seeds
at `--target-rel 2e-3`:[^n38-d3]

| row | MadEvent | here | here/MG − 1 | pull |
|---|---|---|---|---|
| `e+ e- > z z, z > e+ e-, z > mu+ mu-` (500 GeV) | 9.4388e-4 ± 6.7e-7 | 9.4474e-4 ± 3.8e-7 | +9.1e-4 | +1.12 |
| the same, `cut_decays = T`, ptl 10, etal 2.5, drll 0.4 | 6.6379e-4 ± 1.5e-6 | 6.6608e-4 ± 2.8e-7 | +3.4e-3 | +1.5 |
| `e+ e- > t t~, t > w+ b, t~ > w- b~` | 0.526305 ± 1.5e-4 | 0.526359 ± 1.2e-4 | +1.0e-4 | +0.28 |
| `e+ e- > t t~, (t > w+ b, w+ > e+ ve), t~ > w- b~` | 5.7137e-2 ± 3.1e-5 | 5.7111e-2 ± 2.3e-5 | −4.4e-4 | −0.66 |
| `p p > t t~, t > b e+ ve, t~ > b~ mu- vm~` (13 TeV, μ = 173) | 6.4132 ± 7.1e-3 | 6.4091 ± 4.7e-3 | −6.3e-4 | −0.48 |
| `e+ e- > z z, z > e+ e-` (informational) | 4.7194e-4 ± 3.3e-7 | 4.7318e-4 ± 2.0e-7 | +2.6e-3 ± 0.8e-3 | +3.2 |

Two caveats come with the table. On the `cut_decays = T` row it is MadEvent
that misses: twenty seeds scatter by 1.0% against a quoted 0.23% with a
one-sided low tail, and two 50k-event runs land +0.06% from this side, so
MadEvent converges upward with budget. The last row is the pairing
difference: keeping both pairings and their interference puts the
identical-lepton σ 0.26% ± 0.08% above MadGraph's one pairing ÷ 2. `generate`
writes status-2 resonance records with mother pointers for a decay-chain card,
so a shower keeps the forced line shapes
([resonance records](../events/resonance-records.md)).

The forced invariants are mapped exactly; the decay angles are not. Unweighting
efficiency falls 2.6× (`z z`), 5.7× (`t t~`) and 16× (`t t~ h h`) against the
undecayed cores while the decayed `|M|²` costs 0.7–1.2× its core's, because
V−A and spin-correlation angular shapes are left to a factorised grid. That
cost, and why no MadSpin-style step is needed for correctness, is
[decay chains without MadSpin](../performance/decay-chains-without-madspin.md);
open work:
[decay angles drawn flat](../backlog/performance/decay-angles-drawn-flat-in-decay-chains.md).

## Related rows

`ee_to_mumua`'s photon-pole tail is a standing finding owned by validation:
its pull is reported, not asserted, against a fixed +1.04% offset that is the
reference's own ([gating exceptions](../validation/sigma-row-gating-exceptions.md)).
The pole-map reference survey is
[MadEvent phase-space maps](../references/codebases/madevent-phase-space-maps.md).

[^dc-rs]: `vibegraph-lib/src/phasespace/diagram_channel.rs`, module doc and the invariant-map functions.
[^n21-closeout]: Note 21 close-out, firing-test inventory.
[^n21-production]: Note 21, sampler-in-production addendum (Defect 1 and the degenerate case).
[^mg-setgrid]: MadEvent `dsample.f` `setgrid` at the pinned commit.
[^mg-myamp]: MadEvent `myamp.f` at the pinned commit: `cut_bw` from line 2, `set_peaks` from line 207.
[^n38-d3]: Note 38 D3.
[^cuts-rs]: `vibegraph-lib/src/cuts.rs`, `ForcedResonances` and `ForcedLine` docs.
[^maps-rs]: `vibegraph-lib/src/phasespace/maps.rs`, `MapChoices::channel`.
