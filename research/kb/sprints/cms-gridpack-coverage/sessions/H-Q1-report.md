---
type: Session Report
title: "H-Q1 report: the light-quark QCD offset is MadEvent's sampling, not a vibegraph factor"
description: "u d > d g u agrees with MadGraph per helicity and flow to 9e-16, and per point (PDF, |M|², cuts, on both sides' events) to 4e-8. Of the +1.56 % single-subprocess offset, 1.2 % is MadEvent's default survey (H-G1's references ran it); at hard_survey = 2 MadEvent still reads 0.37 % low, flat from 1× to 10× events. An oracle that folds fixed-energy σ̂(ŝ) with an LHAPDF luminosity quadrature puts vibegraph at −0.09 ± 0.04 % and MadEvent at −0.44 %, and the missing part is at √ŝ > 1.5 TeV, where MadEvent's own fixed-energy σ̂ is right. The 2 → 4 HT card behaves the same way and climbs with MadEvent's budget. Uncut Drell–Yan against a deterministic quadrature: vibegraph +3e-5, so the ~0.1 % term H-M2 sees is not in vibegraph's PDF, |M|², flux or (τ, y) map. No vibegraph change."
status: draft
tags: [sigma, qcd, madgraph-parity, proton, madevent-reference, survey, sprint-h, report, validation]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-05}
sources:
  - {id: survey, resource: "../../../validation/madevent-default-survey-bias.md", title: "MadEvent's default survey settings bias σ low"}
  - {id: iter, resource: "../../../validation/madevent-iteration-weighting-bias.md", title: "MadEvent's iteration combination reads low"}
---
Session H-Q1 of sprint H, item `light-quark-qcd-sigma-high-vs-madgraph`.
Worktree `/home/user/wt-qcd`, branch `sprinth-qcd-sigma` at `cbfaec61`.
I checked both before any action: `git rev-parse --show-toplevel` →
`/home/user/wt-qcd`, `git branch --show-current` → `sprinth-qcd-sigma`.

**Outcome.** No vibegraph factor differs from MadGraph. The offset is MadEvent's
integration, in two parts:

- **Most of it** is the default survey. H-G1's references were run with it,
  and it is the catalogued defect
  [madevent-default-survey-bias](../../../validation/madevent-default-survey-bias.md).
- **What is left** at `hard_survey = 2`: MadEvent under-samples large √ŝ in the
  (x₁, x₂) convolution. Its fixed-energy σ̂ there is right. The deficit does not
  shrink with its event budget on the 2 → 3 card, and shrinks only slowly on
  the 2 → 4 HT card.

The second part is a candidate for the defect catalogue. It is not filed here.

vibegraph matches an oracle that shares neither program's hadronic sampling,
to 0.09 % ± 0.04 %. Nothing on this side is changed, so no code is committed:
this report is the only file.

Two things bound the result:

- **The 2 → 4 HT card** is diagnosed by its budget ladder and its
  equal-weight recombination only. The fold oracle was built for the 2 → 3
  card. The HT card's `ihtmin` is boost invariant, but I did not build its
  σ̂(ŝ) table.
- **The fold oracle** uses vibegraph's own fixed-energy σ̂. MadGraph's
  fixed-energy σ̂ agrees with it at four energies, to 0.1–0.3 % (one of
  MadGraph's errors), so the oracle is no better than that as a test of σ̂
  itself. As a test of the hadronic sampling, which is the question, it shares
  nothing with either program.

## Setup

- **MadGraph:** 3.7.1 at `b7687064`
  (`python3 /home/user/vibegraph/research/refs/mg5amcnlo/bin/mg5_aMC`), LHAPDF
  6.5.4, `pdlabel = lhapdf`, `lhaid = 247000`, nb_core 2.
- **Templates:** one never-run `output` per process, under
  `/home/user/H-Q1-scratch/mg/H-Q1-tmpl-{ud,uu,dy}`.
- **Fresh directories:** every MadEvent seed ran in its own `cp -r` of the
  template, through `/home/user/H-Q1-scratch/mg/H-Q1-mg-seeds.sh`. The script
  installs `iseed` and `hard_survey` into the card and records the effective
  `min_events` / `max_events` from `Source/run_config.inc`. These read
  10000/30000 for every `hard_survey = 2` seed and 1000/2000 for every default
  seed.
- **vibegraph:** the release-debug binary of `cbfaec61`, copied to
  `/home/user/H-Q1-scratch/vg/H-Q1-vibegraph`, run through H-G1's seed script
  with `LHAPDF_DATA_PATH` pointed at this worktree's `validation/pdf`.
- **PDF files:** the PDF set under `/home/user/lhapdf/share/LHAPDF` is
  byte-identical to `validation/pdf/NNPDF23_lo_as_0130_qed`: `cmp` reports
  every one of its 103 files the same.
- **Cards:** H-G1's `H-G1-conj_run_card_vg.dat` (13 TeV, `ptj = 20`,
  `etaj = 5`, `drjj = 0.4`, fixed μ = 91.188, `sde_strategy = 1`) and
  `H-G1-uugguu_run_card_5k_vg.dat` (the same with `ihtmin = 300`, 5000
  events). The variants are named where they are used.
- **Seed statistics:** `/home/user/H-Q1-scratch/tools/H-Q1-seedstats.py`
  reports the unweighted mean, the seed sd, the error max(quoted/√n, sd/√n)
  and χ²/dof about the mean.
- **Equal-weight recombination:** `/home/user/H-Q1-scratch/tools/H-Q1-iterrecomb.py`,
  which is H-G1's script, copied.
- **Scratch probes:** `examples/hq1_{probe,dsig,pdf,maps}.rs`. They were never
  committed and are deleted.
- **MadEvent drivers:** `/home/user/H-Q1-scratch/drv/hq1drv.f` and
  `hq1quad_main.f`, each linked into a copy of a MadEvent P-directory.

## 1. |M|² per helicity and per flow, bit-exact

**Inputs.** H-G1's standalone table `/home/user/H-G1-scratch/ud_to_dgu.json`:
4 points × 32 helicities × 4 flows, carrying its own param card. vibegraph
binds that same card.

**Command.** `target/debug/examples/hq1_probe /home/user/H-G1-scratch/ud_to_dgu.json`.
It calls `eval_m2`, and `run_flows` per helicity, then fits one complex
constant per point.

```
point 0: vg 7.05075224927616956e-2 mg_sum_hel 7.05075224927618066e-2 ratio 0.99999999999999845
  G = (2.2e-16, -0.9999999999999991)  worst per-hel per-flow |vg - G mg|/max|mg| = 4.357e-16
point 1: ... ratio 0.99999999999999889  G = (-2.5e-16, -0.9999999999999993)  worst 5.894e-16
point 2: ... ratio 0.99999999999999856  G = (1.1e-16, -0.9999999999999996)  worst 5.211e-16
point 3: ... ratio 0.99999999999999911  G = (-5.7e-17, -1.0000000000000004)  worst 9.265e-16
```

**Result.** G = −i at every point, which is the amplitude gate's convention,
and every (helicity, flow) entry agrees to ≤ 9.3e-16. |M|² is cleared.

**Blind spot.** This test cannot see a phase common to all flows. It does not
matter here: |M|² is all the cross section reads.

## 2. dσ factors at fixed points, against MadEvent's own routines

### The driver

`hq1drv.f` is linked against the madevent objects of a fresh P-directory, with
its `driver.o` replaced. It calls MadEvent's own `SETRUN`, `SETPARA` and
`SETCUTS`, then per point:

- sets `XBK` and `CM_RAP`;
- calls `PASSCUTS`;
- calls `DSIG1(…, IMODE = 4)`, which returns `PD(0)`, the product of
  `PDG2PDF` values;
- calls `SMATRIX1` with `MULTI_CHANNEL = .FALSE.`, which returns
  Σ_hel |M|² / `IDEN`;
- prints `G²/4π`.

MadEvent's own cut code needs partonic-CM momenta. Lab-frame input fails 762
of 2000 of MadEvent's own events. The CM-boosted input passes all of them, and
the reports below use it.

`hq1_dsig.rs` evaluates the same points on this side:

- `FlavorGroup::luminosity / (x₁x₂)`;
- `eval_m2` of the production group evaluator;
- `cuts().pass` on the lab-boosted momenta.

It binds MadEvent's param card at MadEvent's α_s. The driver prints that value
as `0.13000271085472237`, and LHAPDF's `alphasQ(91.188)` is
`1.30002710854722342e-01`.

### Results

| points | PASSCUTS (MG / vg) | PDF product, worst rel | |M|²/36 against SMATRIX1, mean / worst rel |
|---|---|---|---|
| 2000 MadEvent events, `u d > d g u`, seed 501 | 2000 / 2000 pass | 4.4e-16 | 3.5e-12 / 2.1e-8 |
| 20000 vibegraph events, seed 601 | **20000 / 20000 pass MadEvent's cuts** | — | −5.2e-12 / 4.1e-8 (product with the PDFs) |
| 2000 MadEvent HT events, `u u > g g u u`, seed 521 | 2000 / 2000 | 4.4e-16 | −6.9e-11 / 2.1e-6, with `IDEN = 144`, which is 36 × 4 |
| 10000 vibegraph HT events, seed 611 | **10000 / 10000 pass MadEvent's cuts, `ihtmin = 300` included** | — | — |

The worst relative entries come from the printed momenta in the LHE file,
which carry 11 significant digits; the means are at rounding level.

**Non-vacuity of the cut check.** I halved every outgoing pT of 500 vibegraph
HT points. MadEvent's `PASSCUTS` then fails 473 of the 500.

**Mirror.** vibegraph's group has `has_mirror false`, and its mirrored
luminosity is 0 on every point. MadEvent's `mirrorprocs.inc` is `.FALSE.` too.

**Symmetry factor.** On the HT card the factor ¼ is applied on both sides:
vibegraph's luminosity × |M|²/(36·4) equals MadEvent's `PD × ANS`.

### The PDF reader

Asked by the manager. `/home/user/H-Q1-scratch/pdf/lha.cpp` calls LHAPDF
`xfxQ`, and `hq1_pdf.rs` calls `PdfMember::xfx_q2`. Both run on 61 points
log-spaced in x over [1e-6, 0.95], at Q = 10, 91.188 and 1000 GeV, for
d, u, d̄, ū, g, b, s and c.

**Result.** The worst relative difference over the 183 points and 8 flavours
is 2.9e-16 (d at x = 3.98e-4, Q = 1000). vibegraph's PDF reader is exact.

### The flux constant

vibegraph's `GEV2_TO_PB` is `3.893793721e8`. MadEvent's `CONV` is
`389379.66*1000`, a single-precision literal, so MadEvent is higher by 7.4e-7.
That is negligible.

**What step 2 cannot see.** It cannot see the hadronic Jacobian and the
phase-space measure: neither enters a per-point value. Sections 3 and 4 test
those.

## 3. The universal ~0.1 % term: uncut Drell–Yan against a quadrature

H-M2 reported vibegraph +7e-4 to +1.1e-3 above MadGraph on 7 rows. I tested
that term where an exact answer exists.

### Fixed points

The card is `u u~ > e+ e-` with MadEvent's lepton cuts as generated. Over 200
random points (mₗₗ from 55 GeV to 2 TeV, random y and cos θ):

- PDF product: worst 4.4e-16;
- |M|²: worst 2.5e-14, both with MadEvent's param card and with vibegraph's
  default model;
- cut indicator: identical point by point (109 of 200 pass on both sides).

### σ against a deterministic oracle

**The oracle.** `hq1quad_main.f` is a Gauss–Legendre quadrature in
(ln mₗₗ over 11 segments with knots at the Z peak, y, cos θ). It calls
MadEvent's own `PDG2PDF` (through `DSIG1` with `IMODE = 4`) and `SMATRIX1`.
There is no adaptation and no sampling.

**The card.** `mmll ≥ 50` and nothing else: `ptl = 0`, `etal = -1`,
`drll = 0`. The file is `/home/user/H-Q1-scratch/drv/dy_run_card_nocut.dat`.

The quadrature converges:

```
QUAD   32   32    4   0.343043959889712E+03 pb
QUAD   64   64    4   0.343040816700987E+03 pb
QUAD   96   96    8   0.343039433683291E+03 pb
```

| estimate | σ (pb) | relative to the quadrature |
|---|---|---|
| quadrature (96 × 96 × 8) | 343.0394 | — |
| vibegraph, 5 seeds (811–815), `--target-rel 0.0002` | 343.049 ± 0.030 (sd 0.054, χ²/dof 0.61) | **+2.8e-5 ± 8.9e-5** |
| MadEvent, `hard_survey = 2`, 5 fresh seeds (801–805) | 343.238 ± 0.121 (χ²/dof 0.72) | +5.8e-4 ± 3.5e-4 |
| MadEvent, equal-weight | 343.122 ± 0.198 | +2.4e-4 |

**Result.** vibegraph's whole hadronic chain is exact to 1e-4 on Drell–Yan:
PDF, |M|², flux, (τ, y) map and mirror handling. So H-M2's ~0.1 % term is not
in any factor this row exercises.

**What this row cannot see.** It has no cuts, so a cut-region effect on H-M2's
rows is not tested. Those rows have lepton cuts, which this card does not.
On this card MadEvent sits on the high side, by 1.6σ.

## 4. The 2 → 3 card: what each estimator reads

`u d > d g u` alone: the single-subprocess control, the H-G1 card.

| estimator | seeds | σ (pb) | sd | χ²/dof | relative to vibegraph |
|---|---|---|---|---|---|
| vibegraph CLI, `--target-rel 0.0007` | 601–605 | **168 283.7 ± 51.1** | 112 | 1.00 | — |
| H-G1: MadEvent default survey, one reused directory | 31–35 | 165 724 ± 455 | — | — | −1.52 % |
| MadEvent default survey, **fresh** directories | 511–515 | 165 696 ± 597 | 1334 | **9.95** | −1.54 % |
| MadEvent `hard_survey = 2`, fresh, 10k events | 501–505 | 167 650 ± 175 | 392 | 0.98 | −0.38 % |
| same, 4× events | 531–535 | 167 660 ± 119 | 266 | 1.65 | −0.37 % |
| same, 10× events | 561–565 | 167 658 ± 58 | 131 | 1.27 | **−0.37 % (8σ)** |
| same, `sde_strategy = 2` | 541–545 | 167 384 ± 238 | 532 | 1.89 | −0.53 % |
| 1× equal-weight recombination | 501–505 | 168 243 ± 534 | 1193 | 9.73 | −0.02 % (noisy) |
| 4× equal-weight | 531–535 | 167 923 ± 127 | 283 | 1.99 | −0.21 % |
| 10× equal-weight | 561–565 | 167 931 ± 75 | 167 | 2.06 | −0.21 % (3.9σ) |

### The default survey

- Moving to `hard_survey = 2` alone raises MadEvent by 1.18 % (1 954 pb).
- The default runs scatter at χ²/dof 9.95.
- Per-channel `results.dat` shows where: under the default survey G2 reads
  45 156–47 059 pb across the five seeds, against 47 075–47 393 pb at
  `hard_survey = 2`. G1, G3 and G5 sit low as well.

H-G1's reused directory reads the same as the fresh ones (165 724 against
165 696). The bias is the survey, not the directory reuse.

This defect is already catalogued in
[madevent-default-survey-bias](../../../validation/madevent-default-survey-bias.md).
Its policy is that references are banked with `hard_survey = 2` in fresh
directories. H-G1's local references did neither.

### What remains at `hard_survey = 2`

- **It does not shrink with budget.** 167 650, 167 660 and 167 658 at 1×, 4×
  and 10× events.
- **It is not only the iteration-combination bias.** That bias shrinks with
  budget. Re-combined with equal weights, MadEvent is still −0.21 % at 10×.
- **Rare large-weight iterations appear at 1×.** Seed 504's G1 has an
  iteration reading 41 619 ± 15 842 among others at about 26 300. That is the
  pattern documented in
  [madevent-iteration-weighting-bias](../../../validation/madevent-iteration-weighting-bias.md).

### vibegraph is map-independent

`hq1_maps.rs` builds `ProtonIntegrand::new_with_maps` / `new_unbounded`, then
`use_run_card_scales`, `adapt_alphas(seed, 8000, 5, 0.5)` and
`adapt_grids(150000, 10, seed)`, over seeds 701–705:

```
vg maps=default   150k x10: n=5 mean=168099.7 sd=174.5 err=159.5 chi2/dof=0.24
vg maps=legacy    150k x10: n=5 mean=168311.5 sd=167.6 err=144.8 chi2/dof=0.27
vg maps=unbounded 150k x10: n=5 mean=167864.1 sd=574.0 err=271.3 chi2/dof=1.04
vg maps=softall   150k x10: n=5 mean=168217.0 sd=259.2 err=131.6 chi2/dof=0.77
```

These are four different channel constructions:

- `default`: the production maps;
- `legacy`: an isotropic angle and a log τ map;
- `softall`: soft-shaped angles;
- `unbounded`: no cut-implied floors.

All four agree with each other and with the CLI to within their errors. The
χ²/dof well below 1 on `default` and `legacy` says the library run's quoted
error is conservative at this budget; it does not say there is extra scatter.

The `unbounded` run has heavy-tailed iterations (iteration χ²/dof up to 5.65)
and reads lowest. That is the same failure mode MadEvent shows.

### At fixed energy the two programs agree

The card is the same with `lpp = 0` and `ebeam = 250`, at α_s = 0.118 on both
sides (vibegraph's events write `AQCDUP` `1.18000000e-01`). vibegraph reads
19 898.3 ± 7.1 (5 seeds, χ²/dof 2.79). MadEvent `hard_survey = 2` reads
19 915.2 ± 35.8, and its equal-weight value is 19 933 ± 52. The ratio is
0.9991 ± 0.0018.

So the hadronic offset is in the convolution over (x₁, x₂), not in the
partonic integration.

## 5. An oracle without hadronic sampling: σ̂(ŝ) folded with an LHAPDF luminosity

**Why this card.** With `etaj = -1` every cut on this card is invariant under
longitudinal boosts: `ptj` and `drjj` (Δy, Δφ for massless legs). The scale is
fixed. So, exactly,

σ = (α_s(μ)/0.118)³ · ∫ dτ L(τ) σ̂(√(τs)), where L(τ) = ∫ dy f_u(x₁) f_d(x₂).

**The two pieces:**

- **L(τ):** Gauss–Legendre over 8 y-segments × 32 nodes, on LHAPDF `xfxQ`
  (`/home/user/H-Q1-scratch/slice/lumi.cpp`). 16 and 64 nodes agree to 1e-7.
- **σ̂:** 24 vibegraph fixed-energy runs at √ŝ = 61 GeV to 8 TeV, each at
  `--target-rel 0.0007` (`slice/H-Q1-slices.sh`). These are fixed-beam
  integrals with no PDF and no τ or y sampling.

`slice/H-Q1-fold.py` interpolates ln σ̂ on ln √ŝ with a natural cubic spline
and integrates it against L on 16 Gauss nodes per knot interval:

```
knots 24 spline 178154.2 pb, linear-in-log 177387.1 pb, diff -4.31e-03
leave-one-out worst 1.78e-04
fold stat err 36.1 pb (2.03e-04)
[60,61] bound 0.012 pb
```

Leaving any one knot out moves the result by at most 1.8e-4. The region below
the first knot is bounded at 0.012 pb.

**Checking σ̂ against MadGraph.** MadEvent's own fixed-energy σ̂ on the same
card, `hard_survey = 2`, 3 fresh seeds each:

| √ŝ | vibegraph σ̂ | MadEvent | ratio MadEvent / vibegraph |
|---|---|---|---|
| 100 GeV | 929.15 ± 0.63 | 926.19 ± 2.26 | 0.9968 ± 0.0025 |
| 1 TeV | 30 910.7 ± 21.6 | 30 859 ± 94 | 0.9983 ± 0.0031 |
| 5 TeV | 57 348.8 ± 37.5 | 57 273 ± 110 | 0.9987 ± 0.0020 |

### The hadronic integrals against the fold

The card is H-G1's with `etaj = -1`: `mg/H-Q1-ud_run_card_noeta.dat`.

| estimate | σ (pb) | relative to the fold (178 154 ± 36) |
|---|---|---|
| vibegraph CLI, 5 seeds 631–635 | 177 998 ± 54 (χ²/dof 0.62) | **−0.088 % ± 0.037 %** |
| MadEvent `hard_survey = 2`, 5 fresh seeds 581–585 | 177 376 ± 246 (χ²/dof 1.29) | **−0.44 % ± 0.14 %** |
| MadEvent equal-weight | 177 472 ± 201 | −0.38 % |

The fold's error counts the knots' statistics only. vibegraph's fixed-energy
runs scatter at χ²/dof 2.79 at 500 GeV, so the true error is closer to 60 pb.
On that error vibegraph sits about 1.3σ below the fold.

### Where MadEvent's deficit sits

`slice/H-Q1-foldbins.py` bins the fold, on knot edges, against each side's
unweighted events: 50 000 per side, normalised to that side's own σ.

| √ŝ bin (GeV) | fold (pb) | MadEvent / fold | vibegraph / fold |
|---|---|---|---|
| 130–175 | 12 823 | 0.965 ± 0.016 | 0.996 ± 0.017 |
| 175–250 | 21 383 | 1.012 ± 0.013 | 1.001 ± 0.013 |
| 250–400 | 32 855 | 1.003 ± 0.010 | 1.001 ± 0.010 |
| 400–700 | 38 428 | 1.005 ± 0.010 | 0.993 ± 0.010 |
| 700–1000 | 21 379 | 1.004 ± 0.013 | 1.009 ± 0.013 |
| 1000–1500 | 19 699 | 1.002 ± 0.013 | 0.997 ± 0.013 |
| 1500–3000 | 19 303 | 0.980 ± 0.013 | 1.001 ± 0.014 |
| **3000–13000** | 4 023 | **0.896 ± 0.028** | 0.964 ± 0.029 |

The bins below 130 GeV carry 4.6 % of σ and read inside 2.5σ on both sides.

- **Above 1.5 TeV:** MadEvent is 3.5 % low (813 pb, −2.8σ). That matches its
  whole deficit of 778 pb.
- **Fixed energy at 5 TeV:** MadEvent's σ̂ is right (0.9987).
- **So:** MadEvent under-samples the large-τ end of its (x₁, x₂) map. This is
  the region of large rapidity separation, where σ̂ still grows with ŝ.

The binned comparison cannot localise below a few percent per bin. The
130–175 bin (−2.2σ) is not followed up.

## 6. The HT-binned 2 → 4 card

`u u > g g u u` alone: the H-G1 control, 5000 events, `ihtmin = 300`.

| estimator | seeds | σ (pb) | χ²/dof | relative to vibegraph |
|---|---|---|---|---|
| vibegraph CLI, `--target-rel 0.002` | 611–615 | **22 517.9 ± 19.7** | 0.89 | — |
| H-G1: MadEvent default survey, reused directory | 41–45 | 22 078 ± 38 | — | −1.95 % |
| MadEvent `hard_survey = 2`, fresh | 521–525 | 22 321.8 ± 34.9 | 1.08 | −0.87 % |
| same, 4× events (20 000) | 551–555 | 22 415.2 ± 19.0 | 0.89 | −0.46 % (3.7σ) |
| `hard_survey = 2` equal-weight | 521–525 | 22 528 ± 112 | 12.2 | +0.05 % |
| 4× equal-weight | 551–555 | 22 428 ± 25 | 1.76 | −0.40 % |

**Pointwise.** Per point the two sides agree (section 2): PDF, |M|²
including ¼, and MadEvent's cuts, `ihtmin` among them, on vibegraph's events.

**The ladder.** MadEvent climbs by +0.42 % from 1× to 4× events. A deficit
that moves with the event budget is sampling, not a factor.

**What is missing.** I built no fold oracle for this card. Whether MadEvent
converges onto vibegraph at higher budget is not measured.

## Diagnosis chain, and what would falsify it

1. **|M|²** per helicity and per flow: equal to 9e-16 (1).
2. **PDF, |M|², spin and colour average, symmetry factor, cuts and mirror**
   per point: equal, on both sides' events (2).
3. **Flux, (τ, y) map and normalisation:** exact to 1e-4 against a quadrature
   on uncut Drell–Yan (3).
4. **Partonic integration** at fixed energy: equal at 0.1–0.3 % on four
   energies (4, 5).
5. **The hadronic 2 → 3 integral.** vibegraph equals a sampling-free fold.
   MadEvent is low by 0.44 %, concentrated at √ŝ > 1.5 TeV, and the deficit
   does not move from 1× to 10× events (4, 5).
6. **The 1–2 % of H-G1:** about 1.2 % of that is the default survey (4), and
   the rest is item 5. The HT card's 2 % splits the same way, into 1.1 % of
   survey and a remainder that shrinks with budget (6).

**Falsifiers:**

- MadEvent at `hard_survey = 2` and ≥ 10× events converging on 178 154 for the
  `etaj = -1` card would put the remainder in MadEvent's budget after all.
- A fold built from MadEvent's own σ̂ at the 24 knots differing from this one
  by more than the 0.2 % the four spot checks allow would mean section 5's σ̂
  table needs re-reading.
- A second, independent fold (other PDF nodes, or MadEvent σ̂) reading below
  177 600 would put vibegraph on the high side.

## Gates

No source change. `git diff --stat` on the branch shows only this report. The
scratch examples under `vibegraph-lib/examples/hq1_*.rs` and the untracked
`.cargo/config.toml` were deleted before the commit. No cargo gate was rerun
for a tree identical to `cbfaec61` apart from this file.

## Found

1. **MadEvent under-samples large √ŝ on light-quark QCD 2 → 3, at
   `hard_survey = 2`, independent of budget.**
   - *Evidence:* sections 4 and 5. −0.37 % at 1×, 4× and 10× events; −0.44 %
     against the fold on the boost-invariant card; −10 % ± 3 % in
     3–13 TeV √ŝ, where MadEvent's own fixed-energy σ̂ is right.
   - *To close:* catalogue it beside the default-survey entry in
     `validation/madgraph-defects.md`, as a MadEvent sampling approximation
     that reads low. Bank any reference for such cards with that caveat, or
     compare against a fold oracle. An upstream report would need the channel
     that misses it: per-channel σ against the fold restricted to that
     channel's `AMP2` share.
2. **H-G1's σ references for these cards ran MadEvent's default survey.**
   - *Evidence:* the default runs read 1.2 % (2 → 3) and 1.1 % (HT) below
     `hard_survey = 2` on fresh directories, with χ²/dof 9.95.
   - *To close:* every local MadEvent σ used as a reference goes through
     `bank_seeded_sigma.sh`-style runs: `hard_survey = 2`, fresh directories,
     with `run_config.inc` recorded. The item's closing condition is then met
     on MadGraph's side.
3. **H-M2's universal ~0.1 % is not in vibegraph's PDF, |M|², flux or
   hadronic map.**
   - *Evidence:* section 3. Uncut Drell–Yan at +2.8e-5 ± 8.9e-5 against a
     deterministic quadrature on MadEvent's own PDF and matrix element, and
     PDF reader values exact on 183 points × 8 flavours.
   - *To close:* the same quadrature on H-M2's cut cards. `hq1quad_main.f`
     with `PASSCUTS` folded in, and more cos θ / y nodes at the cut edges.
     That says whether MadEvent or vibegraph moves there.
4. **vibegraph prints an overflowed iteration χ²/dof at fixed √ŝ = 60.5 GeV,
   just above the 3-jet threshold.**
   - *Evidence:* `/home/user/H-Q1-scratch/slice/rs_60.5_killed/integrate.log`.
     Iterations 1–66 read χ²/dof 0.25–0.41. From iteration 67 on it prints
     1e289-scale values while σ stays at 9.21e-12 GeV⁻². The run had not
     reached its 5e-4 target after 228 iterations.
   - *Suspicion:* a near-zero-variance iteration enters the χ² sum. That would
     also feed the consistency-scaled stop.
   - *To close:* find which iteration's variance collapses, guard the χ² sum
     against it, and add a test at a threshold-adjacent fixed-beam point.
5. **vibegraph's fixed-energy runs scatter above their quoted error at 3 jets.**
   - *Evidence:* χ²/dof 2.79 over 5 seeds at √ŝ = 500 GeV and
     `--target-rel 0.0005` (section 4). Hadronic runs read 0.6–1.0.
   - *To close:* a 20-seed calibration at fixed energy, as the seed policy
     asks before any gate reads it.

## Files

- `research/kb/sprints/cms-gridpack-coverage/sessions/H-Q1-report.md` (this
  file).
- **Scratch, under `/home/user/H-Q1-scratch/`:**
  - `mg/`: templates, seed directories, cards, `H-Q1-mg-seeds.sh`;
  - `vg/`: the binary, seed runs and events;
  - `drv/`: the MadEvent drivers, the point files and both sides' dumps;
  - `slice/`: fold oracle and slices;
  - `pdf/`: grid comparison;
  - `tools/`;
  - logs `H-Q1-*.log`.
