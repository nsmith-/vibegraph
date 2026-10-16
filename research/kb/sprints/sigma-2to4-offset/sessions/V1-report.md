---
type: Session Report
title: "V1 report: the fixed-beam 2 → 4 offset is MadEvent's iteration weighting"
description: "On fixed-beam u u~ > e+ e- g g, vibegraph agrees with an independent non-adaptive estimator to 0.05 %; MadEvent reads 0.1–0.2 % low because it combines its VEGAS iterations with mean²/σ² weights, and its own iterations averaged without them land on vibegraph. The m(ll) > 150 'tail surplus' was an artefact of MadEvent's event sample, not of σ."
status: draft
tags: [validation, sigma, madevent-reference, 2to4, report]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-15}
sources:
  - {id: base, resource: "https://github.com/nsmith-/vibegraph/commit/05c214b", title: "Measured at 05c214b (sigma/2to4-offset base), no source changes"}
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-15}]
---
Diagnosis only. No source change, no probe test committed. Every MadEvent run
below comes from the **pinned MadGraph 3.7.1** (`validation/madgraph/mg5_pinned.sh`
under `pixi run -e madgraph`, banner `VERSION 3.7.1`). No 3.5.7 run was made.
Each MadEvent seed ran in a **fresh copy of a never-run process directory**
(`cp -a` of a just-generated template, the same thing as regenerating, because
generation is deterministic and all carried-over state is made by runs), with
`nb_core = 2`. vibegraph ran at `-j 2` from the warm `target/release-debug`
binary. Throwaway work dirs, logs and probe code are under the scratchpad
(`2to4-work/`).

## Verdict

- **Side:** MadEvent. **Region:** none in particular. The deficit is spread
  over the phase space, and the m(ll) > 150 tail is the one slice where it is
  absent. **Mechanism:** MadEvent combines its VEGAS iterations with weights
  `mean²/σ²` (`Template/LO/Source/dsample.f:1995-1997`, `xxmean = tmean/tsigma`
  at `:2273`, pinned 3.7.1). An iteration that under-samples a peak reads both
  low and precise, so it gets more weight. That is the classic correlated-weight
  bias, and it is low.
- **Evidence, three independent lines:**
  1. A non-adaptive estimator, using MadGraph's own standalone |M|², agrees with
     vibegraph to +0.048 ± 0.028 % and sits above MadEvent.
  2. MadEvent's own per-iteration values, averaged without weights, agree with
     vibegraph and with the estimator. The paired shift from MadEvent's reported
     number is +0.18 % at 4.1σ (60 seeds), replicated at 4.6σ on 36 more.
  3. The shift shrinks with MadEvent's budget and is zero in the tail slice,
     which is what an iteration-weighting bias predicts.
- **The +0.29 % in the brief was overstated.** With 60 fresh directories the
  offset is +0.17 %. The earlier MadEvent mean (six runs, two not fresh) read low.
- **The tail localization was an artefact.** "m_ll > 150 +14 %" and similar
  were read off MadEvent's *unweighted event sample*, which under-populates
  m(ll) > 150 by 5.4 % relative to MadEvent's own sliced σ (12σ). Sliced σ shows
  no tail surplus.
- **What would falsify it:**
  - an unbiased estimator agreeing with MadEvent's reported value rather than
    vibegraph, for example a MadEvent run whose iterations are re-combined with
    equal weights *inside* `dsample.f` and still read 0.1 % low;
  - a non-adaptive estimate with a different phase-space parametrisation
    disagreeing with this one by more than its error;
  - the reported-minus-unweighted shift failing to shrink as points per
    iteration grow.

## Setup (both sides, one card)

- **Process card:** `import model sm` / `generate u u~ > e+ e- g g`, SM
  `restrict_default`.
- **Run card:** `validation/madgraph/pp_to_ll_0j2j_mlm_run_card.dat` with
  `lpp1 = lpp2 = 0`, `ebeam = 250`, `fixed_ren_scale = fixed_fac_scale = True`
  (91.188), `ickkw = 0`, `xqcut = 0`, `mmjj = 20` and `use_syst = False`.
  - That leaves `ptj = 20`, `etaj = 5`, `ptl = 10`, `etal = 2.5`, `drll = 0.4`,
    `drjj = drjl = 0`, `mmll = 50` and `sde_strategy = 1`, confirmed from the
    MadEvent banner.
  - Slices add `mmll` / `mmllmax` / `mmjj` / `mmjjmax` to this card, the same
    file on both sides.
- **Parameter provenance:**
  - Both LHE headers carry `SCALUP 91.188`, `AQEDUP 7.546771e-3` and
    `AQCDUP 0.118`.
  - |M|² was compared point by point on 2000 points from the estimator below:
    MadGraph standalone `SMATRIX` against vibegraph `BoundAmplitude::eval_m2`
    × `initial_spin_color_average` × `outgoing_symmetry_factor`. The ratio is
    1 within 3e-14 (min 5183.99999999983/5184, max 5184.00000000029/5184), on
    points spanning the Z peak, the tails, and soft and collinear gluons.
    Parameters, widths and conventions are identical. The probe was a scratch
    example and is not committed.
- **Cut acceptance, per event:** the run-card cuts were re-applied to every
  event of 300k vibegraph events (`IDWTUP = -4`, seed-1 grid) and of 190k
  MadEvent events. No event on either side fails a cut.
  - Minima seen: ptj 20.0001 / 20.0001, mll 50.0002 / 50.047, mgg 20.0002 /
    20.004, drll 0.417 / 0.434.
  - Blind spot: acceptance in regions a sampler never visits, which the sliced
    σ and the estimator below cover.

## The non-adaptive estimator

- **What it is:** Fortran driver `bwps5.f` (scratchpad) linked against the
  pinned MadGraph's *standalone* `matrix.o`, with RANMAR seeded per run. It
  uses a fixed density, never adapted:
  - m(ll)² from a fixed 50/50 mixture of a Z Breit–Wigner (MZ = 91.188,
    ΓZ = 2.441404) and 1/Q², on [50², 480²];
  - the (Q, g, g) system from massive RAMBO;
  - Q → e⁺e⁻ isotropic in its rest frame;
  - weight `|M|²·conv/(2s) · W_RAMBO(2π)⁻⁵ · 1/(2π g(Q²)) · 1/(8π)`;
  - its own implementation of the cuts from the card.

  A flat-RAMBO variant (`flatps.f`) was run as a cross-check.
- **What its error covers:** pure Monte-Carlo variance of a fixed density. It
  needs no adaptation, so there is no correlation between estimate and weight.
  The error used is the spread over 46 independent seeds of 1.3e7 points each
  (5.98e8 points), seed χ²/dof 1.10.
- **What it does not cover:**
  - the cuts are re-implemented by hand, so a misreading shared with both
    generators would go unseen;
  - the weight distribution is heavy-tailed (single weights up to ~1400× the
    mean, from beam-collinear gluons at ptj ≈ 20 and m(gg) near 20). This is
    why the seed spread, not the per-seed quote, is used. Early 10-seed batches
    showed χ²/dof up to 2.8.
  - It shares the |M|² with both sides by construction. That is the point:
    with |M|² identical, it tests only the integration.

## Results

Units pb. Each row is the unweighted mean over seeds, with error
max(√Σσᵢ²/n, sd/√n) (`tests/common/seeds.rs` rule) and χ²/dof about the mean.

- **ME** is MadEvent, 3.7.1, one fresh directory per seed.
  - "default" is nevents = 10000.
  - "n100k" is nevents = 100000.
  - "hard2" is `hard_survey = 2` + nevents = 100000: survey points ×4,
    iterations +3, accuracy /4.
- **ME-unw** is the same MadEvent runs, each channel re-combined as the equal-
  weight mean of the iteration rows in its `G*/results.dat`, then summed over
  channels. The error is the seed spread.
- **VG** is vibegraph, `--fixed-budget --neval 2000000 --niter 8` (the item's
  budget), `--map-split-angle auto`.
- **Probe** is the estimator above.

| region | ME default | ME n100k | ME-unw (n100k) | VG 2M×8 | probe |
|---|---|---|---|---|---|
| inclusive | 0.527760 ± 0.000211 (60, χ² 0.67) | 0.527975 ± 0.000155 (10, 0.47) | 0.528511 ± 0.000145 | 0.528663 ± 0.000071 (10, 0.53) | 0.528408 ± 0.000130 (46, 1.10) |
| inclusive, hard2 | — | 0.527915 ± 0.000158 (10, 0.99) | 0.528374 ± 0.000172 | — | — |
| m(ll) > 150 | 0.041156 ± 0.000039 (10, 0.44) | — | 0.041157 ± 0.000047 (default) | 0.041196 ± 0.000007 (6, 0.58) | 0.041225 ± 0.000018 (36, 1.06) |
| m(ll) < 150 | 0.486639 ± 0.000515 (10, 0.53) | 0.486698 ± 0.000163 (9, 0.69) | 0.487325 ± 0.000237 | 0.487399 ± 0.000087 (6, 0.93) | 0.487279 ± 0.000142 (36, 1.04) |
| 80 < m(ll) < 100 | — | 0.433557 ± 0.000133 (10, 0.91) | 0.433879 ± 0.000144 | 0.433926 ± 0.000075 (6, 0.99) | 0.433773 ± 0.000136 (36, 0.88) |
| m(gg) > 150 | 0.243576 ± 0.000256 (10, 0.34) | 0.243502 ± 0.000081 (10, 0.45) | 0.243699 ± 0.000100 | 0.243904 ± 0.000040 (6, 0.43) | 0.243822 ± 0.000093 (36, 1.57) |
| m(gg) < 150 | 0.284538 ± 0.000254 (10, 0.41) | — | 0.285022 ± 0.000191 (default) | 0.284741 ± 0.000065 (6, 0.71) | 0.284682 ± 0.000120 (36, 0.89) |
| 20 < m(gg) < 30 | — | 0.042929 ± 0.000012 (10, 1.18) | 0.043006 ± 0.000015 | 0.043002 ± 0.000009 (6, 1.03) | 0.043043 ± 0.000071 (36, 0.95) |
| 30 < m(gg) < 50 | — | — | — | 0.058539 ± 0.000022 (6, **2.96**) | 0.058466 ± 0.000059 |
| 50 < m(gg) < 100 | — | — | — | 0.103390 ± 0.000049 (6, **2.65**) | 0.103267 ± 0.000062 |
| 100 < m(gg) < 150 | — | — | — | 0.079877 ± 0.000024 (6, 0.90) | 0.079906 ± 0.000051 |

Seeds:
- MadEvent default: 1001–1060.
- MadEvent slices: 2001–2010.
- n100k: 3001–3010; hard2: 4001–4010; m(ll) < 150 n100k: 5001–5010; peak:
  6001–6010; 20–30: 7001–7010; m(gg) > 150 n100k: 8001–8010.
- vibegraph: inclusive 1–10, slices 101–106.
- The probe's RANMAR pairs are `(100+37s, 200+53s)`, `(500+41s, 700+59s)` and
  `(1000+43s, 3000+61s)`.

Two MadEvent seeds (1013 first attempt, 5006) of 204 runs failed before integrating, on a
parallel-make race: `libgammaUPC.a: archive has no index`. 1013 was re-run;
5006 is absent. Neither failure depends on the result.

**Closure.** The slices sum to the inclusive value on each side:
- MadEvent: m(ll) slices 0.527795 ± 0.000516 and m(gg) slices
  0.528114 ± 0.000361, against 0.527760 ± 0.000211.
- vibegraph: m(ll) 0.528595 ± 0.000087 and m(gg) 0.528645 ± 0.000076, against
  0.528663 ± 0.000071. Its four m(gg) < 150 sub-slices sum to 0.284808 against
  0.284741 ± 0.000065.

**vibegraph against the probe, by disjoint region:**

| region | pull |
|---|---|
| inclusive | +1.7σ (+0.048 %) |
| m(ll) > 150 | −1.5σ |
| m(ll) < 150 | +0.7σ |
| m(gg) > 150 | +0.8σ |
| m(gg) < 150 | +0.4σ |
| 20 < m(gg) < 30 | −0.6σ |
| 30 < m(gg) < 50 | +1.2σ |
| 50 < m(gg) < 100 | +1.6σ |
| 100 < m(gg) < 150 | −0.5σ |

There is no coherent pattern.

**vibegraph's internal invariances** (seeds 201–206; 8M × 8 on 301–305):
- At 2M × 8, the split-angle maps agree:

  | map | σ (pb) |
  |---|---|
  | isotropic | 0.528681 ± 0.000096 |
  | windowed | 0.528710 ± 0.000103 |
  | soft-all | 0.528767 ± 0.000092 |
  | auto | 0.528663 ± 0.000071 |

- The budget ladder 2M × 8 → 8M × 8 reads 0.528643 ± 0.000049 (5 seeds,
  χ²/dof 0.35). There is no budget shift (−0.004 ± 0.016 %).

**MadEvent:**
- **Budget.** Default 0.527760 → n100k 0.527975 → hard2 0.527915: +0.04 %,
  0.8σ, so no significant climb.
- **Seed χ²/dof** is 0.67 (60 seeds), 0.47 and 0.99, and 0.34–0.53 on the
  default slices. Its quoted errors run 1.2–1.7× its seed spread.
- **Per channel.** The ten `G*` channels over the first 10 default seeds have
  sd/quoted between 0.54 and 1.17. No channel is over-dispersed, so the bias is
  not a missed region that some seeds find.

**The iteration re-combination.** Paired per seed, ME-unw − ME reported:

| run | shift (pb) | relative | significance |
|---|---|---|---|
| default, 60 seeds | +0.000962 ± 0.000233 | +0.18 % | 4.1σ |
| independent default-budget batch, 36 seeds (`notrunc`, below) | +0.000946 ± 0.000204 | — | 4.6σ |
| n100k | +0.000537 ± 0.000109 | +0.10 % | 4.9σ |
| hard2 | +0.000457 ± 0.000108 | +0.09 % | 4.2σ |
| peak n100k | +0.000321 ± 0.000128 | — | — |
| m(gg) > 150 n100k | +0.000206 ± 0.000065 | — | — |
| 20 < m(gg) < 30 n100k | +0.000077 ± 0.000008 | +0.18 % | 9.6σ |
| m(ll) > 150 | +0.000002 ± 0.000035 | — | none |

- The `mean²/σ²`-weighted mean of a channel's rows reproduces its reported
  value: checked on the banked `ee_to_mumua` survey logs, for example G1
  0.0121895 against reported 0.1219E-01.
- The equal-weight mean is unbiased whatever the adaptation, because each
  iteration's estimate is unbiased given the grid frozen during that iteration.
  It is noisier, which is why the paired difference is the statistic quoted.

**Inclusive pulls against the probe:**

| estimate | pull |
|---|---|
| ME default | −2.6σ (−0.12 %) |
| ME n100k | −2.1σ |
| ME hard2 | −2.4σ |
| ME-unw default | +1.0σ |
| ME-unw n100k | +0.5σ |
| ME-unw hard2 | −0.2σ |
| VG | +1.7σ |

## The tail in MadEvent's event sample

In the default runs, MadEvent's unweighted events put a fraction
0.07374 ± 0.00036 of σ above m(ll) = 150 (540k events, 54 runs; binomial
error). Its own sliced σ ratio is 0.041156/0.527760 = 0.07798. The sample is
therefore 5.4 % short in the tail, about 0.42 % of σ, at 12σ. The cause is in
how events are distributed across regions, not in σ: the tail's σ itself shows
no bias.

vibegraph's weighted sample gives 0.07843 ± 0.00049 (300k events) against its
own sliced 0.07792: consistent.

**Mechanism, partly pinned:**
- MadEvent's final unweighting truncates. Measured with the log level raised in
  a copy of `bin/internal/madevent_interface.py`, it removes 0.13–0.43 % of σ
  (six runs: "truncation 0.13 / 0.33 / 0.33 / 0.25 / 0.43 / 0.33 %").
- With that truncation off (`trunc_error = 0` in the copy, 35 runs, 90k
  events), the fraction is 0.07452 ± 0.00087. That is still 4σ short of 0.078,
  and consistent with the truncated runs.
- **So final-stage truncation is not the cause, or not the whole cause.** The
  deficit is already in the per-channel event files.
- The Fortran unweighting (`unwgt.f:253-257`) keeps over-maximum events with
  weight above one, so the remaining candidate is the per-channel
  normalisation: each channel's events are scaled to its biased reported σ.
  That is not tested.

This is the consequence that matters downstream: **a MadEvent reference for a
sub-region taken as inclusive σ × event fraction under-reads high-weight
regions by several percent**. The m_ll > 150 "+14 %" and ŝ > 600 GeV "+9.5 %"
on `qq_llgg` in the item and note 41 D2 are of that kind. MadEvent's dedicated
m_ll > 150 run "11 % above its inclusive run's tail" is this effect, seen from
MadEvent's side alone.

## Errors in the brief

1. **"ŝ > 600 GeV" has no meaning at fixed beams:** ŝ = (500 GeV)² on every
   event. I sliced m(gg) instead, at 150, plus 20/30/50/100 sub-slices, as the
   second axis.
2. **The +0.29 % premise.**
   - With 60 fresh directories MadEvent reads 0.527760 ± 0.000211; the earlier
     0.52743 ± 0.00040 (six runs) sat 0.8σ low.
   - vibegraph at the item's budget reads 0.528663 ± 0.000071 over ten seeds,
     against the earlier 0.52894 ± 0.00024 over four.
   - The offset is +0.17 % (4.0σ) at MadEvent's default budget and +0.13 %
     (4.0σ) at n100k.
3. **"Surplus in the high-ŝ tail".** As sliced σ, the tail agrees (m(ll) > 150:
   VG −0.07 %, ME −0.17 % against the probe, both under 2σ). The surplus
   localisation came from the event-sample artefact above.

## For downstream sessions

- **Does the mechanism predict the other ours-high offsets?**
  - The iteration-weighting bias is low by construction. It grows with the
    per-iteration relative variance and skew of a channel's weights, which
    means narrow peaks a channel does not map, radiative return, or vetoed
    on-shell windows.
  - Here it is 0.1–0.2 %. It explains +1–2 % only if those processes' channels
    are about 10× worse conditioned. That is plausible for `ee_to_mumua`
    radiative return; for the `ttx` MLM @1 and `uu_tt` veto rows it is
    untested.
  - The event-sample mechanism predicts an ours-high reading of several
    percent wherever a reference was derived from MadEvent event fractions
    rather than a dedicated σ run.
- **Hadronic 2 → 3: `g g > t t~ g` (PR #23).** Ours-high there is +0.23 % at
  fixed scale and +0.41 % at dynamical −1, with matched residuals of gg
  +0.57 % and gq +0.87 %.
  - The mechanism produces offsets of the first size naturally. Here it is
    0.18 % at MadEvent's default budget on a fixed-beam 2 → 4 with eight
    integration dimensions.
  - A hadronic run adds x₁, x₂ and PDF/scale weight variation to every
    channel. That raises the per-iteration relative variance, and with it the
    `mean²/σ²` bias.
  - A dynamical scale adds further weight spread, which fits the larger
    offset at −1.
  - The matched +0.57 / +0.87 % needs a channel conditioning roughly 3–5×
    worse than here. That is not excluded, since matching rejections make the
    weights spikier, but it is not shown.
  - **Decisive and nearly free:** if #23's fresh MadEvent directories still
    hold `SubProcesses/P*/G*/results.dat`, compute the paired per-seed shift
    (equal-weight iteration mean summed over channels, minus reported) with
    `iter_seed.py`. A shift equal to the offset attributes it to this
    mechanism; a shift well below it leaves the residual to something else,
    for example large-x coverage.
- **Cheap test, no new MadEvent run needed.** The banked process directories
  keep `SubProcesses/P*/G*/run_01_log.txt` (survey) and, where present,
  `G*/results.dat` rows. Re-combine each channel's "Iteration N Mean:" values
  with equal weights and compare with "Results Last … Integral". For
  `ee_to_mumua` (one survey only) the five channels sum to 0.09791 equal-
  weighted against 0.09725 reported, +0.68 %. That is suggestive but one run
  and survey only.
- **The decisive version** is a few fresh-directory MadEvent runs per row,
  keeping `G*/results.dat`, with the paired shift as in this report, plus a
  fixed-density estimator on MadGraph standalone |M|². Templates for both are
  in the scratchpad: `me_run.sh`, `iter_seed.py`, `bwps.f`.

## Found

1. **The MadEvent σ references are biased low by the iteration weighting in
   `dsample.f`.**
   - Size: 0.1–0.2 % on this process, shrinking with the survey and refine
     budget. On rows gated tighter than about 0.3 % it is a systematic, not
     noise.
   - Evidence: the paired shift above (4–10σ per configuration) and the
     independent estimator.
   - Closing it needs one of three things: references re-derived with equal-
     weight iteration means (a banking request, below); a documented allowance
     per row; or MadEvent driven with more points per iteration until the
     paired shift is below the gate's resolution.
   - It is a MadGraph property and must not be "fixed" on vibegraph's side.
     It belongs with the entries in `validation/madgraph-defects`.
2. **MadEvent's unweighted event samples under-populate high-weight regions,
   beyond the final truncation.**
   - Size: −5.4 % in the m(ll) > 150 tail here (12σ), with 0.13–0.43 % of σ
     removed by the final unweighting alone.
   - Every comparison that reads a MadEvent sub-region σ as inclusive σ ×
     event fraction is exposed, including the per-event shape checks used to
     localize offsets.
   - Closing it needs a MadEvent run with per-channel events kept (patch
     `combine_events` to keep `G*/events.lhe`), to split the deficit between
     the per-channel normalisation and the event generation.
3. **vibegraph's quoted error is too small on two m(gg) slices.**
   - 30 < m(gg) < 50 and 50 < m(gg) < 100 have seed χ²/dof 2.96 and 2.65 over
     six seeds; per-seed quotes are 1.6–1.7× smaller than the seed spread. The
     inclusive and other slices are at 0.4–1.0.
   - The seed-combination rule absorbs it.
   - Closing it needs 20 seeds on those two slices, to tell a true
     under-quote from six-seed scatter.
4. **A MadEvent parallel-make race is visible in fresh directories.**
   `libgammaUPC.a: archive has no index` failed 2 of 204 runs at `nb_core = 2`
   before integration. It is harmless to σ, but a banking driver should retry
   a compile failure rather than drop the seed.
5. **A stray symlink.** A failed `generate_events` drops a `ME5_debug` symlink
   into the *caller's* working directory, here the worktree root. It was
   removed. Banking drivers that run from the repository root can leave
   untracked links.

## Banking request

None is needed to close the item's question. If the user wants the
`pp_to_ll_0j2j_mlm` `@2` reference and the fixed-beam reproducer corrected for
finding 1:

- **Rows:** the fixed-beam reproducer, with the cards above, as a new
  informational row; and `pp_to_ll_0j2j_mlm` `@2`.
- **Seeds:** 20 fresh directories each, `nb_core` free, nevents 100000.
- **Keep** `SubProcesses/P*/G*/results.dat` from every run; today the bundle
  keeps only `run_01_log.txt` for some rows.
- **Record** per seed: the reported σ, and the equal-weight per-channel
  iteration sum computed by the `iter_seed.py` logic (sum over `G*` of the
  mean of the 6-column rows of `results.dat`).
- **Expected outcome:** the reproducer's equal-weight sum lands at
  0.5284–0.5287 pb and the reported value about 0.10 % lower.
