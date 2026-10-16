---
type: Session Report
title: "V2 report: the top-free deficit is confirmed, and the |cos θ_W| > 0.99 slice does not hold it"
description: "Five fresh MadEvent seeds put the top-free continuum 2.0 % below vibegraph (8 σ, seed χ²/dof 4.4); equal-weight recombination closes 0.8 points of it. The decisive slice falsifies V1's region: MadEvent integrates both-W |cos θ| > 0.99 correctly in isolation and is still 1.5 % low on the complement. The deficit sits entirely in the three double-peripheral channels G7, G8, G9."
status: draft
tags: [process-grammar, onshell-veto, madgraph-defect, info-cell, top, report]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-15}
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-15}]
---
Diagnosis only. No source change is committed. Every number below comes from a
run made in this session, apart from the rows marked V1. The scratch is under
`scratchpad/uux/v2/`: drivers in `drv/`, result lines in
`uux-v2-{me,vg,third}-results.txt` and `uux-v2-third-all.txt`, and every
MadEvent run's `results.dat` plus all `G*/results.dat` in
`runs/<tag>.G.tgz`. The run directories were deleted.

All runs use the top-free process `u u~ > w+ b w- b~ / t t~` (13 diagrams,
pinned MadGraph 3.7.1 via `mg5_pinned.sh`, unpatched because there is no `$`).
They use `onshell_uu_run_card.dat` with `lpp1=lpp2=0`, `ebeam=250`,
`nevents=10000` and `use_syst=False`. "Banked card" means its dynamical scale.
"Fixed μ" means `fixed_ren_scale = fixed_fac_scale = True`, μ = 91.188, which is
the scale at which the standalone-based third opinion (α_s = 0.118) is
comparable. vibegraph is the `release-debug` CLI at `90e1427`. The slice runs
used an uncommitted `VG_PROBE_FWD` filter in `Cuts::pass`, which has been
reverted (`git diff --stat` is empty).

## Verdict

1. **The deficit is confirmed.** Five fresh MadEvent seeds of the top-free
   process at the banked card give 0.044956 ± 0.00011 pb. That is **−2.03 %
   from vibegraph** (0.0458886 ± 0.000022), 8.0 σ on the seed scatter. The
   seeds scatter at **χ²/dof 4.4** about their own mean, so MadEvent's quoted
   error understates its spread by about 2×. At fixed μ, two MadEvent seeds sit
   **−2.3 % from the third opinion** (4.3 σ). vibegraph sits −0.14 % from the
   third opinion there (1.2 σ).
2. **Iteration weighting is about 40 % of it.** Recombining each channel's
   iterations with equal weights moves every seed up, by +0.42 % to +1.21 %.
   The equal-weight sums are still **−1.27 %** from vibegraph (5.2 σ,
   χ²/dof 3.4).
3. **The slice falsifies V1's region attribution.** Run separately, MadEvent
   integrates the both-W |cos θ| > 0.99 slice (FW) to within 0.5–1 % of the
   reference. On the complement (NF) it is still **−1.46 %** low over five
   seeds (3.9 σ, χ²/dof 8.5). The NF deficit is 0.00065 pb, which is ~75 % of
   the unsliced 0.00090 pb, while FW carries only 3.1–3.2 % of σ. The
   heavy-weight region V1 located is real (FW is 3.14 % of σ by the third
   opinion), but it is not where MadEvent loses the cross section.
4. **The cause, as far as the channel table shows it.** The whole deficit sits
   in the three double-peripheral configurations **G7, G8, G9**: t-channel
   d-quark exchange emitting both W's, with g* → b b̄ hung off the chain.
   - They carry 72 % of σ. They are the only channels whose seed-to-seed
     scatter exceeds their quoted error: χ²/dof 19, 8 and 10 unsliced, 5, 6
     and 33 on NF. Every other channel is at 0.2–1.8.
   - Their summed reported value is 2.0–2.7 % below what vibegraph leaves for
     them after the well-behaved channels are subtracted, which is the whole
     gap.
   - G8 and G9 are CP mirrors (W⁺ emitted first from u, against W⁻ first from
     ū), so their exact integrals are equal. MadEvent's values for them differ
     by up to 5 % within a run, in either direction.

## 1. Five fresh seeds (scope 1)

Banked card, top-free, MadEvent 10k. "Equal-weight" is the sum over channels
of each channel's unweighted iteration mean (`drv/chan_table.py`, same
algorithm as V1's `iter_combine.py`).

| seed | σ reported (pb) | equal-weight (pb) | equal − reported | wall (s) | events written |
|---|---|---|---|---|---|
| 20260925 (V1) | 0.045176 ± 0.000173 | 0.045537 | +0.80 % | 1313 | 1704 |
| 20260926 | 0.044685 ± 0.000129 | 0.044959 | +0.61 % | 557 | 4400 |
| 20260927 | 0.045032 ± 0.000137 | 0.045358 | +0.72 % | 1029 | 2262 |
| 20260928 | 0.044747 ± 0.000161 | 0.045161 | +0.93 % | 729 | 7985 |
| 20260929 | 0.045330 ± 0.000111 | 0.045522 | +0.42 % | 445 | 4251 |
| 20260930 | 0.044986 ± 0.000175 | 0.045529 | +1.21 % | 673 | 1840 |

Statistics: unweighted mean, error = sd/√n, χ²/dof about the own mean with
quoted errors (`drv/stats.py`):

| estimator | configuration | σ (pb) | seeds | χ²/dof | vs reference |
|---|---|---|---|---|---|
| vibegraph, `--target-rel 2e-3` (V1 seeds) | banked | 0.0458886 ± 0.000022 | 5 | 0.38 | reference |
| vibegraph, `--neval 1e6 --target-rel 1e-3` | banked | 0.045918 (0.045943, 0.045892) | 2 | — | +0.06 % |
| MadEvent reported, fresh | banked | 0.044956 ± 0.00011 | 5 | 4.43 | −2.03 % (−8.0 σ) |
| MadEvent reported, with V1's seed | banked | 0.044993 ± 0.00010 | 6 | 3.69 | −1.95 % (−8.7 σ) |
| MadEvent equal-weight, fresh | banked | 0.045306 ± 0.00011 | 5 | 3.40 | −1.27 % (−5.2 σ) |
| MadEvent equal-weight, with V1's seed | banked | 0.045344 ± 0.00010 | 6 | 3.03 | −1.19 % (−5.4 σ) |
| third opinion (`third6.py`, 3 channels, 2e6 pts) | fixed μ | 0.054842 ± 0.000051 | 5 | 0.17 | reference |
| vibegraph, default | fixed μ | 0.054694 ± 0.000026 | 5 | 0.39 | −0.27 % (−2.6 σ) |
| vibegraph, `--neval 1e6` | fixed μ | 0.054765 ± 0.000036 | 3 | 1.84 | −0.14 % (−1.2 σ) |
| MadEvent reported | fixed μ | 0.053601 ± 0.00029 (0.053887, 0.053314) | 2 | 5.78 | −2.26 % (−4.3 σ) |
| MadEvent equal-weight | fixed μ | 0.054002 (0.054319, 0.053685) | 2 | — | −1.53 % |

Against the third opinion directly, only fixed μ is available, because the
third opinion cannot reproduce MadEvent's dynamical scale. At the banked card
the reference is vibegraph, which sits −0.14 % from the third opinion at
fixed μ when given enough points per iteration.

## 2. The decisive slice (scope 2)

FW: both W's at |cos θ| > 0.99 in the lab, which is the partonic CM at fixed
beams. NF is its complement. MadEvent cuts in `dummy_cuts`
(`drv/fwd_cut.f`); vibegraph cuts in `Cuts::pass` (probe). The two pieces sum
to the whole on both sides: vibegraph FW + NF = 0.045900 against 0.045918
unsliced.

| estimator | configuration | slice | σ (pb) | seeds | χ²/dof | vs reference |
|---|---|---|---|---|---|---|
| third opinion, dedicated FW sampler (`third_fw.py`) | fixed μ | FW | 0.0017232 ± 0.000001 | 5 × 2e6 | 0.16 | reference |
| third opinion, FW tally of the full sampler | fixed μ | FW | 0.001771 ± 0.000044 | 5 × 2e6 | 0.47 | +2.7 % (1.1 σ) |
| vibegraph, default neval, `--target-rel 1e-2` | fixed μ | FW | 0.0016724 ± 0.0000058 | 5 | 0.81 | **−2.95 % (−8 σ)** |
| vibegraph, `--neval 1e6` | fixed μ | FW | 0.001721 ± 0.000006 | 1 | — | −0.13 % |
| MadEvent | fixed μ | FW | 0.0017041 ± 0.0000056 | 1 | — | −1.1 % (−3.4 σ) |
| vibegraph, `--neval 1e6 --target-rel 3e-3` | banked | FW | 0.0014867 ± 0.000002 | 3 | 0.77 | reference |
| vibegraph, default neval | banked | FW | 0.0014454 ± 0.0000048 | 5 | 0.66 | −2.8 % |
| MadEvent | banked | FW | 0.0014800 ± 0.0000016 (0.0014815, 0.0014784) | 2 | 0.26 | −0.45 % (−2.7 σ) |
| vibegraph, default | banked | NF | 0.0444128 ± 0.000019 | 5 | 0.29 | reference |
| MadEvent reported | banked | NF | 0.043763 ± 0.00016 | 5 | 8.51 | **−1.46 % (−3.9 σ)** |
| MadEvent equal-weight | banked | NF | 0.044073 | 5 | — | −0.77 % |

MadEvent NF per seed (20260925–29): 0.044200 ± 0.000134, 0.043487 ± 0.000123,
0.043319 ± 0.000122, 0.044025 ± 0.000117, 0.043786 ± 0.000110.

**FW share (third opinion):** 0.0017232 / 0.054842 = **3.14 %** of the
top-free σ at fixed μ. vibegraph at the banked card gives 3.24 %. V1's signed
region (W⁺ forward, W⁻ backward) is 98 % of FW. The both-backward
combination is 0.0015 %.

**Reading.**
- The brief's criterion is met: MadEvent agrees with the reference inside the
  slice, to 0.45 % at the banked card and 1.1 % at fixed μ, against a 2 %
  whole-process gap.
- The deficit is elsewhere. NF alone is −1.46 %, and its absolute deficit,
  0.00065 pb, is about 75 % of the unsliced 0.00090 pb.
- Pairwise, MadEvent's FW + NF (seeds 25, 26: 0.045682, 0.044965) against the
  unsliced runs of the same seeds (0.045176, 0.044685) shows that removing FW
  helps a little. The NF runs alone already carry most of the deficit.
- **What remains:** MadEvent under-samples G7–G9 in a region that is not
  confined to both W's within 8° of the beams. It may be one W forward, a
  wider cone, or the soft-m_bb corner at any angle. The slice cannot localise
  it further.
- The FW cut does change MadEvent's behaviour in one respect. It makes the
  region sampleable: the FW run writes all 10000 events and its G8 and G9
  agree to 0.9 %.

## 3. The channel table (scope 3)

Per-channel reported values across the six unsliced banked seeds
(`drv/chan_seeds.py`). The quoted error is the mean of the per-seed quoted
errors.

| channel | configuration | mean (pb) | seed sd | quoted | χ²/dof | equal-weight mean |
|---|---|---|---|---|---|---|
| G9 | t-chain from ū: W⁻, then W⁺, g* at the u end | 0.011655 | 2.0e-4 | 7.5e-5 | 10.2 | 0.011739 |
| G8 | t-chain from u: W⁺, then W⁻, g* at the ū end | 0.011473 | 3.0e-4 | 1.0e-4 | 8.2 | 0.011689 |
| G7 | g* emitted between the two W's | 0.009460 | 2.0e-4 | 4.6e-5 | 18.9 | 0.009478 |
| G13.02 | | 0.004966 | 1.4e-5 | 2.8e-5 | 0.3 | 0.004975 |
| G12.02 | | 0.004934 | 3.4e-5 | 3.0e-5 | 1.1 | 0.004945 |
| G10 | | 0.001246 | 1.6e-5 | 1.5e-5 | 1.1 | 0.001256 |
| G11 | | 0.001247 | 7.2e-6 | 1.5e-5 | 0.3 | 0.001250 |
| G5.02, G1, G3.02 | | < 1.2e-5 | | | 0.2–1.8 | |

The configurations are read from `configs.inc`: config 7 is the s-channel g*
on (b, b̄) plus a t-chain ū → W⁻ · g* · W⁺; configs 8 and 9 are the t-chains
starting from u and from ū respectively. The subtraction reads as follows:

- vibegraph's total minus MadEvent's well-behaved channels (0.012404 pb)
  leaves 0.033485 pb for G7 + G8 + G9.
- MadEvent's mean for G7 + G8 + G9 is 0.032588: −0.00090 pb, −2.7 % of these
  channels, which is the entire gap.
- On NF the same subtraction gives −0.00065 pb, again the entire NF gap.

**Does the grid ever find the region?** The iteration lines carry `xeff`, the
per-point relative standard deviation (`dsample.f:2006`), and `xwmax`. An
iteration with `xeff` > 5 has sampled at least one very large weight.

- **G8:** heavy iterations (`xeff` 25–70, `xwmax` 180–1170) occur in every
  seed: 6, 7, 9, 7, 9 and 9 of 9 iterations. They read 0.0116–0.0123. The
  light iterations (`xeff` ≈ 2, `xwmax` ≈ 1.5) read 0.0099–0.0106. Seed
  20260926 is the clearest case: its last three iterations lose the region
  (0.010441, 0.011207, 0.009916), and the final one, at 0.009916 ± 0.000103,
  carries the smallest error and dominates the inverse-variance result
  (0.011276).
- **G9:** the same alternation. Heavy in 9, 3, 8, 6, 0 and 9 iterations.
- **G7:** never produces a heavy iteration: `xeff` ≤ 1.5 in all 54 iterations
  over six seeds. Yet its seeds scatter at 4.4× its quoted error (0.00920 to
  0.00970). G7's grid does not sample the region rarely; it does not sample it
  at all, and where its grid settles decides how much of the region it covers.
  Equal-weight recombination cannot correct this, which is why it closes only
  0.8 of the 2 points.
- With the FW cut, G7, G8 and G9 read 0.000481, 0.000400 and 0.000403, with no
  seed-level pathology.

So V1's `G8` is one of three. G7, the self-mirror configuration, is the
silent one, and G8 and G9 are the noisy ones.

## 4. Side findings on this side and on the third opinion

- **vibegraph under-integrates the FW slice at the default `--neval`.** At
  120k points per iteration, 13 timelike-only channels ("0 peripheral" in the
  phase-space line) and 3 % acceptance:
  - it reads −2.95 % at fixed μ and −2.8 % at the banked card;
  - its five seeds agree with each other (χ²/dof 0.8) and each run reports its
    1 % target met;
  - at `--neval 1e6` it reads −0.13 % (fixed μ) and matches MadEvent's FW to
    0.35 % (banked).
  
  The run summary's χ²/dof on these sliced runs is meaningless: 8845, 824,
  then values of order 1e180 and more, printed as integers hundreds of digits
  long. A seed-22 run at the banked card took 240 iterations to meet 1 %, and
  a 3e-3 target gave up at the 300-iteration cap. The effect on the unsliced
  process is FW's −3 % × 3.2 % ≈ 0.09 %. That is consistent with vibegraph's
  −0.27 % (default) and −0.14 % (`--neval 1e6`) against the third opinion,
  and immaterial to this gap.
- **The third opinion's quoted errors are conservative by about 2×.** Seed
  χ²/dof is 0.16 (FW) and 0.17 (ALL), with p ≈ 0.04 each for 4 dof. The
  sample variance of a heavy-tailed weight distribution over-estimates more
  often than not at 2e6 points. The means agree across densities: the FW
  sampler at two settings gives 0.0017232 and 0.0017214 (two seeds,
  `DELTA2=0.2`, sign mixtures 0.6); the full sampler's FW tally gives
  0.001771 ± 0.000044. The Jacobian check: |M|² = 1 over the full space gives
  the 4-body volume 2.4363e6, 2.4345e6 and 2.4276e6 (± 0.0026e6) at three
  settings, against V1's 2.4236–2.4370e6.

## Brief corrections

- **The falsification criterion of scope 2 tests the wrong thing.** A
  separately cut MadEvent run measures whether MadEvent can integrate the
  slice when the slice is the whole integrand. It does not measure what the
  unsliced run does there. Agreement inside the slice would be expected under
  V1's attribution too, because the defect is sampling, not integrand. The
  measurement that discriminates is the complement. That run was made: NF is
  −1.46 %, so the attribution to the |cos θ_W| > 0.99 region is falsified
  either way.
- **"Both W's at |cos θ| > 0.99" is not V1's measured region.** V1's tally
  (`third5_regions.py`, `fwd99`) is signed: W⁺ cos > 0.99, W⁻ cos < −0.99.
  It is 98 % of the unsigned slice, so the difference is immaterial. V1's
  2.9 % share was measured on the `$` row; the top-free share is 3.14 %.
- **"V1 named G8."** G8 is one of three channels with the defect, and G7, not
  G8, shows it without ever producing a large weight.
- **Run time.** V1 saw 22–75 min per top-free run. Here, under lighter load,
  ALL runs took 7.5–17 min and NF 4–12 min. FW took 9–27 min at the banked
  card and 2 min at fixed μ.

## Banking request

None is needed for the attribution. If the `uu_tt` row is documented rather
than gated, V1's request stands (`uu_notop` row, seeds 20260925–29, each run's
`G*/results.dat` kept). This session's expectation for it, from six seeds:
0.04499 ± 0.00010 pb, seed χ²/dof ≈ 4, −2.0 % from vibegraph; equal-weight
−1.2 %.

## What did not fit

- The five further unsliced seeds (20260931–35) were dropped in favour of
  three more NF seeds, which settled the slice question.
- No localisation inside NF was attempted: no one-W-forward or wider-cone
  slice, and no m_bb slice.
- No higher-budget top-free MadEvent run was made.
- Fixed μ has two MadEvent seeds for ALL and one for FW.

## Gate results

- `cargo build -j 2 --profile release-debug -p vibegraph`, with the probe
  applied: `Finished`, `real 12m45.863s`. The probe was reverted afterwards
  (`git checkout vibegraph-lib/src/cuts.rs`; `git diff --stat` empty).
- `cargo test` and `pixi run --skip-deps validate`: not run. This commit adds
  one Markdown file and no source.
- `python3 scripts/kb.py index`: `wrote 4 index files`.
  `python3 scripts/kb.py lint`: `568 concepts, 193 backlog items; 0 errors,
  2 warnings`. Both warnings were already there, on performance backlog items.

## Found

- **MadEvent mis-integrates the double-peripheral configurations of
  `u u~ > w+ b w- b~ / t t~` (G7, G8, G9), −2.0 % on σ (six seeds, 8 σ).**
  Their seed scatter is 8–19× their quoted variance; every other channel is
  consistent. The region is not confined to both W's at |cos θ| > 0.99. To
  close: decide that rows dominated by these configurations are not gated
  against MadEvent's σ (reference-side defect, measured cause: G7–G9), or
  localise the region further with NF sub-slices.
- **CP-mirror channel pairs are a free per-channel oracle on MadEvent
  references.** G8 and G9 must integrate to the same value, but MadEvent's
  differ by up to 5 % within a run (seed 20260925: 0.011267 against 0.011890).
  A check that flags |G_i − G_mirror(i)| beyond the quoted errors in a banked
  run's channel table would have caught this row without any second
  generator. To close: add mirror-pair consistency to the reference-run record
  for symmetric-beam processes.
- **vibegraph under-integrates heavily cut slices at the default `--neval`
  while reporting its target met:** −3 % on a 3 %-acceptance slice, five
  mutually consistent seeds, correct at `--neval 1e6`. To close: find why the
  per-channel 512-point floor and Neyman allocation leave the cut region
  under-covered when acceptance is low, and make the convergence stop see it.
- **vibegraph's printed VEGAS χ²/dof overflows on sliced runs.** Values such
  as 8845 and integers of 1e180 or more appear in the run summary of the FW
  runs. The statistic is unusable as a diagnostic exactly where it is needed.
  To close: guard the χ² against channels or iterations with near-zero
  variance and report a finite, documented statistic.
- **vibegraph sits −0.14 % to −0.27 % below the third opinion on the top-free
  continuum at fixed μ** (1.2–2.6 σ). It moves toward the third opinion with
  `--neval`, so it is the same budget effect as the FW finding, at a level
  immaterial to any current gate. To close: nothing unless a gate tightens
  below 0.3 %.
