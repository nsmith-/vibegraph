---
type: Session Report
title: "T-F2 report: the γγ → ℓℓ corner was the flat draw between rungs; a flat/1/ŝ remainder mixture and a 10³ GeV² pole scale"
description: "DY's 1547× event is drawn by the γγ ladder channel itself. The weight comes from the flat draw of the invariants between rungs (1/ŝ there) and a lepton line 0.66 GeV² off shell under the 10 GeV² floor. Those remainders are now drawn from an even flat/1/ŝ mixture, and the default pole scale is 10³ GeV². On DY at R1 the largest w/w_max falls from 13–1547 to 6–18. Over 12 seeds the seed spread falls 2–3× and the paired R1 → R4 climb from +0.54 ± 0.27 to +0.32 ± 0.12 pb: smaller, not stopped. Bias checks hold, spine_sigma still fails under M1, and 15 banked rows move and pass their re-gates."
status: draft
tags: [phase-space, spine, vbf, cms, overweight, sprint-e, report]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-30}
---
The session brief is [T-F2](T-F2.md). Code commits on `sprinte-spine-fallback`:
`7263d4c8` (the weight-ranked dump), `9dc80f04` (the remainder mixture and the
10³ GeV² default), and `d54f0e46` (the mixture tied to chain positions, the
key written only when set, and the census probe). The report commits follow
them.

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

## What actually ran

| phase | ran | wall (UTC) |
|---|---|---|
| A. before generations, R1 seeds 1–4 | 4 of 4 | 02:07–04:38. Seed 2 took 2.5 h at efficiency 3.7e-4; the host was shared with P-V2 and with this session's probes |
| B. diagnosis, fix, mutations | – | 02:30–04:05 |
| C. after: R1 seeds 1–4, then generations | 4 + 4 of 4 + 4 | 04:04–05:43 |
| D. R4 seeds 1–12 | 12 of 12 | 04:58–09:37 |
| E. gates, census, re-gates | all listed below | 04:08–05:53, with one restart after ENOSPC |

Total, from first action to the last result: about 8 h.

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

## 2. The fix (`9dc80f04`, corrected in `d54f0e46`)

The fix has two parts, both in `vibegraph-lib/src/phasespace/diagram_channel.rs`.

1. **Straddling remainders are drawn from a mixture.** Each remainder before
   the last rung of a spine (`R_i`, `i < r`) is drawn from an even mixture of
   the flat draw and a `1/ŝᵢ` log map: `draw_rest`, `rest_measure`,
   `REST_LOG_SHARE = 0.5`. The log map is the existing zero-width `log_scale`
   under a massless pole, with its 10 GeV² floor and linear tail. The walk and
   the density both use the mixture's density, so it is one map. Each rung
   records the choice in `SpineRung::rest_mixed`, which is set on every rung
   but the last; `with_rung_order` re-derives it by position. `map_key` writes
   an `M` only when it is set, and `map_identity` carries it, so the
   channel-map digest sees the change, which is intended, while a chain with
   no straddling remainder keeps its key. `with_flat_remainders()` rebuilds the
   flat draw for measurement. The hand-built `from_topology_ladder` /
   `from_topology_tchannel` keep flat remainders.
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

## Mutations (`T-F2-mut.log` on `9dc80f04`, `T-F2-mut-final.log` on `d54f0e46`)

Each mutation is applied to `diagram_channel.rs`, the named tests run, and the
file restored from a backup. `cmp` reports it identical every time. The three
were run on both commits with the same outcome and the same numbers. On
`d54f0e46` MR replaces `rest_mixed: i + 1 < sides.len()` with `false`. The
output shown is the final commit's.

```
== MR: remainders flat in production       (spine_chain: rest_mixed: i + 1 < sides.len() -> false)
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

On the committed tree the same test passes, reading
`production 1.586e3, remainders flat 1.804e4 (bound 4e3)`
(`T-F2-gate-dc-final.log`).

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

### Re-gates of the moved rows

Each re-gate ran as a separate detached target on `cd8775e0`'s tree, with
`--profile release-debug --config 'profile.release-debug.debug=0'` and
`--features extended-validation`, under each target's own seed policy.
`T-F2-regate-summary.log`:

```
decay-chain rc=0 wall=390s   cargo test -p vibegraph --test cli_decay_chain
onshell-uu-tt rc=0 wall=300s cargo test -p vibegraph --test cli_onshell_veto -- a_top_pair
sigma rc=0 wall=1053s        cargo test -p vibegraph-lib --test validate_sigma    → test result: ok. 8 passed; 0 failed; 33 ignored
samples rc=0 wall=373s       cargo test -p vibegraph-lib --test validate_samples  → test result: ok. 6 passed; 0 failed; 2 ignored
```

"Before" is T-F1's recorded after-values, the tree this session started
from.

| row (gate) | before (T-F1's after) | after (this session) |
|---|---|---|
| `ee_to_wpwm` (validate_sigma, 5 seeds) | 7.196160 ± 1.56e-3, pull +0.47, χ² 0.55 | 7.196325 ± 1.56e-3, pull +0.51, χ² 0.55 |
| `ee_to_wpwm_cw` | 7.287786 ± 1.62e-3, +0.39, χ² 0.38 | 7.287743 ± 1.62e-3, +0.38, χ² 0.42 |
| `ee_to_wp0wm` default / transverse / unpolarized (10 seeds) | 0.2578659 (+0.05) / 6.938526 (+0.76) / 7.196430 (+0.26) | 0.2578741 (+0.08) / 6.938531 (+0.76) / 7.196464 (+0.27) |
| `ud_to_epemud_qcd0` (5 seeds) | 1.413063e-2 ± 1.19e-5, pull +1.47, χ² 0.36 | 1.413128e-2 ± 1.04e-5, pull +1.55, χ² 0.64 |
| `zz_ee` (informational) | 4.731459e-4 ± 1.64e-7, pull +3.20 | 4.731611e-4 ± 1.64e-7, pull +3.24 |
| `zz_emu` | 9.447290e-4 ± 2.29e-7, +1.15 | 9.447292e-4 ± 2.29e-7, +1.15 |
| `uu_tt` on-shell veto (5 seeds) | 4.975220e-2 ± 3.1e-5, +4.69, χ² 0.20 | 4.977760e-2 ± 3.4e-5, +4.81, χ² 1.26 |
| `ee_to_wpwm` samples (validate_samples, 3 seeds × 20000) | min KS p 1.27e-4 (T-F1's after) | worst KS per seed `pt(w+)` 2.80e-2 / 1.06e-2 / 1.43e-4, min χ² p 0.29, gate passes |
| controls: `zz_emu_cut`, `pp_ttx_lep` (census: unchanged) | 6.661582e-4; 6.410089 | 6.661582e-4; 6.410089 (identical) |

- Every gated row passes, at the same pulls to within a few tenths.
- `uu_tt` keeps its known +2% offset from MadEvent; it is gated as
  "reported against".
- `zz_emu_cut` and `pp_ttx_lep` reproduce to every printed digit, which
  confirms the census's unchanged verdict on two rows.

**Not re-gated:**
- `uux_to_ccx_emmm_qcd0`, `bbx_to_ccx_emmm_qcd0`: `Plan::Long`, oracle tier.
- MLM `pp_to_ll_0j2j_mlm`, `pp_to_ttx_0j1j_mlm`: long tier, `#[ignore]`,
  `pixi run validate-mlm-sigma`.
- `uu_full`, `uu_tt_mg371`: references of the `uu_tt` comparison, no gate of
  their own.

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

## 4. The CMS DY card (TV2, shipped param card)

### Overweight at R1, before and after (same harness)

- **Harness.** `T-F2-gen.sh`: `generate --seed 7 --nevents 600` with the dump,
  on R1 grids (`--neval 10000 --niter 3`).
- **Before** is T-V2's R1 grids, seeds 1–4, under `T-F2-bin-before` (TV2
  `e28f084b`: T-V2's maps plus the dump).
- **After** is new R1 grids, seeds 1–4, under `T-F2-bin-after` (TV2
  `45d9fe28`, the fix).

Results lines are in `T-F2-results-gen.txt`.

| R1 seed | before: overweight share | before: largest w/w_max (channel) | before: trials | after: overweight share | after: largest w/w_max (channel) | after: trials |
|---|---|---|---|---|---|---|
| 1 | **0.751** | **1546.5** (2, γγ e) | 324124 | 0.085 | 6.3 (77) | 83597 |
| 2 | 0.328 | 207.7 (2, γγ e) | 1641029 | 0.125 | 11.0 (53) | 42879 |
| 3 | 0.205 | 25.4 (78) | 407246 | 0.124 | 11.1 (71, γγ τ) | 38258 |
| 4 | 0.151 | 12.9 (57) | 372961 | 0.155 | 18.1 (3, γγ e) | 48823 |

The other per-seed figures:

| R1 seed | integration σ before → after (pb) | sample − integration pull, before → after | predicted unweighting efficiency, before → after |
|---|---|---|---|
| 1 | 11.97 → 12.78 | +1.05 → +0.05 | 1.86e-3 → 7.48e-3 |
| 2 | 14.94 → 12.63 | +0.30 → +0.94 | 4.62e-4 → 1.43e-2 |
| 3 | 11.80 → 13.38 | +0.87 → +0.13 | 1.59e-3 → 1.66e-2 |
| 4 | 14.00 → 12.87 | −0.58 → +1.07 | 1.91e-3 → 1.24e-2 |

- **The largest ratio** falls from 13–1547 to 6–18. The γγ ladder channels
  (2, 3, 70, 71) still appear among the heaviest, at 11–18×, but no longer at
  hundreds.
- **The overweight share** falls on the seeds whose tail was the corner
  (0.75 → 0.09, 0.33 → 0.13). It stays at 0.12–0.16 elsewhere.
- **The unweighting efficiency** the frozen scan predicts rises 4–30×.
- **Caveat on the harness.** 600 events cost 4–40× fewer trials after the fix,
  so the after samples see fewer trials in which to meet a rare tail. The
  before samples' tails are single events: seed 1's 1547 is one trial in
  3.2·10⁵. Four seeds bound how often such an event appears only loosely.

### The paired R1 → R4 climb, 12 seeds

- **Rungs.** R1 is `--neval 10000 --niter 3`, R4 is `--niter 12` at the same
  `--neval`. The rungs nest, so a seed's R1 is its R4 log's iteration-3 line.
  This was checked on both sides: `after-r4-s1`'s iteration 3 reads 12.7805 pb,
  `after-r1-s1`'s σ is 12.780548. `T-F2-paired.py` reproduces T-V2's
  +0.538 ± 0.268 from T-V2's own R4 logs.
- **Runs.** Seeds 1–12, `T-F2-results-campaign.txt`, 1186–1307 s a run once
  the host was free (2886 s for seed 1, run beside the gates). Every run's
  rc is 0.

| | before (T-V2) R1 | before R4 | **after R1** | **after R4** |
|---|---|---|---|---|
| mean (pb) | 12.357 | 12.895 | 13.058 | 13.378 |
| median | 12.048 | 12.877 | 13.017 | 13.335 |
| seed sd | 1.201 | 0.636 | 0.560 | 0.224 |
| sd/√12 | 0.347 | 0.184 | 0.162 | 0.065 |
| seed χ²/dof | 3.55 | 3.45 | 1.62 | 0.52 |
| sd / mean quoted error | 1.32 | 1.17 | 1.00 | 0.77 |
| **paired R4 − R1** | | +0.538 ± 0.268 (sd 0.927), 9 of 12 rise | | **+0.320 ± 0.124** (sd 0.429), 9 of 12 rise |

Per seed after, R1 → R4: 1: 12.78 → 13.15, 2: 12.63 → 13.23,
3: 13.38 → 13.55, 4: 12.87 → 13.34, 5: 13.14 → 13.24, 6: 12.91 → 13.33,
7: 13.41 → 13.16, 8: 13.95 → 13.87, 9: 13.80 → 13.72, 10: 13.12 → 13.37,
11: 11.83 → 13.22, 12: 12.86 → 13.37.

**Reading it against the measured spread:**
- **The seed spread fell 2.1× at R1 and 2.8× at R4.** Seed χ²/dof fell from
  3.5 to 1.6 (R1) and 0.5 (R4). The quoted errors now cover the scatter.
  At R4 they slightly overstate it (sd/quoted 0.77). A χ²/dof of 0.5 over 12
  seeds is within its own sampling range, but it is the low side to watch.
- **The climb is smaller but has not stopped.** +0.32 ± 0.12 pb is 2.6σ on
  its own spread. Nine of twelve seeds still rise, and the one large riser
  (seed 11, +1.39) is an R1 low outlier being corrected. This is the same
  pattern AGENTS.md describes, at about 60% of the size and half the scatter.
  **The pass condition "the climb has stopped" is not met** at R4. A third rung
  (R16), or 20+ seeds on these two, would be needed to say more.
- **Against the corrected MadGraph reference** (survey per-P, the CERN host's
  note), quoted only as residuals and not tuned toward:
  - total R4 13.378 ± 0.065 against 13.12 ± 0.031, a ratio of
    1.020 ± 0.005;
  - e/μ part 9.149 ± 0.046 against 9.0052 ± 0.028, a ratio of 1.016;
  - τ part 4.229 ± 0.050 against 4.1146 ± 0.013, a ratio of 1.028.

  Before, at R4: e/μ 8.598 (0.955) and τ 4.297 (1.044). The opposite-signed
  split between the parts is gone; both now sit 1.6–2.8% high. That fits the
  CERN-host hypothesis that one under-mapped corner moved both parts, though
  it does not prove it. With the climb still running, neither number is a
  converged residual.

## 5. The τ part under the fix

From the same R4 runs (the `masses 1.777, 1.777, 0, 0` part of each log):
4.229 ± 0.050 pb after, against 4.297 ± 0.099 before. That is 1.028 of the
corrected MadGraph τ value (4.1146 ± 0.013), against 1.044 before. The e/μ
part moved up by 0.55 pb and the τ part down by 0.07. The fix (the heavy τ
point was the same γγ corner) touches both, but most of the τ excess over
MadGraph is not explained by it. No τ-only card was run (**not run**). The
VBF-4to50 τ reference is soft on MadGraph's side, per the note, and was not
used.

## Gates (on `/home/user/wt/T`)

The final gate ran on `cd8775e0`. Its code is `d54f0e46`; `cd8775e0` adds
only report text. `CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0
CARGO_PROFILE_TEST_DEBUG=0` throughout. `T-F2-gates-final.log`:

```
fmt rc=0           cargo fmt --all --check
clippy rc=0        cargo clippy -j 2 --workspace --all-targets -- -D warnings
clippy-ev rc=0     cargo clippy -j 2 --workspace --all-targets --features vibegraph/extended-validation -- -D warnings
clippy-ev-lib rc=0 cargo clippy -j 2 -p vibegraph-lib --all-targets --features extended-validation -- -D warnings
test rc=0          cargo test -j 2 --workspace --no-fail-fast -- --test-threads 2
                   48 suites, 1713 passed, 0 failed, 24 ignored
kblint rc=0        python3 scripts/kb.py lint → 1004 concepts, 333 backlog items; 0 errors, 4 warnings
```

The four kb warnings are pre-existing `state: blocked` backlog items.

**`spine_sigma`**, under its feature (`--profile release-debug --config
'profile.release-debug.debug=0' --features extended-validation`), on the tree
with both fixes (`T-F2-gate-spine-sigma.log`, `-M1.log`):

```
u c > e+ e- u c QCD=0 at √ŝ = 1000 GeV, 200000 × 8 a seed: 10 peripheral channels of 10
  16 seeds: mean 1.799563e-2 pb, spread 1.65e-5 (rel 9.15e-4), seed χ²/dof 1.27, 62 s
test result: ok. 1 passed; 0 failed; 1 ignored

M1 (spine withheld, the restored condition `&& (n_out == 2 || fiducial_scale > F::zero())`):
u c > e+ e- u c QCD=0 at √ŝ = 1000 GeV, 200000 × 8 a seed: 0 peripheral channels of 10
  16 seeds: mean 1.829004e-2 pb, spread 6.74e-4 (rel 3.68e-2), seed χ²/dof 5.74, 106 s
the seeds' relative spread 3.68e-2 exceeds 3.0e-3
test result: FAILED. 0 passed; 1 failed; 1 ignored
```

The source was restored from a backup and `cmp` reported it identical.

**Recalibration from 40 seeds per arm.** The default changed, so the spread
bound was re-derived as T-V2 did: `probe_the_seed_scatter_of_the_no_scale_spine`,
seeds 1001–1040, with 16-of-40 resampling by `T-V2-calib.py`.

| arm | 40-seed mean (pb) | per-seed rel. sd | sd / quoted | 16 of 40: rel. spread 0.1% / 50% / 99.9% | 16 of 40: χ²/dof 0.1% / 50% / 99.9% |
|---|---|---|---|---|---|
| spine (as committed) | 1.797511e-2 | 3.11e-3 | 1.05 | 3.9e-4 / 7.7e-4 / 1.08e-3 | 0.28 / 1.08 / 2.09 |
| spine withheld (M1) | 1.809914e-2 | 1.27e-1 | 1.46 | 7.5e-3 / 2.9e-2 / 4.4e-2 | 0.38 / 2.36 / 7.17 |
| T-V2's spine arm (10⁴, flat remainders) | 1.79781e-2 | 3.29e-3 | 0.91 | 4.5e-4 / 8.2e-4 / 1.11e-3 | 0.25 / 0.85 / 1.57 |

- The M1 arm is T-V2's to every printed digit, as it should be: with no
  spine, neither change applies.
- The spine arm's 99.9% spread quantile, 1.08e-3, is below T-V2's.
  `REL_SPREAD_MAX = 3.0e-3` keeps a factor 2.8 over it and 2.5 under M1's 0.1%
  quantile. `CHI2_PER_DOF_BAND = (0.15, 4.0)` brackets the spine arm's
  0.28–2.09. **Neither constant changes.**

## Corrections to the brief

- **"Whether a channel exists with both photon rungs as spacelike lines."**
  It exists and draws the heavy points: channel 2 is the photon → lepton →
  photon chain, holding 80% of `g` there. The brief's three candidate causes
  (no channel, the floor, the channel weights) miss the dominant one, the
  flat draw of the remainder invariants between rungs. The fix therefore
  includes a channel change the brief did not list (the remainder mixture),
  alongside a lower default floor, which it did list.
- **"Re-calibrate its spread bound from 40 seeds per arm."** Done. The bound
  did not need to move (table above).
- **The overweight harness.** "The same harness before and after" holds: one
  command, one binary family. But 600 events cost 4–40× fewer trials after the
  fix, so the before/after comparison is not at equal trial counts (see the
  caveat under the table).

## Incidents

- **Isolation leak into the main checkout, 03:49:55 UTC.**
  - *What happened:* a command of the form
    `cd /home/user/wt/T && cp … && sed -i … && ( cargo test … ) &` put the
    whole `&&` chain in the background, its `cd` included. The foreground
    `cp <backup> vibegraph-lib/tests/diagram_channel.rs` and `grep` that
    followed ran in the default cwd, `/home/user/vibegraph`.
  - *Effect on main:* the main checkout's `diagram_channel.rs` received T's
    file. It is byte-identical to my backup
    (`cmp T-F2-leaked-main-diagram_channel.rs T-F2-dc-tests.rs.bak`). The
    coordinator restored it, and `git -C /home/user/vibegraph status --short`
    is empty.
  - *Effect on T:* T was left with the temporary `n_flat = 1_000` in
    `probe_the_no_scale_spine_on_a_mass_cut_peripheral_process`, restored
    before any commit.
  - *Results affected:* none. No build or test ran in `/home/user/vibegraph`,
    since every cargo command ran with an explicit `cd /home/user/wt/T` in the
    foreground or under `setsid bash -c`. The one run using the temporary
    `n_flat` is `T-F2-sigma-probe2.log`, whose token flat arm is set aside
    above. From then on every write used an absolute path.
- **ENOSPC during the first workspace test build (about 04:24).**
  - *What happened:* `T/target/debug` with debuginfo reached 17 GB. I killed
    my gate processes by PID, deleted only `T/target/debug`, and re-ran every
    gate with debuginfo off.
  - *Effect on the campaign:* it ran through it. `after-r1-s1` had already
    written its grid. `after-r1-s2`'s log is complete (every iteration line
    present) and so is its grid.

## Found

- **The remainder mixture's share is fixed at ½.**
  - *Evidence:* on `e⁺e⁻ → e⁺e⁻µ⁺µ⁻` the mixture costs 1.4× per-draw variance
    against the flat draw (124 against 90 at 10⁴). The DY corner gains 8×
    from it. Neither process is served by a fixed even split.
  - *To close:* adapt the share per channel, as the α-survey adapts channel
    weights. The map then needs one more parameter in its key, or a VEGAS
    dimension that already exists (the remainder's own coordinate) could carry
    it. Measure on both processes.
- **10³ GeV² costs ee → eeμμ 2.1× against its best scale.**
  - *Evidence:* the variance table (84 at 10⁵, 174 at 10³). The leptonic
    process wants a high floor, because the default `|η_l| < 2.5` keeps its
    exchanges hard. The DY corner wants a low one.
  - *To close:* derive the floor from what the cuts allow per rung, for
    example an η cut implying a pT floor on a massless leg, instead of one
    constant.
- **The census probe misreads decay-chain fiducial scales.**
  - *Evidence:* `probe_the_banked_rows_whose_maps_moved` compiles `Cuts` over
    every leg and reads `zz_ee`/`zz_emu` at 100 GeV². Production exempts the
    decay products under `cut_decays = F` (floor 0, T-F1).
  - *To close:* take the fiducial scale from the integrand production builds
    (`FixedBeamIntegrand` / the decay-chain cut filter), not from
    `Cuts::compile` on the full leg list.
- **`wpwm_to_wpwmz_cw` cannot be censused.**
  - *Evidence:* its banked run card sets `nhel = 1`, which `RunCard::parse`
    refuses (`UnsupportedField`).
  - *To close:* parse the card with the field tolerated for map-only
    purposes, or record how that row's gate builds its cuts.
- **`with_rung_order` carries `rest_floor` with the line, not the position.**
  - *Evidence:* `with_rung_order` clones whole rungs, and `rest_floor` is
    installed per remainder `R_i` by `with_timelike_floors`, which is a
    property of the position. A reordering after the floors are installed
    leaves each remainder with another one's floor. `rest_mixed` had the same
    defect and is fixed here (`d54f0e46`).
  - *To close:* re-derive `rest_floor` by position in `with_rung_order`, or
    assert that floors are installed after any reordering. It is a test-only
    API today.
- **The rung-ordering test's coverage share is calibrated on flat remainders.**
  - *Evidence:* under the production mixture the derived two-rung chain's Z
    rung holds 0.88% in its thinnest bin, against the 1.2% bound (first run,
    `T-F2-dc-tests.log`). The test now measures flat remainders on both arms.
  - *To close:* recalibrate `T_ORD_MIN_BIN_SHARE` on the production map, or
    give the probe integrand the remainder structure a real matrix element
    has.
- **MadEvent's dominant DY channel G1 is not identified.**
  - *Evidence:* the CSV names G1 (48% of e/μ, 43% of τ, climbing 17× over
    its first iterations), but `configs.inc` is not in this checkout.
  - *To close:* map G1 and G78.22 to their diagrams from the gridpack's
    `configs.inc` and compare with channels 2/3/70/71 here.
- **`generate`'s scan runs on every core whatever else is running.**
  - *Evidence:* four generation jobs, started as "one thread each" with no
    `-j`, each ran its maxima scan multithreaded, at load average 22. Only
    the trial loop is serial. T-V2's "generate is serial" holds for the trial
    loop only.
  - *To close:* document `-j` for `generate`, or pass it through to the scan.
- **T-V2's report miscounts the rising seeds.**
  - *Evidence:* its R1 → R4 per-seed list has seed 5 at 12.13 → 12.09, a
    fall. So 9 of 12 rise, not "ten of twelve" with only 2 and 4 falling.
    `T-F2-paired.py` reproduces the paired +0.538 ± 0.268 from the R4 logs'
    iteration-3 lines.
  - *To close:* correct the sentence in the T-V2 report.
- **Process: a backgrounded `cd … && … &` leaked an edit into the main
  checkout.**
  - *Evidence:* the `&` backgrounded the whole `&&` chain, its `cd` included,
    so a following foreground `cp` ran in `/home/user/vibegraph`. The
    coordinator restored main's file.
  - *To close:* agent prompts should require absolute paths in every command
    that writes, never relying on a `cd` in a chain that may be backgrounded.
- **Process: a debug workspace test build with debuginfo filled the disk.**
  - *Evidence:* `T/target/debug` reached 17 GB, and the workspace test build
    died with ENOSPC (`T-F2-gate-test.log`, first attempt).
  - *To close:* the gate scripts in the agent prompts should set
    `CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0` by default.
