---
type: Design Decision
title: "Phase-space map choices: --map-* flags, the auto rules and the measurements behind them"
description: "MapOptions/MapChoices (split angle, τ map, rung order); auto = soft-emission / log τ / derived order, banked in the artifact; why soft-all and 1/τ² stay options, from 20-seed evaluations-to-0.1%."
status: draft
tags: [phase-space, maps, vegas, madevent, configuration]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: n37-survey, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/37-madevent-map-survey-and-soft-angle.md#L26-L68", title: "Note 37 §1 (MadEvent's maps against ours)"}
  - {id: n37-s3, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/37-madevent-map-survey-and-soft-angle.md#L162-L249", title: "Note 37 §3 (fixed-beam measurements; ee_to_mumua)"}
  - {id: n37-dec, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/37-madevent-map-survey-and-soft-angle.md#L268-L296", title: "Note 37 §4 (decisions)"}
  - {id: n37-s5, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/37-madevent-map-survey-and-soft-angle.md#L297-L359", title: "Note 37 §5 (the map choices as configuration)"}
  - {id: n37-s6, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/37-madevent-map-survey-and-soft-angle.md#L400-L541", title: "Note 37 §6 (hadronic measurements, the rules, the held-back cells)"}
measured:
  - {host: "4-core Linux container", command: "vibegraph integrate <proc> --run-card <gu_to_epemu run card> --target-rel 0.001 --seed {20260719,20,21}, lpp = 0, ebeam = 250"}
  - {commit: 02e8b25, host: "M3 Max", command: "vibegraph integrate <banked card> --target-rel 0.001 -j 16 --map-*, seeds 20260719-38 (160 seeds for g u > e+ e- u)"}
---

# Phase-space map choices

## The choices

Every choice here is a different parametrisation of the same phase space: the
estimator is unbiased under any of them, and what moves is the variance of the
weight, i.e. the evaluations a run needs to reach an accuracy. The types are in
`vibegraph-lib/src/phasespace/maps.rs`:

| Choice | Enum | Values | `auto` |
|---|---|---|---|
| 2-body split angle | `SplitAngle` | `Isotropic`, `Windowed`, `SoftEmission`, `SoftAll` | `SoftEmission` if any split has a single massless-vector daughter, else `Isotropic` |
| `τ = ŝ/s` draw (proton beams) | `TauMap` (`:47`) | `Log` (∝ 1/τ), `InverseSquare` (∝ 1/τ²) | `Log` |
| peripheral chain rung order | `RungOrder` | `Derived`, `Reversed` | `Derived` |

- `Windowed`: the isotropic angle confined to the angles at which both
  daughters clear their cut-implied energy floors (`Cuts::energy_floor`), on
  every split whose parent moves. The floor without the shape.
- `SoftEmission`: the soft-shaped `∝ 1/(E₁E₂)` map, inside that window, on the
  splits with a single gluon or photon daughter; isotropic elsewhere.
- `SoftAll`: the soft-shaped map on every split whose parent moves.

The angular map itself is [phase-space/soft-shaped-split-angle](soft-shaped-split-angle.md);
the τ map's Jacobians are [phase-space/hadronic-tau-y-sampling](hadronic-tau-y-sampling.md);
rung chains are [phase-space/t-channel-spine](t-channel-spine.md). On a `2 → 2`
process no split moves and every split-angle choice is the same map.

**Flags**: `vibegraph integrate --map-split-angle`, `--map-tau`,
`--map-rung-order`, each accepting `auto` (default) or a named value
(`vibegraph-cli/src/integrate.rs`, `MapArgs`).

**Plumbing**: `MapOptions` is what a caller asks (`None` = auto);
`MapOptions::resolve(&ProcessShape)` settles it into `MapChoices`.
`ProcessShape::of` reads the channels a decomposition builds, not the process
string: soft-emission splits, moving splits, the longest chain, and whether a
finite-width resonance spans the whole final state (the detector MadEvent's τ
rule would read; in place and pinned, unused by `auto`). `MapChoices::channel`
(`maps.rs:143`) is the one place a channel is built for integration or replay,
so an integrator and the generator replaying its grids cannot disagree.

**Banked**: `IntegrateArtifact::maps` records the settled choices and
`generate` rebuilds its channels and τ draw from them (`MapOptions::fixed`),
never from whatever the current default is. An artifact from before the
choices were banked reads back `MapChoices::LEGACY` (isotropic, log, derived),
which is what every such run used.[^n37-s5] The version that added this is in
`artifact.rs`'s `FORMAT_VERSION` doc comment
([pipeline/artifact-format-versioning](../pipeline/artifact-format-versioning.md)).
A mixed-multiplicity run settles one set over the union of its parts' shapes
([phase-space/mixed-multiplicity-integrand](mixed-multiplicity-integrand.md)).

## The rule is a measurement

A rule picks the option that measured best on the process class it recognises,
and falls back to the option every gated row was banked under where no
measurement exists. A rule with no measurement behind it is not written down
(`maps.rs` module doc). The figure of merit is the evaluations the convergence
stop needs to reach a scaled 0.1% ([phase-space/convergence-stop-rule](convergence-stop-rule.md)),
as a ratio to the map replaced.

### Twenty-seed hadronic measurements (M3 Max)

Seeds 20260719–38 per arm, banked cards, `-j 16`; σ per arm within 0.6σ of its
baseline.[^n37-s6] Where a row stops on the `--min-iters` floor on every seed
(`p p > b b~`), the ratio is of error² × evaluations instead.

| Row | Arm | Evaluations (M, mean ± sd) | Ratio to baseline |
|---|---|---|---|
| `p p > e+ e-` | log (baseline) | 2.14 ± 0.33 | — |
| | inverse-square | 2.28 ± 0.24 | 1.06 ± 0.05 |
| `p p > j j` | log (baseline) | 1.90 ± 0.22 | — |
| | inverse-square | 1.47 ± 0.14 | **0.77 ± 0.03** |
| `p p > b b~` | log (baseline) | 0.72 (floor) | — |
| | inverse-square | 0.72 (floor) | **0.65 ± 0.05** in error²·N |
| `p p > l+ l- j` | isotropic, log (baseline) | 18.3 ± 3.2 | — |
| | inverse-square | 18.8 ± 2.5 | 1.03 ± 0.05 |
| | windowed | 9.9 ± 3.0 | 0.54 ± 0.04 |
| | soft-all | 9.2 ± 0.8 | **0.50 ± 0.02** |
| | soft-all + inverse-square | 8.9 ± 0.5 | 0.48 ± 0.02 |
| `g u > e+ e- u` (160 seeds) | isotropic (baseline) | 1.06 ± 0.26 | — |
| | windowed | 0.96 ± 0.29 | 0.91 ± 0.03 |
| | soft-all | 0.90 ± 0.24 | **0.85 ± 0.02** |
| `g g > g u u~` | soft-emission (baseline) | 3.77 ± 0.97 | — |
| | soft-all | 3.85 ± 1.09 | 1.02 ± 0.09 |
| `u u~ > g g g` | derived rungs (baseline) | 3.23 ± 0.20 | — |
| | reversed rungs | 3.27 ± 0.26 | 1.01 ± 0.02 |

Readings:

- **llj's lever is the lepton pair.** `p p > l+ l- j` has no gluon-daughter
  split, so `SoftEmission` does nothing there; its one boosted composite split,
  `Z*/γ* → l⁺l⁻`, is where shaping pays. The window alone takes most of it, and
  the shape the rest with a far narrower seed spread (sd 0.8M against 3.0M):
  the isotropic draw spends its weight tail on leptons at the `pT` edge.
- **τ behaves as MadEvent's rule predicts.** `1/τ²` wins where nothing spans
  the final state (`j j`, `b b~`) and loses on Drell–Yan, where the Z peak sits
  in τ itself; neutral on llj.
- **A 2σ σ-pull that was noise.** On `g u > e+ e- u` both shaped arms read σ
  high: +1.6σ at 20 seeds, +2.2σ at 60, growing with statistics like a bias.
  160 seeds settled it at +0.36σ. Meanwhile the one mechanism that could bias a
  windowed map was pinned directly:
  `no_accepted_configuration_sits_below_an_energy_floor` (`cuts.rs:2041`) finds
  no accepted point below a subsystem's energy floor over 183,424 points in
  three boosted frames (closest approach 1.0007 of the floor).

### Fixed-beam measurements (4-core container, three seeds)

At fixed beams (`ebeam = 250`, MG default cuts, kT-clustered scale), evaluations
to 0.1% in millions, seeds 20260719/20/21:[^n37-s3]

| Process | isotropic | soft-emission, floored | soft-all, floored | soft-all, unfloored |
|---|---|---|---|---|
| `u u~ > g g g` (16 ch) | 5.04 / 4.80 / 4.80 | **3.48 / 3.36 / 3.12** | = soft-emission | 6.24 / 6.96 / 6.60 |
| `g g > g u u~` (16 ch) | 5.11 / 3.49 / 2.75 | 5.36 / 3.49 / 2.74 | 3.12 / 5.74 / 3.87 | 6.99 / — / — |
| `g u > e+ e- u` (4 ch) | 1.20 / 0.96 / 1.08 | bit-identical to isotropic | 1.20 / 0.72 / 0.72 | 1.44 / 0.84 / 0.84 |

- On `u u~ > g g g` (`g* → g g`, the `P_gg` case) the rule spends about a third
  fewer evaluations on every seed; σ unchanged across arms.
- **Unregulated, the shape is a loss** (+24% to +45%): `1/(E₁E₂)` runs far below
  the `ptj` threshold and spends its draws on rejected points. The cut-implied
  floor is what makes a `1/x` map pay,[^n37-dec] as for the invariant draw
  ([phase-space/cut-implied-timelike-floors](cut-implied-timelike-floors.md)).
- `SoftEmission` is inert where it selects no split: `g u > e+ e- u`
  reproduces isotropic bit for bit.
- The one gated Standard-Model row the rule touches is `ee_to_mumua`
  (`μ* → μ γ` splits). Over five seeds at 80,000 × 8 its quoted error is 12–21%
  smaller (variance −22% to −37%), χ²/dof 0.64–1.35, and its known +1.0% offset
  against MadGraph is unmoved
  ([backlog](../backlog/validation/ee-mumua-radiative-return-sigma-high.md)).

## Why `soft-all` and `1/τ²` are options, not defaults

Each measured improvement, made the default, took one banked gate cell over
its threshold, and each time a reference-free check said the map moved no
physics. Both cells are at-threshold statistics, which this repository settles
by matching the statistic to its calibration, never by widening; that decision
is the user's, so the rules stay where they validated and both maps ship as
options. Flipping either is a one-line change to `resolve`.[^n37-s6]

- **`soft-all` and `pp_to_llj_dyn`'s scatter guard.** The σ gate is fine
  (+0.23%, pull +0.69), but the five-seed χ²/dof guard (limit 4.0) reads 4.17:
  one seed sits ~3.6σ below the other four under *every* map. Forty seeds at
  the gate's configuration read χ²/dof 0.91 under both maps; the gate's five
  seeds drew a one-in-eight quintet
  (`probe_llj_dyn_scatter_guard_calibration`;
  [backlog](../backlog/validation/llj-dyn-scatter-guard-under-soft-all.md)).
- **`1/τ²` and `pp_to_jj`'s samples cell.** The flavour χ² against MadGraph's
  banked events crossed its `1e-4` floor on one seed (p = 2.8e-5); the three-seed
  sum was 325/212 against 267/210 under `log`, with the reference's own χ²
  already high. Two 200,000-event samples of our own, one per map, agree at
  χ² 40.5/47 (same-map controls 47.6/47 and 47.7/46; effective sample sizes
  ≈19,700 of 20,000 on both)
  ([backlog](../backlog/validation/jj-samples-flavour-chi2-under-tau-rule.md)).

Two three-seed readings did not survive twenty seeds: reversed rungs "4–14%
better" (1.01 ± 0.02 at twenty) and the lepton-pair window "worse than the
shape's floor alone". Three seeds do not set a rule; read a rung-to-rung or
arm-to-arm difference against a measured spread, as `AGENTS.md` requires.

## Under `1/τ²`, the setup probe draws τ logarithmically

`ProtonIntegrand::probe_scale` checks at setup that some cut-passing point
clears the factorisation-scale floor. Drawing through a `1/τ²` map crowds the
probe at `ŝ_min`, so a card whose support is only partly below the floor read
as wholly below it and was refused. The probe asks about support, not sampling,
so it always uses the `Log` map (`proton.rs:1846`), and
`both_tau_maps_integrate_a_known_function` (`proton.rs:3122`) pins both maps'
Jacobians against `∫ τ^(−1/2) dτ/τ` to 0.2%.

## MadEvent's maps, and what is still open

MadEvent's generator is the same recursive 2-body decomposition and
importance-samples by analytic `transpole` maps and pre-warped VEGAS grids
(`setgrid`). It has **no** shaped decay angle (flat `cos θ`, `φ`), uses `1/τ²`
unless a resonance spans the final state, and alternates beams on a t-channel
chain (`tstrategy` ping-pong) for ladders of three or more transfers with
massless ends.[^n37-survey] The full survey is
[references/codebases/madevent-phase-space-maps](../references/codebases/madevent-phase-space-maps.md).
Open items it named:

- ping-pong rung order for ≥ 3-rung ladders
  ([backlog](../backlog/performance/tchannel-pingpong-for-three-rung-ladders.md));
- a one-sided `1/E_g` shape for `q* → q g`, and llj's `1/(ŝ − ŝ_rest)`
  soft-gluon structure on the spine's remainder invariant, which would be a
  second channel per spine rather than a change to an existing draw
  ([backlog](../backlog/performance/soft-gluon-map-shapes-missing.md));
- MadEvent bins each invariant on the absolute dimensionless invariant, where
  this code bins the window-relative position
  ([backlog](../backlog/performance/vegas-grid-coords-not-absolute.md)); the
  per-channel grids are [phase-space/per-channel-vegas-grids](per-channel-vegas-grids.md).

[^n37-survey]: Note 37 §1: MadEvent's maps against ours, at the pinned tree.
[^n37-s3]: Note 37 §3: fixed-beam evaluations to 0.1% and the `ee_to_mumua` row.
[^n37-s6]: Note 37 §6: the twenty-seed hadronic measurements, the rules and the two held-back cells.
[^n37-dec]: Note 37 §4: the soft-emission rule adopted; `soft-all` everywhere and the unregulated map not adopted.
[^n37-s5]: Note 37 §5: the choices as flags, banked in the artifact, `MapChoices::LEGACY`.
