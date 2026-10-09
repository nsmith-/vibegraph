---
type: Algorithm
title: Soft-shaped 2-body split angle (SoftSplit) with energy floors
description: "Draws a split's angle from the parent's flight direction with density ∝ 1/(E₁E₂) inside cut-implied energy floors: the z(1−z) structure, the map, reciprocity, and where it fires."
status: draft
tags: [phase-space-map, soft-emission, splitting-kernel, energy-floor, map-choices]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: n37-z, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/37-madevent-map-survey-and-soft-angle.md#L71-L123", title: "Note 37 §2.1–2.2, the z(1−z) structure and the map"}
  - {id: n37-llj, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/37-madevent-map-survey-and-soft-angle.md#L124-L161", title: "Note 37 §2.3, where it can fire in p p > l+ l- j"}
  - {id: n37-meas, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/37-madevent-map-survey-and-soft-angle.md#L181-L230", title: "Note 37 §3.1, partonic evaluations to 0.1%, three seeds"}
  - {id: n37-dec, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/37-madevent-map-survey-and-soft-angle.md#L268-L296", title: "Note 37 §4, decisions"}
  - {id: n37-had, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/37-madevent-map-survey-and-soft-angle.md#L416-L466", title: "Note 37 §6.1, hadronic measurements at twenty seeds"}
  - {id: dc-soft, resource: "vibegraph-lib/src/phasespace/diagram_channel.rs#L1960-L2120", title: "SoftSplit, ShapedSplit, angle_window, shaped_split"}
  - {id: dc-with, resource: "vibegraph-lib/src/phasespace/diagram_channel.rs#L955-L995", title: "DiagramChannel::with_split_angles"}
  - {id: cuts-ef, resource: "vibegraph-lib/src/cuts.rs#L741-L756", title: "Cuts::energy_floor"}
  - {id: maps-rs, resource: "vibegraph-lib/src/phasespace/maps.rs#L26-L45", title: "SplitAngle and MapOptions::resolve"}
measured:
  - {host: "4-core Linux container, release build, -j 4", command: "vibegraph integrate <proc> --run-card <gu_to_epemu's banked run card> --target-rel 0.001 --seed {20260719,20,21}, ebeam = 250, lpp = 0"}
  - {commit: 02e8b25, pr: 7, landed_in: d54ae73, host: "M3 Max, -j 16", command: "the banked runs' own cards, --target-rel 0.001, seeds 20260719–38 per arm; g u > e+ e- u to 160 seeds"}
---

## The structure it targets

A 2-body split of a parent with CM energy `E`, momentum `P`, mass `M`
(`γ = E/M`, `β = P/E`) into daughters of rest-frame energies `a`, `a′` and
momentum `p*`, at rest-frame angle `θ*` from the parent's flight
direction, gives[^n37-z]

```text
E₁ = γ(a + βp* cos θ*),   E₂ = γ(a′ − βp* cos θ*),   z = E₁/E,   E₁E₂ = E² z(1−z)
```

A massless emission's splitting kernel (`P_gg ∝ 1/z + 1/(1−z)`,
`P_qq ∝ 1/(1−z_q)`) puts a `1/z` or `1/(z(1−z))` into `|M|²`, which an
isotropic draw leaves in the weight. Since `dz = (βp*/E) d cos θ*`, a density
`∝ 1/(E₁E₂)` in `cos θ*` is `dz/(z(1−z))` over the accessible `z` range — a
two-sided log map in the energy fraction, regulated by the pair's own mass
through `β < 1`. For a parent at rest it is the isotropic map.

The isotropic draw is not measured from the parent's flight direction:
`sample_branch` builds the rest-frame vector against the collision-CM axes and
boosts it, as MadEvent's `mom2cx`/`boostm` do. The energy fraction is then a
function of `(cos θ, φ)` *and* the boost — a correlation a product-form VEGAS
grid cannot learn, which is the reason to shape the map instead of leaving the
angle to the grid ([VEGAS integrator](vegas-integrator.md)).

## The map

`SoftSplit` (`phasespace/diagram_channel.rs`) draws
`w = ln((a + b·c)/(a′ − b·c))` uniformly, `b = βp*`, `c = cos θ*`[^dc-soft].
Since `dw/dc = b(a + a′)/((a + b·c)(a′ − b·c))`, the density is
`∝ 1/(E₁E₂)` and the measure replacing the isotropic `2` is

```text
Δw·(a + b·c)(a′ − b·c) / (b(a + a′))  =  Δw·E₁E₂·M / (b·E²)
```

Written against `w − ln(a/a′)` with `log1p`/`expm1`, it reaches the flat map
continuously as `b → 0`, so a parent at rest to rounding reads the same either
way (pinned at 1e-12); a parent exactly at rest returns the isotropic map.
`b < min(a, a′)` always, so every logarithm is finite.

**Energy floors.** The draw is confined to the window where both daughters
clear their cut-implied CM-energy floors (`angle_window`):

```text
c ∈ [ (e1_min/γ − a)/b ,  (a′ − e2_min/γ)/b ] ∩ [−1, 1]
```

`Cuts::energy_floor(slots)` supplies them: a lower bound on a subsystem's
energy in any frame reached from the lab by a boost along the beam, since each
leg's energy is at least its `pT` (boost-invariant) and at least `emin` in the
lab[^cuts-ef]. A floor outside the accepted region would bias σ, so the bound
must hold; `no_accepted_configuration_sits_below_an_energy_floor`
(`cuts.rs`) found no accepted point below a floor over 183 424 points in three
longitudinally boosted frames, closest approach 1.0007 of the
floor[^n37-had]. Configurations outside the window keep the window's positive
density rather than a zero: the same bound says they fail the cuts, so their
density is never read against a non-zero integrand[^dc-with].

`AngleShape::Windowed` is the window with a flat density; `AngleShape::Soft`
is the window with the `1/(E₁E₂)` density. The density side rebuilds `E₁`,
`E₂`, `E` from the momenta through the subsystem memo, as collision-CM
invariants, while the walk accumulates the measure from the drawn `c`, so
reciprocity remains a real check
(`soft_split_angles_stay_reciprocal_and_cover_the_same_volume`, worst 1e-9
over the topology spread, massless `V_n` reproduced). A rule that selects no
split leaves the channel bit-identical, map key included; a selected split
marks the key with its floors (`S(E₁ᵐⁱⁿ,E₂ᵐⁱⁿ)`), so `generate` refuses an
artifact trained under a different angular map[^n37-z].

## The rules, and where each fires

`SplitAngle` (`phasespace/maps.rs`):[^maps-rs]

| rule | shapes |
|---|---|
| `Isotropic` | nothing; the map every gated row was banked under, and MadEvent's |
| `Windowed` | every split whose parent moves, flat over the energy window |
| `SoftEmission` | splits with a single massless vector daughter (gluon or photon, `massless_vector_slots`), soft-shaped |
| `SoftAll` | every split whose parent moves, soft-shaped |

The soft shape applied to a split with no soft enhancement varies the weight
as `E₁E₂` where the integrand does not, so it is a worse map in principle.
`u u~ > g g g` (`g* → g g`, `P_gg`) and `g g > g u u~` (`q* → q g`, `P_qq`; its
`g* → u ū` is skipped by the soft rule) are the positive controls;
`g u > e+ e- u` is the negative one.

On `p p > l+ l- j` the soft-emission rule selects nothing: in `q q̄ → l+ l- g`
both diagrams are spines with the gluon as a rung leaf, its angle fixed by the
`1/t` draw and its energy by the remainder invariant; in `g q → l+ l- q` the
s-channel root `q* → q + (l+ l-)` is at rest in the collision CM. The one
boosted composite split in every llj channel is `Z*/γ* → l+ l-`, which only
`Windowed` and `SoftAll` touch[^n37-llj]. The soft-gluon structure of llj sits
elsewhere: `|M|² ∝ 1/(tu) = (1/t + 1/u)/(ŝ − s_ll)` puts the initial-state
`1/(z(1−z))` on the spine's remainder invariant as `1/(ŝ − ŝ_rest)`, a
different map that would compete with the Z/γ* pole on the same variable
([t-channel spine](t-channel-spine.md)).

## Measurements

Figure of merit: evaluations the convergence stop needs for a χ²-scaled 0.1%;
σ is the consistency check across arms. Partonic, three seeds, millions of
evaluations:[^n37-meas]

| process | isotropic | soft, floored | every split, floored | every split, unfloored |
|---|---|---|---|---|
| `u u~ > g g g` (16 ch) | 5.04 / 4.80 / 4.80 | 3.48 / 3.36 / 3.12 (−30 to −35%) | same as soft | 6.24 / 6.96 / 6.60 (+24 to +45%) |
| `g g > g u u~` (16 ch) | 5.11 / 3.49 / 2.75 | 5.36 / 3.49 / 2.74 | 3.12 / 5.74 / 3.87 | 6.99 (series stopped) |
| `g u > e+ e- u` (4 ch) | 1.20 / 0.96 / 1.08 | bit-identical | 1.20 / 0.72 / 0.72 | 1.44 / 0.84 / 0.84 |

No arm moved a σ by more than its own error. **Unfloored, the map is a
loss:** the `1/(E₁E₂)` shape runs down to `z_min ~ m₁₂²/(4E²)`, orders below
the `ptj = 20` GeV threshold, and spends most of its draws on rejected points.
`g g > g u u~` is neutral under the soft rule: its `q* → q g` splits sit on
channels carrying little of the mixture, and the symmetric map spends half its
attention on the quark's end, which `P_qq` lacks.

Hadronic and partonic, twenty seeds per arm, ratio of mean evaluations to the
baseline:[^n37-had]

| row | arm | evaluations (M) | ratio |
|---|---|---|---|
| `p p > l+ l- j` | isotropic (baseline) | 18.3 ± 3.2 | — |
| | windowed | 9.9 ± 3.0 | 0.54 ± 0.04 |
| | soft-all | 9.2 ± 0.8 | **0.50 ± 0.02** |
| `g u > e+ e- u` (160 seeds) | isotropic | 1.06 ± 0.26 | — |
| | windowed | 0.96 ± 0.29 | 0.91 ± 0.03 |
| | soft-all | 0.90 ± 0.24 | 0.85 ± 0.02 |
| `g g > g u u~` | soft-emission | 3.77 ± 0.97 | — |
| | soft-all | 3.85 ± 1.09 | 1.02 ± 0.09 |

**llj's lepton pair is where the angle pays.** The window (both leptons held
above `ptl`) takes most of the gain and the shape the rest, with a far
narrower seed spread (sd 0.8M against 3.0M): the isotropic draw spends its
weight tail on leptons at the `pT` edge. So the angle *is* where a large part
of llj's residual lives, through the lepton pair rather than a gluon daughter;
the soft-emission rule, which only looks for gluon and photon daughters, is
inert on llj by construction and cannot show it, and three-seed readings were
too few to separate the arms.

**A 2σ pull that was noise.** On `g u > e+ e- u` both shaped arms read σ high
against isotropic, +1.6σ at 20 seeds and +2.2σ at 60, the growing-with-
statistics signature of a bias. The only mechanism that could bias the map,
an accepted point below an energy floor, was pinned absent (above), and 160
seeds settled it at +0.36σ ([seed sweeps](../validation/seed-sweeps-and-budget-ladders.md)).

## What production uses

`MapOptions::resolve` picks `SoftEmission` wherever some split has a single
gluon or photon daughter, else `Isotropic`; under it every gated row but
`e+ e- > mu+ mu- a` draws bit-identically to isotropic. `SoftAll` measures
better where it differs but is not the rule, because under it
`pp_to_llj_dyn`'s five-seed scatter guard reads 4.17 against its 4.0 limit,
while forty seeds at the gate's configuration read χ²/dof 0.91 under either
map — a decision about that cell, not the map. The decision and its flags are
[map choices](map-choices.md); the held-back cell is
[llj_dyn scatter guard under soft-all](../backlog/validation/llj-dyn-scatter-guard-under-soft-all.md).
Not adopted: shaping every split by default, and the unregulated map[^n37-dec].
Open shapes: a one-sided `1/E_g` map for `q* → q g`, and llj's
`1/(ŝ − ŝ_rest)` on the spine remainder:
[soft-gluon map shapes missing](../backlog/performance/soft-gluon-map-shapes-missing.md).
Related: [resonance maps](resonance-and-pole-maps.md) for the invariant draws
the same splits use.

[^n37-z]: Note 37 §2.1–2.2.
[^dc-soft]: `vibegraph-lib/src/phasespace/diagram_channel.rs`, `SoftSplit` doc.
[^cuts-ef]: `vibegraph-lib/src/cuts.rs`, `energy_floor` doc.
[^n37-had]: Note 37 §6.1.
[^dc-with]: `vibegraph-lib/src/phasespace/diagram_channel.rs`, `with_split_angles` doc.
[^maps-rs]: `vibegraph-lib/src/phasespace/maps.rs`, `SplitAngle` and `MapOptions::resolve` docs.
[^n37-llj]: Note 37 §2.3.
[^n37-meas]: Note 37 §3.1.
[^n37-dec]: Note 37 §4.
