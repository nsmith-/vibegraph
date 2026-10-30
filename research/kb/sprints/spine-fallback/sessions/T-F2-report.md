---
type: Session Report
title: "T-F2 report: the γγ → ℓℓ corner on DY"
description: "In progress. The campaign plan, written before the long runs."
status: draft
tags: [phase-space, spine, vbf, cms, overweight, sprint-e, report]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-30}
---
The session brief is [T-F2](T-F2.md).

## Campaign plan (written before launch)

**Harness.** A weight-ranked dump, committed on `sprinte-spine-fallback`
(`7263d4c8`) and merged into `/home/user/wt/TV2`. `generate` keeps its eight
heaviest overweight trials and, at debug level, splits each one's weight into
the grid's Jacobian per coordinate, the channel's α, the integrand `f`, the
mixture density `g` and the channels carrying it, and the partonic momenta.
It draws nothing extra, so a seed's sample is the same as before. The
before and after measurements both use this harness:

```
RUST_LOG=info,vibegraph::generate=debug vibegraph --no-tui -y generate <R1 grid> \
  --seed 7 --nevents 600 -o <tag>.lhe --force
```

Driver `T-F2-gen.sh`: one summary line per run in `T-F2-results-gen.txt`,
resumable by tag. Integrations use T-V2's `T-V2-run.sh` command line
(`--fixed-budget --neval 10000 --niter <K>`, `-j 4`).

**Costs** (measured by T-V2 on this host, idle): DY R1 integrate ≈ 400 s,
DY R4 ≈ 1150 s, generate 600 events ≈ 24 min on one thread. P-V2 shares the
cores for the first hour or two, so the first phases run slower.

| phase | runs | estimate |
|---|---|---|
| A. before: generate on T-V2's R1 grids, seeds 1–4, 4 jobs of 1 thread | 4 | 0.5–0.8 h |
| B. hermetic diagnosis and the fix, bias checks on `/home/user/wt/T` | – | 2 h, mostly dev |
| C. after: integrate R1 seeds 1–4, then generate as in A | 4 + 4 | 1 h |
| D. paired R1 → R4 climb: R4 seeds 1–12 (R1 read off iteration 3) | 12 | 4 h |
| E. gates, banked re-gates of changed rows | – | 1.5 h |
| total | | ≈ 9–9.5 h |

- R4's first three iterations are R1's bit for bit (T-V2), so phase D gives
  both rungs per seed from one run.
- The "before" climb is T-V2's seeds 1–12, same command and budget.
- If phase D runs late, it stops at 12 seeds and τ (brief §5) is not run.

## 1. Diagnosis at the point

**Harness check.** The before run on R1 seed 1 (generation seed 7) reproduces
T-V2's sample exactly: 324124 trials, largest w/w_max 1546.549, overweight
share 0.7513. The dump draws nothing, as intended.

### The dump (`T-F2-gen/before-r1-s1.log`)

A trial's weight is `J · αⱼ f/g`: `J` is the frozen grid's Jacobian, `αⱼ` the
channel's selection weight, `f` the integrand and `g = Σₖ αₖ gₖ` the mixture
density. The two heaviest trials:

| | heavy 0 (56% of the sample) | heavy 1 (7%) |
|---|---|---|
| subprocess (by momenta) | `u u > e+ e- u u` | `u u > ta+ ta- u u` |
| w/w_max | 1546.5 | 65.1 |
| drawn in | channel 2, merged over 29 (group, diagram) pairs, σⱼ 0.93 pb | channel 70 (τ part's channel 2), σⱼ 0.31 pb |
| αⱼ | 2.16e-2 | 4.07e-2 |
| VEGAS Jacobian, per coordinate | 3.78 = [0.45, 1.15, 2.22, 1.06, 0.94, 1.68, 1.56, 1.51, 1.03, 0.82] | 1.52, no factor above 2.3 |
| own share αⱼgⱼ/g | 0.80 (channel 3: 0.19) | 0.57 (channel 71: 0.40) |
| value αⱼf/g against the channel's w_max | 3.9e4 against 95.5 | 1.9e3 against 45.1 |
| u of the two straddling remainders | 0.028 (ŝ₁), 0.010 (ŝ₂) | 0.013 (ŝ₁), 0.94 (ŝ₂) |

Invariants, from the dumped momenta:

| | heavy 0 (e) | heavy 1 (τ) |
|---|---|---|
| √ŝ (GeV) | 1047 | 2079 |
| photon virtualities `t₁`, `t₃` (GeV²) | −10.7, −28.9 | −4.27, −44.1 |
| lepton line between the photons, `t₂ − m²` (GeV²) | **−0.66** (e⁺ side) | −91.8 (τ⁻ side), −3726 (τ⁺ side) |
| `ŝ₁` = M²(ℓℓ u₄), as a fraction of ŝ | 3.3e4 (3.0%) | 5.7e4 (1.3%) |
| `ŝ₂` = M²(ℓ u₄), as a fraction of ŝ₁ | 336 (1.0%) | 5.3e4 (93%) |
| mℓℓ (GeV) | 59.5 | 61.4 |

Channel 2 is the spine photon → lepton → photon with rungs emitting `u₃`,
`e⁺`, `e⁻` and recoil `u₄`: a two-sided γγ-fusion ladder, every spacelike line
mapped, each rung's pole floored at 10 GeV² (all three lines are massless, and
τ's m² = 3.16 is under the floor too). Its first two remainders, `ŝ₁` and `ŝ₂`,
bound no line of the diagram, and the chain drew them **flat**.

### The same points with no grid in between

`probe_the_photon_pair_corner_against_the_pole_scale` (ignored, in
`tests/diagram_channel.rs`): the same two points in the SM, the same channel
set at the point's √ŝ with uniform α, and plain draws from that mixture under
the DY acceptance (`mℓℓ > 50`, `mjj > 4`, quark pT > 2 GeV, the μF floor's edge).
For each point it reports `|M|²/g` as a multiple of the mixture's mean weight
and of the weights' 99.9% quantile, with 4·10⁵ draws an arm.
`T-F2-corner-probe2.log`:

| pole scale (GeV²) | remainders | e point / mean | e point / q99.9 | e rel. var | τ point / mean | τ rel. var |
|---|---|---|---|---|---|---|
| 10⁴ (default) | flat (as built) | **8.8e4** | **797** | 5.5e3 | 1.3e4 | 4.4e3 |
| 10⁴ | 1/ŝ | 3.5e3 | 20.9 | 915 | 673 | 288 |
| 10³ | flat | 1.7e4 | 152 | 3.1e3 | 6.0e3 | 8.7e3 |
| 10³ | 1/ŝ | 625 | 3.2 | 283 | 395 | 226 |
| 10² | flat | 1.2e4 | 109 | 4.9e3 | 8.4e3 | 1.1e3 |
| 10² | 1/ŝ | 475 | 2.2 | 306 | 487 | 297 |
| 10 | 1/ŝ | 659 | 2.9 | 381 | 661 | 396 |
| 1 | 1/ŝ | 905 | 3.9 | 473 | 950 | 499 |

The per-draw variances are themselves heavy-tailed estimates at 4·10⁵ draws.
Read them to a factor of about 2.

### Why the weight is large

| candidate | verdict | evidence |
|---|---|---|
| no channel maps both photon rungs | **no** | channel 2 is the two-sided γγ ladder and carries 80% of `g` at the point (57% on τ) |
| the channel's α | **no** | αⱼ = 0.022, normal for its region; the next channel (α 0.026) has 4× less density there |
| the VEGAS grid | **no** | Jacobian 3.8, no coordinate above 2.2 |
| the flat remainder draws | **yes, the dominant factor** | `ŝ₁` and `ŝ₂` at 3% and 1% of their flat ranges, and the integrand there goes as `1/ŝᵢ`. With no grid at all, the point sits 8.8e4× the mean under flat remainders and 3.5e3× under a 1/ŝ map (25×), on the τ point 19× |
| the floor sits above where the matrix element peaks | **yes, second** | the e point's lepton line is 0.66 GeV² off shell under a 10 GeV² floor, and its first photon is at 10.7. With 1/ŝ remainders, 10⁴ → 10³ GeV² takes the point from 3.5e3 to 625× the mean. The τ point gains 1.7×: its lepton line is regulated by m_τ |

**The hypothesis from the CERN-host note** (massless leptons regulated only by
the photon virtualities) checked at the points: both heavy events are the same
γγ → ℓℓ corner, one e and one τ. The lepton-collinear structure is on the e
event: its lepton line is 0.66 GeV² off shell. The τ event's lepton lines are
at −92 and −3726 GeV², regulated by the τ mass. What the two share is the soft
photon fractions (ŝ₁ at 3% and 1.3% of ŝ) and a photon virtuality near or
under the floor (10.7 and 4.3 GeV²). The other dumped trials, seeds 3 and 4,
peak at w/w_max 25 and 13. Seed 4's heaviest is drawn in a channel holding
0.7% of `g` at its point, which is ordinary multichannel behaviour.

**Not checked:** which MadEvent channel this corner is. MadEvent's G1 holds
48% of the DY e/μ part and climbs 17× over its first iterations, which would
fit. But `configs.inc` is not in this checkout, so G1's diagram is not
identified here.

## 2. The fix (`9dc80f04`)

The fix has two parts, both in `vibegraph-lib/src/phasespace/diagram_channel.rs`.

1. **Straddling remainders are drawn from a mixture.** Each remainder before
   the last rung of a spine (`R_i`, `i < r`) is drawn from an even mixture of
   the flat draw and a `1/ŝᵢ` log map: `draw_rest`, `rest_measure`,
   `REST_LOG_SHARE = 0.5`. The log map is the existing zero-width `log_scale`
   under a massless pole, with its 10 GeV² floor and linear tail. The walk and
   the density both use the mixture's density, so it is one map. Each rung
   records the choice in `SpineRung::rest_mixed`. It enters `map_key` (`M`/`F`)
   and `map_identity`, so the channel-map digest sees the change, which is
   intended. `with_flat_remainders()` rebuilds the flat draw for measurement.
   The hand-built `from_topology_ladder` / `from_topology_tchannel` keep flat
   remainders.
2. **`DEFAULT_POLE_SCALE` goes from 10⁴ to 10³ GeV²**, a pole floor of 1 GeV².
   It still applies only with no fiducial scale. The support bound is still
   only from `Cuts::spacelike_floor`, unchanged. The floor sits about 2.5·10⁷
   above a massless edge's rounding noise (≈4e-8 GeV² at ŝ = 2.5·10⁵).

**Why a mixture, not the 1/ŝ map alone.** The first version drew the
remainders on the log map alone. It cut the corner's variance 6× and `u c`'s
by 17%, but `e⁺e⁻ → e⁺e⁻µ⁺µ⁻` went from 90 to 252, with seed χ²/dof 2.0. The
default `|η_l| < 2.5` keeps that process's exchanges hard, so a pure 1/ŝ draw
spends its points where nothing passes. The even mixture's density is at least
half of each component's, so it loses at most 2× against the better one. It
measured 124 on ee → eeμμ, 1.4× the flat draw.

**Why not the brief's other options.**
- *A missing two-sided channel*: the γγ ladder exists (channel 2 above).
- *A floor per exchanged species*: with the mixture in place, one lower default
  serves both the photon rungs (4–11 GeV² at the heavy points) and the e line
  (0.66).

**Why 10³ GeV².** Relative per-draw variance, plain MC over the production
set, from `T-F2-sigma-probe2.log` (8 seeds × 2·10⁶) and
`T-F2-corner-probe3.log` (4·10⁵ draws):

| pole scale (GeV²) | 1 | 10² | 10³ | 10⁴ | 10⁵ | before (flat, 10⁴) |
|---|---|---|---|---|---|---|
| `u c > e+ e- u c` | 1040 | 606 | **433** | 355 | 784 | 539 |
| `e+ e- > e+ e- mu+ mu-` | 337 | 224 | **174** | 124 | 84 | 90 |
| DY e corner (`u u > e+ e- u u`, √ŝ 1047) | 620 | 356 | **273** | 664 | 2427 | 5483 |
| DY e corner, the point's weight / mean | 2.3e3 | 1.2e3 | **1.6e3** | 9.9e3 | 4.1e5 | 8.8e4 |
| DY τ corner (`u u > ta+ ta- u u`, √ŝ 2079) | 544 | 327 | **235** | 368 | 2079 | 4397 |

Against 10⁴, the choice costs `u c` 1.2× and ee → eeμμ 1.4×; against ee's best
(10⁵) it costs 2.1×. It gains the DY corner 2.4× in variance and 6× in the
corner point's weight. This is not tuned to MadGraph: no σ enters the choice.

## Mutations (`T-F2-mut.log`)

Each mutation is applied to `diagram_channel.rs`, the named tests run, and the
file restored from a backup. `cmp` reports it identical every time.

```
== MR: remainders flat in production       (spine_chain: rest_mixed: true -> false)
R1 seed 1, heaviest (56%): |M|²/g at the point over the mean weight, production 1.804e4, remainders flat 1.804e4 (bound 4e3)
the production chain weighs the γγ → ℓℓ corner at 1.804e4 times its mean
test the_photon_pair_corner_is_mapped_by_the_chain ... FAILED
== MS: default pole scale back to 1e4      (DEFAULT_POLE_SCALE = 1e3 -> 1e4)
R1 seed 1, heaviest (56%): |M|²/g at the point over the mean weight, production 1.065e4, remainders flat 1.096e5 (bound 4e3)
the production chain weighs the γγ → ℓℓ corner at 1.065e4 times its mean
test the_photon_pair_corner_is_mapped_by_the_chain ... FAILED
== M1: spine withheld past two legs with no scale   (&& (n_out == 2 || fiducial_scale > F::zero()))
u u~ > e+ e- g QCD=2 QED=2 diagram 0: no spine to check without a fiducial scale
assertion `left == right` failed: u u~ > e+ e- g QCD=2 QED=2: a 1-line peripheral diagram built 0 rungs with no fiducial scale
the production chain weighs the γγ → ℓℓ corner at 1.919e10 times its mean
test result: FAILED. 0 passed; 3 failed
```

On the committed tree the same test reads production 4.13e3 / flat 1.10e5
before the default moved, and passes after it. Its committed numbers are in
the gate log.

**One existing test changed.** `the_rung_ordering_test_fires_on_a_swapped_chain`
failed under the mixture. Its raw `ln|t|` occupancy bound (1.2% a bin,
calibrated on flat remainders) saw the derived chain's Z rung fall to 0.88%:
the mixture moves draws to small `ŝ₁`, which narrows that rung's window,
whatever the rung order. The test now draws flat remainders on both arms, so
the rung order is the only difference between them. Passing:
`derived … starved rungs []`, `reversed … starved rungs [0]`. Under production
maps that bin occupancy is lower. What a real matrix element sees of it is the
variance table above (`u c` has Z and W rungs).

### A key-encoding defect, found by the census and fixed before the gate

`9dc80f04` set `rest_mixed` on every rung, the last one included, where it is
never read, and wrote `M`/`F` into every spine's key. That changed the key of
every spine. Single-rung chains got new keys although their maps are
identical, so the digest would have refused their artifacts for no reason. The
census's first run found 42 "moved" rows, mostly single-rung.

The follow-up commit sets the flag only on rungs before the last and writes
`M` only when the flag is set, so an unchanged map keeps its key byte for
byte. `map_key_separates_every_field_the_map_reads` gains a "remainder
mixture" variation that pins the flag into the key.

**The DY campaign ran on `9dc80f04`** (binary `T-F2-bin-after`, TV2
`45d9fe28`). Its maps are the final commit's: the flag is read only through
`Spine::rest_mixed(i)`, which tests `i + 1 < rungs.len()`, so the last rung's
flag is dead in both. Only the key text differs. Those artifacts will not
replay under the final build's digest, which is intended for a key change.

## Banked rows whose maps moved

`probe_the_banked_rows_whose_maps_moved` (ignored, `tests/diagram_channel.rs`)
reads every row of `sigma_reference.json`, `decay_chain_sigma_reference.json`,
`onshell_veto_reference.json`, `grammar_sigma_reference.json` and
`mlm_sigma_reference.json`, with the row's own run card and overrides. For
every channel of every subprocess, at the row's own `Cuts::spacelike_floor`,
it compares production's `map_key` with the same diagram built with flat
remainders and, at fiducial scale 0, a 10⁴ GeV² pole scale. That is the map
before this session, exactly. This is a recorded comparison of map keys, not
an inference from a suite passing. `T-F2-gate-census.log`:

| row | channels moved | longest chain | fiducial scale (GeV²) | why |
|---|---|---|---|---|
| `ee_to_wpwm`, `ee_to_wpwm_cw`, `ee_to_wp0wm`, `ee_to_wp0wmt` | 1 of 3 | 1 | 0 | neutrino exchange re-poled 10 → 1 GeV² |
| grammar `wp_wm`, `wp0_wm`, `wpt_wm` | 1 of 3 | 1 | 0 | same |
| `ud_to_epemud_qcd0` | 13 of 21 | 3 | 400 | remainder mixture |
| `uux_to_ccx_emmm_qcd0`, `bbx_to_ccx_emmm_qcd0` | 48 of 579 / 615 | 2 | 400 | remainder mixture |
| onshell `uu_full`, `uu_tt`, `uu_tt_mg371` | 7 of 16 | 2 | 0 | remainder mixture |
| MLM `pp_to_ll_0j2j_mlm` | 288 of 792 | 2 | 100, 400 | remainder mixture |
| MLM `pp_to_ttx_0j1j_mlm` | 22 of 82 | 2 | 0, 900 | remainder mixture |

- **Unchanged:** 61 rows, every one with no multi-rung chain and either a
  fiducial scale or no light spacelike line.
- **Not censused:** `wpwm_to_wpwmz_cw`. Its card sets `nhel = 1`, which
  `RunCard::parse` refuses.
- **Census blind spot:** the decay-chain rows. The census compiles cuts on all
  legs, so it reads `zz_ee`, `zz_emu` at fiducial 100 GeV². T-F1 recorded
  production's floor there as 0 (`cut_decays = F` exempts the decay products),
  so their electron exchange moves 10 → 1 GeV². They are re-gated below on that
  record. `zz_emu_cut` (`cut_decays = T`) and `pp_ttx_lep` (top exchange,
  m² ≫ floor) do not move.

## 3. Bias checks

**Walk/density** (`a_spine_without_a_fiducial_scale_keeps_its_walk_and_its_density_in_step`,
`WALK_DENSITY_TOL = 1e-7` unchanged, final tree, `T-F2-gate-dc-final.log`):

```
no fiducial scale: 75 spines (39 multi-rung), 85 massless rungs (25 emitting a composite subsystem), pole floor 1e0 GeV²
no fiducial scale, 75 peripheral channels × 2000 draws: worst walk-vs-density gap 2.047e-8 (u d > e+ e- u d QCD=0 diagram 23); with the pole at 1e-20 GeV² the same draws break the identity 51800 times, 16718 of them on 16 of the 25 diagrams emitting a composite subsystem off a massless line
u d > e+ e- u d QCD=0: walk weight vs 1/density over every derived chain (23 of them multi-rung): worst 6.753e-9 relative
```

The 39 multi-rung spines draw their straddling remainders from the mixture,
so the sweep covers `draw_rest` and `rest_measure`. T-F1's long tail probe
(`probe_the_walk_density_tail_against_the_pole_scale`, 7.5M draws an arm) was
**not run**.

**σ level, plain MC, 8 seeds × 2·10⁶ an arm.**
- Flat RAMBO comes from `T-F2-sigma-probe.log` (8 × 4·10⁷). It is the same
  seeds and code as T-F1, and the numbers match T-F1's bit for bit.
- The mixture arms come from `T-F2-sigma-probe2.log`. That run's flat arm was
  cut to 1000 draws by a temporary local edit, so its ratio column there is
  meaningless; the RAMBO row below is the full run's.

| process | arm | mean ± spread | seed χ²/dof | against flat RAMBO |
|---|---|---|---|---|
| `e+ e- > e+ e- mu+ mu-` | flat RAMBO | 90.825 ± 0.107 | 0.41 | – |
| | production, pole scale 10³ (as committed) | 90.722 ± 0.318 | 1.02 | −0.31σ |
| | mixture, 10⁴ | 90.945 ± 0.215 | 0.73 | +0.50σ |
| | flat remainders, 10⁴ (before) | 90.778 ± 0.214 | 1.02 | −0.20σ |
| | mixture, scales 1 / 10² / 10⁵ / 10⁶ | 90.51 ± 0.51 / 90.89 ± 0.30 / 90.75 ± 0.28 / 91.22 ± 0.24 | 1.62 / 0.70 / 1.77 / 1.26 | −0.6 / +0.2 / −0.3 / +1.5σ |
| `u c > e+ e- u c` | production, 10³ | 8083.5 ± 29.5 | 0.48 | (RAMBO unconverged, T-F1) |
| | mixture, scales 1 / 10² / 10⁴ / 10⁵ / 10⁶ | 8081 ± 60 / 8050 ± 46 / 8058 ± 47 / 8061 ± 56 / 7996 ± 222 | 0.88 / 0.86 / 1.47 / 1.38 / 1.77 | spread 7996–8084 |
| | flat remainders, 10⁴ (before) | 8091 ± 52 | 1.24 | |

Every arm agrees with flat RAMBO on ee → eeμμ within 1.5σ. On `u c` the pole
scales, which share no floor, agree within 1σ of the production arm, as does
the flat-remainder map. That is T-F1's independent check, now also across the
remainder change.
