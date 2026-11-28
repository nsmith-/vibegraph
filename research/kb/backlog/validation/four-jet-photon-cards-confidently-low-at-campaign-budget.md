---
type: Backlog Item
title: "γ + 4-jet cards read 5–30% low: every seed puts the γ+4j multiplicity at 0.12–0.75 of MadGraph's own, with γ+1j and γ+2j agreeing"
description: "On 232 and 252 every seed reads γ+4j at 0.12–0.75 of MadGraph's per-multiplicity survey. On 252, cuts, veto, scales and |M|² match MadGraph's events, so it is phase-space coverage."
area: validation
state: open
priority: high
closes_when: "The γ+4j deficit on cards 232 and 252 is attributed (per-subprocess σ against MadGraph's P3_* survey, then an unmatched equal-cuts p p > a j j j j comparison if needed) and either fixed, with both cards' γ+4j within 2σ of MadGraph over 5 seeds at a budget shown converged, or registered as a deviation with its cause."
blocked_by: []
opened: 2026-11-27
tags: [vegas, multichannel, confidently-wrong, multi-jet, photon, cms, budget]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-27}
---
Found by `vg-validator`'s v0.2.3 pre-tag stage B and its follow-ups on the fix
build 2431b587. None of these cards ran in v0.2.2.

| card | rung | seeds | unweighted mean vg/MG | spread |
|---|---|---|---|---|
| 252 GJets-4Jets dRGJ-0p2 HT40-70 | 1× | 4, plus 1 refused | 0.907 ± 0.016 | 0.032 |
| 252 | 4× neval (≈1.15× points) | 5 | 1.012 ± 0.027 | 0.061 |
| 993 GJets_DR-0p4 HT-100to200 | 1× | 5 | 0.958 ± 0.060 | 0.135 |
| 232 GJ-4Jets HT-600to1000 PTG-100to200 | 1× | 4, plus 1 refused | 0.696 ± 0.053 | 0.105 |
| 232 | 4× neval (≈1.16× points) | 3, plus 2 refused | 0.759 ± 0.034 | 0.059 |

Every rung ran `--fixed-budget --niter 3`, so σ comes from the third iteration
alone. With 1068 grids, about 1030 channels sit at the 512-point floor per
iteration, so 4× `--neval` moved the last iteration from 2.24M to 2.50M points
(232) and 2.25M to 2.58M (252): ×1.12–1.14 in the iteration σ comes from,
×1.16–1.18 in total evaluations (`vg-validator`, from the job logs). The 1× →
"4×" shifts on 252 (+0.105) and 232 (+0.063, 1.0σ) are therefore seed scatter
on heavy-tailed rows, not convergence. A real budget step raises the iteration
count, or neval well above 512 × channels (about 550k here).

On stage B's one seed each, about 20 independent 4-jet rows (GJ-4Jets,
GJets-4Jets, WBtoLNuB-4Jets, DYBto2LB-4Jets, GJets_HT/DR) averaged about 0.93.

On 252 at 4× neval, the two seeds with a small quoted error read 0.951 (1.2%) and
0.961 (1.6%), and the seeds that read above 1 carry 7–11% errors. That is the
missed-region signature.

Card 232 differs: its four seeds (0.651, 0.696, 0.842, 0.596; errors 9–13%)
agree on a deficit of about 30% with χ²/dof ≈ 1.7 about their mean, and seed 5
is refused (5.1 effective points). Only the highest seed carries the largest
error. Five seeds cannot tell persistent undersampling from a bias here; a real 4×
budget rung (more iterations) decides it (mean moves toward 1: coverage; stays
near 0.7: a defect). At 4× neval its three surviving seeds read 0.808, 0.777
and 0.693, with seeds 1 and 5 refused (4.6 and 7.5 effective points).
Seed 1 reproduces stage B's value on 54cecfac (0.6507).

Five seeds per rung cannot calibrate the 0.907 → 1.012 shift on a row this
heavy-tailed, so 20 seeds per rung are needed. These cards use
`sde_strategy = 1`, so the MadGraph channel-weight difference
(`scales-pdf/channel-cut-madgraph-versions`) does not apply.

## Per-channel reading of card 232

`vg-validator` read the per-channel σ, error and neval of all 1068 channels from
the result records of 7 completed 232 seeds (4 at 1×, 3 at neval 480000, build
2431b587), taking each channel's effective N as (σ/δσ)².

- 1054–1059 channels sit at the 512-point floor at 1× (1034–1040 at 480000).
  Together they carry 0.26–0.50 of MadGraph per seed, at a median effective N
  of 3–8 each, many at 1.
- By final state, per seed: γ+2j 0.045–0.052 (stable), γ+3j 0.218–0.286,
  γ+4j (945 channels) 0.311–0.519. The two highest seeds are the two whose γ+4j
  sum is about 0.5.
- The well-sampled channels are stable: γ+2j (3,0,0) reads 0.0195 ± 0.0015 at
  effective N ≈ 925, and the top γ+3j channels have effective N ≈ 117.
- The channels with the widest seed spread are all floor channels with
  effective N 1–5, e.g. (5,9,383) at 0.004–0.090 of MadGraph.
- The mean over the 7 seeds of the summed channel σ is 0.723.

That fits about 1000 barely sampled channels, each unbiased in expectation but
with a median far below its mean. A likely upstream cause is the α-survey: its
points per iteration are clamped to 40000 (`survey_points` in
`vibegraph-cli/src/integrate.rs`), about 37 per channel here, too few to rank
1068 channels, so almost all of them get the floor. A bias in the γ+4j channels
themselves is not ruled out. The gridpack banks only MadGraph's total
(101.95 pb), so a per-multiplicity MadGraph survey (@0–@3 separately) is the
oracle that would separate the two.

## MadGraph's per-multiplicity split: γ+4j carries the deficit

Both gridpacks keep MadGraph's own survey and refine results
(`process/madevent/SubProcesses/P0–P3_*/G*/results.dat`, 823 channel files
each, P0–P3 being @0–@3). `vg-validator` summed them by P group; they reproduce
MadGraph's total to 0.2% (232) and 0.8% (252).

| card | γ+1j (@0) | γ+2j (@1) | γ+3j (@2) | γ+4j (@3) |
|---|---|---|---|---|
| 232, MadGraph share | 0 | 0.049 | 0.260 | 0.691 |
| 252, MadGraph share | 0.027 | 0.500 | 0.408 | 0.065 |

vibegraph per final state against MadGraph's own value for that multiplicity,
over every completed seed (232: 7 seeds at niter 3; 252: 10 seeds, one at
niter 6):

- γ+1j 0.98–1.09 and γ+2j 0.92–1.06 on every seed.
- γ+3j 0.81–1.31: scatter around 1, the undersampling seen in the per-channel
  reading.
- **γ+4j low on 17 of 17 seeds:** 232 at 0.45–0.75, 252 at 0.12–0.71. 252's
  niter-6 seed, with 4× the averaged points, reads 0.21.

No seed reads above 1, which at first suggested a missing or suppressed γ+4j
contribution. The niter-6 rung below weakens that: γ+4j moves up with budget.
The γ+3j scatter is a separate effect.

MadGraph's γ+4j subprocess shares, to localise it: on 232, gq → γgggq 0.573,
qq → γggqq 0.286, gq → γgqqq 0.089, gg → γggqq 0.035, qq → γqqqq 0.011,
qq → γgggg 0.004, gg → γqqqq 0.001; on 252, 0.542, 0.172, 0.146 (gg → γggqq),
0.123, 0.011 and 0.006 (gg → γqqqq).

The highest multiplicity of an MLM-matched sample is validated elsewhere:
`sigma_ll_0j2j_mlm_vs_madevent` gates DY's @2 at +0.88% (|pull| < 3). Candidates
specific to these cards that are unchecked: the photon's own cuts at five
outgoing legs (the `DR` and `dRGJ` cards cut on photon–jet ΔR), a subprocess
class missing at four partons, and the highest-multiplicity scale or
max-jet-flavour handling.

## Per-subprocess reading: every γ+4j class is suppressed together

The census matches MadGraph exactly: vibegraph enumerates 15 / 95 / 145 / 485
subprocesses for γ+1j / 2j / 3j / 4j, and MadGraph's `P*/processes.dat` lists
the same counts of non-mirror processes on both cards. Against MadGraph's P3_*
values, every γ+4j class (gg → γggqq, gg → γqqqq, gq → γgggq, gq → γgqqq,
qq → γgggg, qq → γggqq, qq → γqqqq) reads low on every seed, and the classes
move together seed by seed: 232 at 0.45–0.75 per seed, 252 at 0.12–0.71. A
factor common to all four-parton final states (cuts, matching, scale or
phase-space coverage) fits; a defect in one class does not.

252 is far worse than 232. 252 cuts HT to 40–70 GeV and photon–jet ΔR at 0.2;
232 cuts HT to 600–1000 and ΔR at 0.25. Both use ickkw 1, xqcut 10,
auto_ptj_mjj True and dynamical_scale_choice −1. Four jets of pT ≳ 10 under
HT ≤ 70 is a thin corner just above threshold, which either a sampler that
cannot reach it or a cut wrong only there (HT's jet definition, ptj = xqcut,
the kt cut over six jet pairs) would explain. The DY @2 gate has one jet pair.

Rungs built by neval share their floor channels' samples at the same seed:
252 seed 2 reads bit-identical γ+4j values at neval 120000 and 480000.

Next: MadGraph's own @3 events from each gridpack, replayed event by event
through vibegraph's cuts, |M|² and clustered scales.

## Replay of MadGraph's events, and the niter-6 rung on 252

From 2000 events of the 252 gridpack (145 at γ+4j), replayed through the
production `Cuts::compile` on the gridpack's run card and a copy of
`validate_mlm_records`:

- vibegraph's cuts accept 2000 of 2000 events, 145 of 145 at γ+4j. Mutating
  htjmin, htjmax, pta, draj or xqcut rejects events, and the one-cut-off
  recompile names the mutated cut, so the check sees each cut.
- No event is vetoed by the clustering. On the events where a configuration
  reproduces SCALUP, AQCDUP and pt_clust (124 of 145 at γ+4j; the rest are the
  known permuted-P1 class, at similar rates on every multiplicity), the
  `<totfact>` αs/PDF factor matches, worst 1.4e-7.
- |M|² against MadGraph standalone at the events' momenta is not yet compared.

Five seeds at `--niter 6` (4× the averaged points of niter 3) read
0.956 ± 0.030 (spread 0.067, no refusals) against 0.907 ± 0.016 at niter 3.
Per multiplicity: γ+1j 1.019, γ+2j 1.004, γ+3j 0.986 (spread 0.144),
**γ+4j 0.373 (spread 0.125), up from about 0.17 at niter 3**. The γ+4j
fraction roughly doubles for 4× the averaged points, which, with cuts, veto
and scales clean, favours slow coverage of the γ+4j corner over a missing term.

## |M|² matches on 252: the deficit is phase-space coverage

MadGraph 3.7.1 standalone, with the gridpack's param card and
`restrict_ckm_no_b_mass`, against vibegraph's `eval_m2` × initial average /
Π n_k! at the events' own partonic momenta: 1 ± 6e-8 or better on
u g → a g g g u (36 events), g g → a g g u ū (8), u u → a g g u u (5),
u g → a g u c c̄ (6) and d g → a g g g d (7). Non-vacuity: with MadGraph left
on its default card, every event reads 1.0285790 = 132.507 / 128.8253, the
aEWM1 ratio.

With cuts, veto, αs/PDF reweighting and |M|² all matching MadGraph's own
events, card 252's γ+4j deficit is phase-space coverage, a sampler property.
The niter-6 rung spends the same ~2.25M points per iteration as niter 3 but
keeps four adapted iterations instead of one, so the γ+4j rise (0.17 → 0.37,
spread 0.125) fits grids still adapting toward the region. Card 232's replays are pending.

vibegraph and MadGraph count different diagram numbers at identical |M|² on
these subprocesses (e.g. 428 against 516), likely a four-gluon-vertex
convention.

## Where the sampler loses it (card 252, from existing records)

How these runs allocate, from `budget.rs`/`proton.rs` and the logs:
- `BlockAllocation::ByAlpha`: the per-channel split α is set once by the
  Kleiss–Pittau α-survey before grid training and never changes. There is no
  re-allocation between iterations in this mode.
- The survey is per multiplicity: 945 γ+4j channels over 6 × 40000 points,
  about 42 points per channel per pass, on untrained grids.
- The floor counts accepted points: 512 / acceptance, with the multiplier
  capped at 4. Every channel accepts under one point in four, so each draws
  2048 points per iteration with fewer than 512 accepted. The γ+4j channels
  take 1.94M of the 2.23M points per iteration (86%): γ+4j is not starved
  overall, but each of its channels is thin.

What the records show (niter 6, 5 seeds):
- γ+4j's final σ share against its α share has quantiles (5/25/50/75/95%)
  about 0.02 / 0.08 / 0.22 / 0.7 / 4–8, so α barely predicts where σ lands.
  Per-channel χ²/dof has median 78–156, and the median relative error is 0.55.
- **The α-survey's own γ+4j estimate is not low:** 0.654, 0.178, 2.786, 1.924
  and 0.733 of MadGraph, mean 1.25 ± 0.49. Every trained result at every rung
  is 0.12–0.71, and niter 6 reads 0.37 ± 0.06, about 1.8σ below the survey. The
  survey sees rare large weights (per-point spread/mean about 130) that the
  trained per-channel grids, trained on under 512 accepted points per channel
  per iteration, then miss. At 1.8σ this is suggestive, not proven.
- Floor membership and α are identical between niter 3 and niter 6 at the same
  seed. Half of γ+4j's σ sits in 2–8 of the 945 channels, and the top 10
  channels carry 107–187% of the niter-3 → 6 gain: adaptation in a few dominant
  channels, still moving at iteration 6.

No one-to-one channel map to MadGraph exists (MadGraph has 4 / 24 / 128 / 667
channels against vibegraph's 3 / 15 / 105 / 945), so a per-channel MadGraph
comparison is not available. The next diagnostic is vibegraph's mixture
density at MadGraph's own γ+4j events: f/g on events distributed as f locates
the region the channel maps under-cover.

## Card 232: the same four checks are clean

From 2000 events of the 232 gridpack (1392 at γ+4j, 69.6% against the survey's
0.691), with the run and param cards byte-identical to the batch's: the cuts
(HT 600–1000, pta 100–200, draj 0.25, ptj = xqcut 10) accept every event; the
clustering vetoes none; `<totfact>` agrees on 1261 of 1261 γ+4j candidates
(worst 2.1e-7); and |M|² matches MadGraph standalone to 1.4e-7 or better on
831 γ+4j events over five subprocesses. Both cards' deficits are sampling.
