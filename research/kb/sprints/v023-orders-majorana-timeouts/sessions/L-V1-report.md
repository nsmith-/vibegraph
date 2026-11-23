---
type: Session Report
title: "L-V1 report: the VBS stable-boson σ excess is two MadGraph effects"
description: "The VBS cards' matrix elements agree with MadGraph per flow and per helicity. The excess over the gridpacks' MadGraph 2.9.18 is a sampling deficit of 2.9.18 and 3.5.7, registered as a deviation. The residual +1.4–1.9% over MadGraph 3.7.1 reproduces on one partonic channel and closes when MadEvent runs at hard_survey = 2 or its iterations are re-combined with equal weights: MadEvent's default-survey bias."
status: draft
tags: [vbs, sigma, polarization, madgraph-parity, cms-gridpacks, madevent-reference, survey, report, validation]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-22}
---
Worktree `/home/user/wt/sprintl-vbs`, branch `sprintl-vbs` off main `b2434a0`,
with `origin/main` (`a0a6eb4`, kb only) merged as `773f913` before the process
rule against further merges. No source code changes: the diff is the MadGraph
2.9.18 deviation entry, a row on the MadEvent default-survey page and this
report. The item file `vbs-stable-boson-sigma-reads-high` is deleted: R1
confirmed the attribution on the full card 700. On card 134 the same effects
account for most of the excess, and the remaining 0.6% is filed as new work
(see R1 and Found).

## Three MadGraph versions, and where each came from

| version | source | used for |
|---|---|---|
| 2.9.18 | the gridpacks (vg-validator's `gpdb` blobs); a local copy, upstream branch `2.9.18` (`d8a31fa`), shallow-cloned from github.com/mg5amcnlo/mg5amcnlo into the scratch area | the gridpack σ; a local σ on card 700 |
| 3.5.7 | the packaged `madgraph` pixi environment of the main checkout | local σ on cards 134 and 700, and on `p p > w+ z b b~` |
| 3.7.1 | vg-validator's runs on the exact gridpack cards (seeds 1–5, 10k events, MadSpin off); locally, a copy of the pinned submodule (`research/refs/mg5amcnlo`) | the reference σ; standalone amplitudes; reduced single-channel σ |

Local MadEvent work areas need one format fix to compile with this
container's gfortran: the `'(I',nconf,'$)'` edit descriptors in
`symmetry.f` (and `'(a$)'` in 2.9.18's `reweight.f`) take the comma that
3.7.1's template already has (`',$)'`). It changes nothing but the format of a
progress line. 3.7.1 needed no change.

No full-card 3.7.1 σ ran here. One queued local 3.7.1 run of card 700 was
killed by PID while it waited for the lock, before it started, when the
planner reserved those regenerations for vg-validator.

## The cards

vg-validator's essentials (`/home/user/wt/cards-sprintL/vbs.md`): `import model
sm`, `p = g u c d s u~ c~ d~ s~`, `j = p + b b~`; 134 `w-{T} z{0} j j`, 135
`z{0} w-{0} j j`, 698 `w-{T} w-{0} j j`, 700 `w+ z j j`, 702 `z{T} w-{T} j j`,
all `QED=4 QCD=0`; `ebeam` 6800, `lhaid` 325500
(NNPDF31_nnlo_as_0118_nf_4_mc_hessian, member 0 fetched from
lhapdfsets.web.cern.ch, archive SHA-256 `0a2363b5…980af`),
`dynamical_scale_choice` −1, `sde_strategy` 2, `nhel` 0, `bwcutoff` 15,
`ptj` 10, `etaj` 6.5, `drjj` 0.4, `mmjj` 10, `maxjetflavor` 4, `etab` 6.5,
`drbj` 0.4; SM default parameter card.

The local cards were rebuilt from MadGraph 3.5.7's default run card with those
edits, plus `nevents` (2000 or 5000) and `use_syst` F; the parameter card is
3.5.7's SM default, the same values the gridpacks carry. Every vibegraph run
here uses the same three files.

**Brief correction.** The cards are not in `cms-sw/genproductions` master: its
polarised VBS cards (`13TeV/VBS/VBS_SSWW_*`, `13p6TeV/VBS/VVjj_semilep_Polarized`)
all have `mmjj = 100`, `sm-ckm_no_b_mass` and decays in the proc card.
vg-validator's cards replaced them before any σ was compared.

## Oracle 1: per flow and per helicity amplitudes against MadGraph standalone

One partonic channel of each card, MadGraph 3.7.1 standalone (`output
standalone`, banked with `validation/madgraph/gen_standalone_jamps.py`'s
`bank()` from a scratch driver), 3 RAMBO points at √ŝ = 1.5 TeV, `--amps`.
The comparison is `tests/standalone_jamps.rs`'s `measure` and `check` on a
scratch copy of that file (`lv1_probe.rs`, not committed), which fits one
complex constant `G` over every (point, helicity, flow) entry.

```
python lv1_standalone.py all --amps        # MG 3.7.1 standalone, 10 tables
cargo test -p vibegraph-lib --test lv1_probe -- --nocapture --test-threads=1
```

Every width zero on both sides (the `_zw` rows):

| channel (card) | entries | `G` | worst flow | worst |M|² |
|---|---|---|---|---|
| `u u > w+ z u d` (700) | 864 | `2.3e-15 + 1.0000000000000069i` | 1.07e-12 | 2.4e-14 |
| `d d > w-{T} z{0} d u` (134) | 192 | `2.4e-14 + 0.9999999999992542i` | 6.1e-13 | 5.3e-13 |
| `d d > z{0} w-{0} d u` (135) | 96 | `1.2e-13 + 0.9999999999924402i` | 2.4e-11 | 3.3e-11 |
| `d d > w-{T} w-{0} u u` (698) | 192 | `9.2e-13 + 0.9999999999999073i` | 3.6e-12 | 1.5e-12 |
| `d d > z{T} w-{T} d u` (702) | 384 | `3.4e-14 + 1.000000000000037i` | 3.1e-13 | 1.2e-13 |

Four rows exceed `standalone_jamps.rs`'s committed 1e-12 by up to 24×: the
gate's tolerance is set for processes with fewer, smaller cancellations, and
2e-11 on a 2 → 4 electroweak sum with longitudinal bosons is rounding. The
helicity tables are row for row the same, so `{T}` is `[1, −1]` and `{0}` is
`[0]` in the same frame on both sides.

With MadGraph's widths kept (standalone keeps t-channel widths; this crate
zeroes them as MadEvent does) the same rows differ by 7.8e-5 to 3.0e-3 in |M|²:
the size of the t-channel width, which is how big a t-channel-width mistake
could be. MadEvent zeroes them and so does this crate.

Blind to: kinematic regions the 3 generic points do not visit (forward jets at
small `t`, the `mjj` window of an s-channel W/Z); the other partonic channels;
whether MadEvent's own t-channel classification matches this crate's line by
line (bounded above by 3.0e-3 at these points).

## Oracle 2: the MadGraph versions on the cards

| card | MadGraph | events | σ (pb) | equal-weight σ (pb) |
|---|---|---|---|---|
| 700 | 3.7.1, vg-validator, 5 seeds | 10k | 0.45728 ± 0.00043 (spread) | — |
| 700 | 2.9.18 gridpack | — | 0.4418 ± 0.00098 | — |
| 700 | 2.9.18 local, 1 seed | 2000 | 0.4288 ± 0.0023 | 0.4407 ± 0.0037 |
| 700 | 3.5.7 local, 1 seed | 2000 | 0.4277 ± 0.0021 | 0.4383 ± 0.0040 |
| 134 | 3.7.1, vg-validator, 5 seeds | 10k | 0.054374 ± 0.000099 (spread) | — |
| 134 | 2.9.18 gridpack | — | 0.05078 ± 0.000191 | — |
| 134 | 3.5.7 local, 1 seed | 2000 | 0.04784 ± 0.00028 | 0.04864 ± 0.00034 |

vibegraph on card 700, this branch, seed 1, `--target-rel 0.003`: 0.4629 ±
0.0013 pb, χ²/dof 3.96 over its iterations (v0.2.2's five seeds: 0.46382 ±
0.00059).

Local 3.5.7 and 2.9.18 agree with each other, sit below 3.7.1 by 6–12%, and
sit below the gridpack's own 2.9.18 at its larger budget. The 2.9.18 → 3.x
step is therefore not one step: 3.5.7 behaves like 2.9.18, 3.7.1 does not.
The equal-weight column is each channel's refine iterations averaged with equal
weights (`validation/madgraph/ew_recombine.py`, the method of
[the iteration-weighting bias](../../../validation/madevent-iteration-weighting-bias.md)):
it lifts 2.9.18 and 3.5.7 by 1.7–2.8%, more than on any 3.7.1 run here.

What is common to 2.9.18 and 3.5.7 and changed by 3.7.1 is the
`sde_strategy = 2` channel weight: `get_channel_cut` with `(t-M)(t+M)` where
`t` is already `p²` (diffed: 2.9.18 and 3.5.7 identical; 3.7.1 uses
`t - M**2` and a plain Breit–Wigner). That is the 3.5.x defect already
registered, which handed a Higgs pole to the wrong channels. It is the
candidate, not a proven cause: no 2.9.18 was patched here.

**The `μF` route is ruled out.** The same weight picks the clustering
configuration, so it could move `μF`. vibegraph with 2.9.18's weight (a scratch
environment switch in `ChannelSet::channel_cuts`, reverted;
`scripts/oldcut_switch.patch`):

```
VG_LV1_OLD_CHANNEL_CUT=1 vibegraph integrate proc_700.dat --run-card run_card_unpol.dat \
    --param-card param_card.dat --seed 1 --target-rel 0.004
σ = 462.4 ± 1.7 fb          (same seed, 3.7.1's weight: 462.9 ± 1.3 fb)
```

The switch is reached: the same seed gives a different σ. QCD=0 puts no `αs`
in |M|², so the scale enters only through the PDFs, and the configuration draw
moves σ by −0.1 ± 0.5%.

**The `b b~` part shows the sampling.** `p p > w+ z b b~ QED=4 QCD=0`, the
4% of card 700 whose b's are not jets (`maxjetflavor` 4): 3.5.7 gave 0.01284 pb
inside the full card and 0.01670 ± 0.000095 pb run alone at 5000 events;
vibegraph, 3 seeds: 18.230 ± 0.048, 18.318 ± 0.046, 18.300 ± 0.043 fb, mean
18.283 fb. A 30% swing with the budget allocation is a sampler that misses part
of the integrand. The 2.9.18 and 3.7.1 runs of this subprocess were cut short
by a scheduling time limit and not repeated, since the 3.7.1 seeds made them
unnecessary.

Registered in [madgraph-defects](../../../validation/madgraph-defects.md) as a
2.9.18 deviation, with the MadGraph errors from the 5-seed spread and the
1.6× under-quoted error on card 134 stated.

## Oracle 3: the residual over 3.7.1 on one partonic channel

The residual (vg/3.7.1 = 1.0143 ± 0.0016 on 700, 1.0186 ± 0.0023 on 134)
reproduces on the single channel `u u > w+ z u d QED=4 QCD=0` (card 700's
settings, `p p` beams, 52 MadEvent channels, about a minute a run). MadGraph
3.7.1 local, 5000 events, each seed in a fresh copy of a never-run directory:

| estimate | seeds | values (fb) | mean ± sem (fb) | vg/MG | pull |
|---|---|---|---|---|---|
| vibegraph, `--target-rel 0.003` | 1–5 | 142.82, 143.47, 142.13, 143.02, 142.93 | 142.874 ± 0.216 | — | — |
| vibegraph, `--target-rel 0.0015` | 11, 12 | 142.96, 143.07 | 143.02 | — | — |
| MadEvent 3.7.1 default, reported | 1–5 | 141.1, 141.3, 141.5, 140.8, 140.3 | 141.000 ± 0.210 | 1.0133 ± 0.0022 | +6.2 |
| same runs, equal-weight iterations | 1–5 | 141.20, 143.74, 143.76, 143.08, 142.25 | 142.806 ± 0.487 | 1.0005 ± 0.0037 | +0.1 |
| MadEvent 3.7.1 `hard_survey = 2`, reported | 1–5 | 142.86, 142.8, 142.3, 142.7, 142.6 | 142.652 ± 0.099 | 1.0016 ± 0.0017 | +0.9 |
| same runs, equal-weight iterations | 1–5 | 144.23, 142.93, 142.38, 144.73, 143.36 | 143.527 ± 0.426 | 0.9955 ± 0.0033 | −1.4 |

The errors are the seed spreads. `Source/run_config.inc` confirms the survey
mode: `min_events` 1000 at the default and 10000 at `hard_survey = 2`, so it is
not the inert case (`mmjj` 10 < 1% of 13600). vibegraph's χ²/dof over its five
seeds is 1.67; MadEvent's default is 0.73, so its seeds agree with each other
while reading 1.3% low, as the [default-survey bias](../../../validation/madevent-default-survey-bias.md)
page describes.

Single-seed controls:
- `u d~ > w+ z d d~` (s-channel W/Z → jj resonances): vibegraph 12.279 ±
  0.034 and 12.249 ± 0.032 fb; MadEvent default 12.02 ± 0.043 (equal-weight
  12.08), `hard_survey = 2` 12.19 ± 0.037 (equal-weight 12.27). The same
  pattern, +2.0% closing to +0.6% and 0.0%.
- Fixed scales (`fixed_ren_scale = fixed_fac_scale = T`, 91.188) on
  `u u > w+ z u d`: vibegraph 146.38 ± 0.41 and 145.66 ± 0.40 fb, MadEvent
  default 143.93 ± 0.61 (equal-weight 144.97). The residual (+1.5%) does not
  depend on the scale prescription.
- Event shapes, vibegraph 20 000 events against MadEvent default 5000, as
  differential σ in `mjj`, the softer jet's `pT` and `SCALUP`
  (`scripts/lhe_compare.py`): no bin beyond 2.2σ, `SCALUP` quantiles alike.
  At 5000 events this cannot resolve a 1.4% integral; it only excludes a
  localised gross difference.

The polarised channel `d d > w-{T} z{0} d u` (card 134's settings, its `me_frame`
block included) ran the same way, three seeds a setting:

| estimate, `d d > w-{T} z{0} d u` | seeds | values (fb) | mean ± sem (fb) | vg/MG | pull |
|---|---|---|---|---|---|
| vibegraph, `--target-rel 0.003` | 1–3 | 6.537, 6.514, 6.530 | 6.527 ± 0.007 | — | — |
| MadEvent 3.7.1 default, reported | 1–3 | 6.448, 6.430, 6.406 | 6.428 ± 0.012 | 1.0154 ± 0.0022 | +7.1 |
| same runs, equal-weight iterations | 1–3 | 6.554, 6.426, 6.479 | 6.486 ± 0.037 | 1.0063 ± 0.0059 | +1.1 |
| MadEvent 3.7.1 `hard_survey = 2`, reported | 1–3 | 6.526, 6.519, 6.495 | 6.513 ± 0.009 | 1.0021 ± 0.0018 | +1.2 |
| same runs, equal-weight iterations | 1–3 | 6.553, 6.585, 6.470 | 6.536 ± 0.034 | 0.9986 ± 0.0053 | −0.3 |

The polarised channel behaves as the unpolarised one: +1.5% at the default
survey, +0.2% at `hard_survey = 2`. The residual is not polarisation-specific.

**Attribution.** The +1.4–1.9% residual over vg-validator's 3.7.1 is MadEvent
reading low at its default survey settings, which seeds 1–5 at that setting
share. It is not a vibegraph excess: on the reproducing channel vibegraph agrees
with MadEvent run at `hard_survey = 2` to 0.16 ± 0.17% (polarised channel:
0.21 ± 0.18%), and with MadEvent's own default runs re-combined with equal
weights to 0.05 ± 0.37% (0.63 ± 0.59%). The size predicted
for the full cards is the 1.3–1.9% measured. The full-card test is R1 below:
it confirms the attribution on card 700 and leaves 0.6% open on card 134.

## R1: the full cards at `hard_survey = 2`

vg-validator ran request R1 on 2026-11-23. Setup:
- MadGraph 3.7.1, the gridpack madevent cards plus `hard_survey = 2`;
- `nevents` 10 000, `iseed` 1–5, a fresh directory per seed, MadSpin off;
- `min_events` 10000 on every seed (from `Source/run_config.inc`).

The equal-weight (EW) column comes from
`validation/madgraph/ew_recombine.py`, and the host's own tool agreed to every
printed digit. The vibegraph values are the v0.2.2 five-seed means. No change
merged since then can move them: the cards use the SM model with plain
`QED`/`QCD` bounds, no squared orders, no Auto widths and no reweighting.

| card | MG reported, 5 seeds (sd) | MG χ²/dof | MG EW (sd) | vibegraph | vg / reported | vg / EW |
|---|---|---|---|---|---|---|
| 700 `w+ z j j` | 0.46248 (0.00138) | 1.61 | 0.46395 (0.00182) | 0.46382 | 1.0028 ± 0.0013 | **0.9997 ± 0.0017** |
| 134 `w-{T} z{0} j j` | 0.054916 (0.000069) | 0.33 | 0.055038 (0.000166) | 0.055385 | 1.0086 ± 0.0006 | **1.0064 ± ~0.0025** |

The ratio errors use the MadGraph standard error of the mean. On 134, the
±0.0025 also includes vibegraph's own five-seed spread.

- **Card 700 confirms the attribution.** The default survey's −1.4% is
  removed by `hard_survey = 2`, and the remaining +0.3% by the equal-weight
  lift. Against the equal-weight σ, vibegraph agrees to 0.03%.
- **Card 134 misses the prediction** (0.05539 within about 0.4%).
  - The survey effect is confirmed: `hard_survey = 2` lifts MadGraph by 1.0%
    over the default survey, and equal weighting by 0.2% more.
  - vibegraph still sits **0.64% ± ~0.25%** above MadGraph's equal-weight σ.
  - MadGraph's χ²/dof of 0.33 on this card is itself a warning: its quoted
    errors look over-quoted, or five seeds understate its spread.
  - **The residual is not localised.** MadGraph groups 134 into two
    subprocess directories: the light-quark group `P1_qq_wmTz0qq` (96.9% of σ)
    and the b b̄ group `P1_qq_wmTz0bbx` (3.1%). For the b b̄ group alone to
    carry +0.6% of the total, it would need +20%. So the residual most likely
    sits in the light-quark group, or is spread through it. vibegraph's 134
    artifacts are no longer on the host, so vibegraph's side of the split is
    not measured.

So the VBS excess is attributed on the full card 700. On 134 the same two
effects account for 1.2 of its 1.9%, and the remaining 0.6% is open: see
Found.

## What each test cannot see

- **Amplitudes**: as above; also a global phase per process.
- **The configuration-draw switch**: only `get_channel_cut`'s role in the
  draw; the sampling role of that weight inside MadEvent is not reproduced.
- **Reduced channels**: one or two of the 42 subprocess groups; a defect in
  another group (a b-quark cut, an `s c̄` channel) would be invisible. The full
  cards' answer comes from R1.
- **Equal-weight recombination**: unbiased given each frozen grid, but noisy;
  it cannot repair a survey grid that never visits a region, only weighting.
- **σ agreement**: blind to compensating regions; the event-shape check above
  is too small to close that.

## Requests for the long-run host

- **R1.** MadGraph 3.7.1 on cards 134 and 700, the exact madevent cards with
  `hard_survey = 2` added, `nevents` 10 000, `iseed` 1–5, fresh directories,
  MadSpin off. Expected: 700 at 0.4638 pb within about 0.3%, 134 at 0.05539 pb
  within about 0.4% (vibegraph's v0.2.2 five-seed means), against 0.45728 and
  0.054374 at the default. Record `Source/run_config.inc`'s `min_events`
  (10000 means the setting took).
- **R2.** No new run: the equal-weight recombination of the existing default
  seeds 1–5 of 134 and 700 (`validation-report`'s `seed-record` or
  `ew_recombine.py` over `SubProcesses/P*/G*/results.dat`). Expected: each seed
  lifted by about 1–2%, onto vibegraph.
- **R3.** If R1 confirms, the same `hard_survey = 2` repeat on card 967 (ALP)
  and g2HDM M1000, whose excesses over 3.7.1 (+1.2%, +2.8%) are where
  2.9.18 and 3.7.1 agree, as two default-survey runs would.

## Gates

No source change is committed, so no cargo gate is affected; `cargo fmt` and
`kb-lint` ran on the kb diff (below). The scratch probe compiled and ran under
the lock (`cargo test -p vibegraph-lib --test lv1_probe`), and the release
binary every σ here used is this branch's head plus the reverted switch, inert
without `VG_LV1_OLD_CHANNEL_CUT`.

```
pixi run kb-lint
1350 concepts, 470 backlog items; 0 errors, 5 warnings   (the 5 are on backlog items this PR does not touch)
```

## Found

- **The amplitude gate's tolerance on electroweak 2 → 4 with longitudinal
  bosons.** Four of the five VBS channels agree with MadGraph standalone at
  3.6e-12 to 2.4e-11 of the largest amplitude, above `standalone_jamps.rs`'s
  1e-12. Banking a VBS row there needs a tolerance justified by measured
  rounding. Closing: a committed VBS table and a measured bound.
- **Reference σ banked at MadEvent's default survey.** Every MadEvent σ
  compared in the CMS campaign (2.9.18 gridpacks, and vg-validator's 3.7.1
  regenerations) ran the default survey. On electroweak VBS it reads 1.3–1.9%
  low; the default-survey page records −0.6% to −2.0% on other rows. Closing:
  campaign regenerations at `hard_survey = 2` with the equal-weight recombination
  recorded beside the reported σ.
- **g2HDM M1000 (+2.8%) and the ALP card 967 (+1.2%).** Both match the
  generic cause found here: both MadGraph versions ran the default survey,
  which is shared, so their agreement does not exclude it. R3 tests it. Widths,
  scale, PDF and cuts were not needed to explain the VBS residual.
- **vibegraph's iteration χ²/dof 3.96 on card 700, seed 1.** Its σ agrees with
  v0.2.2's five seeds and with the 4× rung on the reduced channel, but its
  iterations scatter beyond their errors. Closing: the per-channel iteration
  scatter on this card at the campaign budget.
- **Card 134 (polarised `w-{T} z{0} j j`) leaves +0.64% ± ~0.25% over
  MadGraph at `hard_survey = 2` with equal weighting.** Card 700 closes under
  the same treatment. The plan, in order:
  1. a per-subprocess-group comparison: vibegraph's σ summed into
     MadGraph's two groups (the b b̄ final states against the light quarks),
     from a vibegraph run of 134 on current main, against MadGraph's per-group
     σ (the host's `R1_perP_hs2.txt`);
  2. then a per-flow and per-helicity standalone table on the subprocesses
     that carry it, which so far is checked on one channel only;
  3. only then a budget question. A comparison between two budget rungs needs
     the measured seed spread, from 20 or more seeds on both rungs.
  MadGraph's χ²/dof of 0.33 here argues for more MadGraph seeds as well, if
  the residual turns out to be spread evenly.
  Closes when the residual is attributed, or registered with its size.
- **gfortran rejects `'$)'` edit descriptors** in MadGraph 3.5.7 and 2.9.18
  `symmetry.f` (MadEvent fails with `could not convert string to float: '#7'`).
  Local MadEvent comparisons on those versions need the comma fix above.
