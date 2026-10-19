---
type: Session Report
title: "V0 report: at the tip, neither guard holds its default back"
description: "The jj flavour χ² and the ℓℓj_dyn scatter guard measured at the branch tip under both maps, with the null of each statistic: both flipped defaults pass today's statistics; the brief's failing readings were taken before later changes to the code and do not reproduce."
status: draft
tags: [maps, tau-rule, soft-all, samples-gate, scatter-guard, calibration, report]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-17}
sources:
  - {id: b23e246, resource: "https://github.com/nsmith-/vibegraph/commit/b23e246", title: "test: calibration probes for the jj flavour chi2 and the llj_dyn scatter guard"}
---
Measurement only. No gate, limit, floor or default changed. The committed code
is `#[ignore]`d probes, plus one refactor in `validate_hadronic.rs`:
`run_seed_shaped` now builds its integrand through `adapted_integrand(…,
MapOptions::default())`, the same `ProtonIntegrand::build(…, MapOptions::default())`
that `ProtonIntegrand::new` calls, so the gate's code path does not change. The
gate run below reproduces the probe's first five seeds to the printed digit.

All runs were at tip `51b643a` plus the probes, on the 4-core cloud host
alongside a heavy sibling job, with `CARGO_INCREMENTAL=0
CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0` and the
`release-debug --features extended-validation` build. Logs are in the session
scratchpad under the `mapguards-V0-` prefix; the analysis scripts are
`mapguards-V0-jj.py` and `mapguards-V0-scatter.py` there.

## The brief's premise does not reproduce at the tip

The brief, both backlog items and note 37 §6.3 quote readings taken on the
M3 host off `main@02e8b25` (2026-09-23). Thirty-four commits have landed since,
including the per-group dynamic-scale fix that the manifest says moved
`pp_to_llj_dyn` by −0.176%. Re-measured on today's code:

| claim in the brief | at the tip | command |
|---|---|---|
| jj under τ: one seed p 2.8e-5 (128.1/70), three-seed sum 325/212 | gate with the τ rule flipped in `resolve` (uncommitted, reverted): flavour **75.5/66 p 0.198, 87.5/68 p 0.0558, 80.0/66 p 0.115**; min p over all columns **5.58e-2**; **passes** | `mapguards-V0-jj-gate-tauflip.log`: `cargo test -p vibegraph --profile release-debug --features extended-validation --test validate_samples_proton -- --nocapture --exact generated_dijet_events_agree_with_madgraphs_banked_ones`, with `tau: self.tau.unwrap_or(if shape.whole_state_resonance { Log } else { InverseSquare })` |
| jj under log: three-seed sum 267/210 | 76.4/65, 65.1/65, 92.2/67 → 233.8/197, min p 2.23e-2 | `probe_jj_flavour_chi2_calibration_log` |
| llj_dyn under soft-all: gate quintet 4.17 | **1.96** | `probe_llj_dyn_scatter_guard_calibration_soft_all` |
| llj_dyn under the default: gate quintet 2.17 | **2.25**. The gate itself reads 2.25 on the same five values (415.103, 416.080, 415.172, 414.579, 416.636 pb) | `sigma_llj_dynamical_scale_vs_mg` (`mapguards-V0-llj-gate.log`) against `probe_llj_dyn_scatter_guard_calibration` |
| seed 20260732 ≈ 3.6σ low under every map | −0.57σ under the default (415.172 ± 0.583), −2.06σ under soft-all (414.409 ± 0.543), relative to the 80-seed means | the two llj probes |

The flipped-τ gate run matches `probe_jj_flavour_chi2_calibration_inverse_square`'s
gate-grid seeds to the printed digit, including the ICOLUP cells (13.8/23,
14.8/22, 24.3/23). So the probe's explicit `--map-tau inverse-square` builds the
same thing a flipped default would. This is expected by construction:
`resolve` takes the named choice first.

**At the tip, both defaults can flip with their cells passing under the
statistics as they stand today.** The tables below are the evidence for D1
either way.

## Part A — `pp_to_jj` flavour composition

Configuration: the gate's own (`NEVAL 300000 × 8`, 20 000 events per
generation seed, against MadGraph's banked 10 000 in `pp_to_jj/run_01`). The
gate grid is integration seed `20260731` with generation seeds
`0x5A4D1001–1005`; the first three are the gate's own. The second grid is
`20260741` with its own seeds `0x5A4D2001–2005`. Commands:
`cargo test -p vibegraph --profile release-debug --features extended-validation
--test validate_samples_proton -- --ignored --nocapture --exact <probe>`, where
`<probe>` is one of `probe_jj_flavour_chi2_calibration_log`,
`probe_jj_flavour_chi2_calibration_inverse_square` or
`probe_jj_flavour_chi2_mg_half_splits`. The log probe took 473 s, the τ probe
401 s and the MadGraph-split probe 358 s.

### A.1 and A.3 — against MadGraph, per seed and pooled

Flavour χ² / dof (p):

| map, grid | seed 1 | seed 2 | seed 3 | seed 4 | seed 5 | pooled 3 (60k) | pooled 5 (100k) |
|---|---|---|---|---|---|---|---|
| log, 20260731 (gate) | 76.4/65 (0.157) | 65.1/65 (0.473) | 92.2/67 (0.0223) | 79.7/67 (0.137) | 80.2/66 (0.112) | 106.3/83 (0.0435) | 127.4/85 (**2.01e-3**) |
| log, 20260741 | 83.5/63 (0.0428) | 78.7/69 (0.199) | 94.6/66 (0.0120) | 74.5/67 (0.248) | 72.3/67 (0.309) | 116.8/81 (5.64e-3) | 117.0/86 (0.0147) |
| τ, 20260731 (gate) | 75.5/66 (0.198) | 87.5/68 (0.0558) | 80.0/66 (0.115) | 92.8/67 (0.0201) | 72.9/66 (0.262) | 109.8/83 (0.0262) | 117.2/85 (0.0119) |
| τ, 20260741 | 64.4/66 (0.532) | 84.6/62 (0.0299) | 85.6/65 (0.0443) | 83.0/64 (0.0558) | 75.6/67 (0.221) | 107.4/83 (0.0371) | 112.7/87 (0.0332) |

The gate's statistic is the minimum p over every column and seed. Over three
seeds it reads 2.23e-2 (log, gate grid), 1.20e-2 (log, second grid), 5.58e-2
(τ, gate grid) and 2.99e-2 (τ, second grid). Over five seeds it reads 2.23e-2,
1.20e-2, 2.01e-2 and 2.99e-2. The flavour column is the minimum in every case.
The ICOLUP, SPINUP and KS columns all sit above 0.05 against MadGraph.
Five-seed sums are 393.7/330, 403.6/332, 408.8/333 and 393.1/324.

### A.2 — the nulls

**(a) This side against itself**, at the gate's sizes. Each grid's five
20 000-event samples, and their pools, go against random 10 000-event halves of
the other grid's samples. That makes 20 references per map, 100 per-seed and
20 pooled readings, and every pair is independent.

| map | statistic | χ²/dof mean (sd; nominal sd) | p-values ~ U (KS p) | frac p < 0.05 |
|---|---|---|---|---|
| log | per seed (20k vs 10k) | 0.983 (0.173; 0.174) | 0.156 | 0.050 |
| log | pooled 3 (60k vs 10k) | 1.010 (0.161; 0.156) | 0.973 | 0.000 |
| log | pooled 5 (100k vs 10k) | 1.017 (0.154; 0.153) | 0.821 | 0.100 |
| τ | per seed | 1.004 (0.149; 0.174) | 0.316 | 0.030 |
| τ | pooled 3 | 0.999 (0.136; 0.155) | 0.466 | 0.050 |
| τ | pooled 5 | 0.990 (0.146; 0.152) | 0.451 | 0.050 |

The gate statistic under this null, as the minimum p over every column and
three seeds against one reference, has a median of 3.9e-2 (log) and 4.3e-2 (τ)
over the 20 references; the smallest reads are 2.4e-3 and 5.5e-3. The
three-seed flavour sum against a *shared* reference has mean 196.7 on 197.4
dof with sd 26.7 (log), and 202.0 on 199.2 with sd 21.0 (τ). Independent χ²
would give sd ≈ 20. This confirms that the brief's sums are not χ² on their
nominal dof, but the effect is in the spread, not the mean.

**(b) MadGraph against MadGraph.** 1 000 random halvings of the banked sample,
5 000 against 5 000 (a split along file order would compare MadGraph's
per-channel concatenation): flavour χ²/dof **0.993** (sd 0.207, nominal 0.218,
42 dof); frac p < 0.05 0.036, p < 0.01 0.008, p < 1e-3 0.001; p ~ U at KS p
0.51. ICOLUP reads 0.999 and SPINUP 0.995.

So **both nulls are nominal**: the two-sample χ² with effective counts is
calibrated at these sizes, per seed and pooled. **Against MadGraph** every
reading sits above it, by the same amount under both maps and both grids. Per
seed, the mean χ²/dof is 1.19, 1.22, 1.23 and 1.21 against a null of
0.98–1.00. Pooled over five seeds it is 1.50, 1.36, 1.38 and 1.30 against
0.99–1.02. That excess is common to the maps, so it is not the map's. All four
readings share one MadGraph sample, which carries 10/11 of the pooled
comparison's variance, so they are close to **one** draw. Read that way, our
composition differs from *this* MadGraph sample at p ≈ 2e-3 – 3e-2. Whether
that is the sample's own fluctuation or a real difference cannot be decided
without a second independent MadGraph sample (banking request below).

**The trial count `1e-4` is designed against** (the `P_FLOOR` doc in
`validate_samples.rs`) is a few hundred correlated draws per run: 46 gating
rows × 3 seeds × 7–21 columns, about 0.1 expected spurious failures per run.
This row contributes 3 seeds × 8 columns (5 KS, 3 χ²), plus its beam and scale
fields.

**What pooling buys.** The non-centrality of a real composition difference
scales with `(1/n_ours + 1/n_ref)⁻¹`: 6 667 per seed, 8 571 pooled-3, 9 091
pooled-5, and at most 10 000 for any generated size. A larger generated sample
or a pooled statistic therefore gains at most 1.5× in sensitivity over today's
per-seed cell. The binding size is MadGraph's 10 000. Pooling replaces three
correlated draws with one, which lowers the trial count, and its null is
measured nominal above.

**Decision table (jj, at the tip, gate grid; the second grid in brackets).**

| candidate | log | τ |
|---|---|---|
| today: min p over 3 seeds ≥ 1e-4 | 2.23e-2 (1.20e-2) pass | 5.58e-2 (2.99e-2) pass |
| min p over 5 seeds ≥ 1e-4 | 2.23e-2 (1.20e-2) pass | 2.01e-2 (2.99e-2) pass |
| pooled 3, one χ² ≥ 1e-4 | 4.35e-2 (5.64e-3) pass | 2.62e-2 (3.71e-2) pass |
| pooled 5, one χ² ≥ 1e-4 | 2.01e-3 (1.47e-2) pass | 1.19e-2 (3.32e-2) pass |

## Part B — `pp_to_llj_dyn` scatter guard

Gate configuration: `LLJ_NEVAL 150 000 × 10`, adapt 8 000 × 5, **80 seeds per
map** (20260730–20260809), the gate's five first. The host allowed it: 72 s and
79 s per seed. Commands: `cargo test -p vibegraph-lib --profile release-debug
--features extended-validation --test validate_hadronic -- --ignored --nocapture
--exact probe_llj_dyn_scatter_guard_calibration` (the default, which draws
bit-identically to isotropic on this process per `resolve`'s doc), 5768 s, and
`… probe_llj_dyn_scatter_guard_calibration_soft_all`, 6306 s. Every seed's σ,
error and six per-channel terms are in the logs as `SCATTER` and `CHANNEL`
lines.

### B.1 — calibration

| map | seeds | mean (pb) | χ²/dof | sd / mean quoted | gate quintet | disjoint quintet χ²/dof | disjoint 20-seed χ²/dof |
|---|---|---|---|---|---|---|---|
| default | 80 | 415.502 | 0.93 | 0.96 (0.131% / 0.136%) | 2.25 | 2.25 1.40 1.24 0.39 1.88 1.07 0.90 1.01 1.03 2.54 0.45 0.38 0.60 0.40 0.50 0.34 | 1.17 1.28 0.96 0.46 |
| soft-all | 80 | 415.528 | 0.94 | 0.97 (0.132% / 0.136%) | 1.96 | 1.96 0.51 0.38 0.11 1.57 0.25 1.01 1.66 1.07 1.66 1.58 0.50 1.97 0.59 0.77 0.59 | 0.72 1.24 1.05 0.85 |

The seed-paired difference soft-all − default is +0.026 ± 0.064 pb, and the
same seed's two σ correlate at ρ = 0.45 across maps (the seed fixes most of
the stream).

### B.2 — false-alarm rates

"Random subsets" means 200 000 random 5- or 20-subsets of the 80 recorded
seeds: a U-statistic of these 80 values, whose uncertainty is set by the few
most extreme seeds. "Gaussian" draws each seed N(0, its quoted error).

| candidate | Gaussian expectation | default: disjoint / random subsets | soft-all: disjoint / random subsets |
|---|---|---|---|
| today: χ²/dof ≤ 4.0 on 5 seeds | 0.30% (0.29% with these errors) | 0/16 / **0.07%** | 0/16 / **0.00%** |
| p ≥ 1%: χ²/dof ≤ 3.32 on 5 seeds | 1.00% (1.05%) | 0/16 / 0.60% | 0/16 / 0.02% |
| 20 pooled seeds, χ²/dof ≤ 1.90 on 19 dof | 1.00% (1.01%) | 0/4 / 0.06% | 0/4 / 0.00% |

**What each would decide.** All three pass both maps' gate seeds: quintet 2.25
and 1.96 against 4.0 and 3.32. The first 20-seed pool reads 1.17 and 0.72
against 1.90. On this estimator the realised rates sit below the Gaussian
ones. The errors are honest to slightly conservative (sd/quoted 0.96–0.97),
and no seed of 160 sits outside ±2.81σ. The p-rule is the stricter of the two
five-seed rules, as the brief notes: 0.60% realised under the default against
0.07%.

### B.3 — is the tail real?

At the tip there is **no seed beyond 3σ** of the 80-seed mean under either map,
against a Gaussian expectation of 0.22. Beyond 2.5σ there are 2 (default) and
0 (soft-all), expectation 0.99. Beyond 2σ there are 4 and 1, expectation 3.64.
The lowest seeds are 20260763 under the default (−2.05σ; −2.06σ against the
other 79) and 20260732 under soft-all (−2.06σ; −2.08σ). Each one's quoted error
is not shrunk (0.538 against a median of 0.562; 0.543 against 0.558), which is
the "confidently wrong" signature that is absent here. The iteration χ²/dof is
1.47 and 0.43.

**Per channel.** The six channel terms move with the seed's adapted α, so a raw
split reads α: channel terms scatter by about 3 pb seed to seed against quoted
errors of about 0.3 pb. Each channel's term is therefore regressed on the seed's
α vector across the 80 seeds. The residual sd per channel is then 0.92–1.16
(default) and 0.95–1.30 (soft-all) of its quoted error. Over 480
channel-seed cells per map, 2 and 1 sit beyond 3 residual sd, against a
Gaussian 1.30. For the low seeds, the per-channel residuals are spread across
the channels: default 20260763 reads +1.34, −1.87, −1.72, −0.59, −0.09 and
−1.17; soft-all 20260732 reads +1.76, −1.37, −1.77, −0.81, −1.34 and −1.27. No
single channel carries the deficit. **Verdict: an honest tail, with no missed
region.** The "low under every map" of the earlier reading is in large part
the seed-paired correlation (ρ = 0.45). What this cannot see: a region every
seed misses, which moves the mean and not the scatter and is the σ gate's
concern, and one of measure too small for 80 seeds to visit.

### B.4 — the fixed-scale reader

`sigma_llj_fixed_scale_vs_mg` reads the same `LLJ_MAX_CHI2_PER_DOF`. The only record
is the manifest's `pp_to_llj_fixed` integrals note: χ²/dof 1.86 on the gate's
five seeds under the default maps. That passes 4.0 and passes 3.32. The 20-seed
pooled reading and every soft-all reading are **unmeasured**.

## Gates on the committed code

- `cargo fmt --all --check`: clean.
- `cargo clippy --workspace --all-targets -- -D warnings`, and the same with
  `--features vibegraph-lib/extended-validation,vibegraph/extended-validation`
  on `-p vibegraph-lib -p vibegraph`: both clean (`mapguards-V0-clippy.log`).
- `cargo test --workspace` (hermetic): 39 test binaries, 1478 passed, 0 failed,
  16 ignored (`mapguards-V0-hermetic.log`). Both edited test files need
  `extended-validation`, so this suite does not compile them. They were built
  and run by the probes and gate runs above.
- `sigma_llj_dynamical_scale_vs_mg` on the refactored helper: pass, χ²/dof
  2.25, rel +0.19% against the seeded MadEvent reference 414.716 ± 0.895 pb.
- `pixi run --skip-deps validate` was not run: no amplitude, eval or coverage
  code changed.

## Banking request (for the user's M3 host)

To separate "our flavour composition differs from MadGraph's" from "this one
MadGraph sample's draw":

- Process `pp_to_jj`, with the banked run's own `Cards/` unchanged except
  `iseed`, set to a value not used before (e.g. 20261017). Use the pinned
  `validation/madgraph/mg5_pinned.sh` (MG 3.7.1).
- Bank `Events/run_02/unweighted_events.lhe.gz` beside `run_01`. 10 000 events
  matches today's size; 50 000 would make the reference no longer the binding
  size.
- Expected readings, both from the probes committed here once a reference path
  is a parameter: MadGraph `run_01` against `run_02` (a genuine two-run MadGraph
  null), and our pooled samples against `run_02`. If pooled-5 against `run_02`
  reads p ≫ 1e-2 while `run_01` stays near 2e-3, the excess is `run_01`'s draw.

## Found

- **One generation seed replayed off two grids gives correlated samples.** A
  generation seed fixes the whole trial stream (channel, point and acceptance
  uniforms), and two grids trained on one card differ only slightly. In this
  session's first run, which reused `0x5A4D1001–1005` on both grids, same-seed
  cross-grid flavour χ²/dof read **0.43** (20 readings) against 0.95 for
  different seeds (80) (`mapguards-V0-jj-log-sameseeds.log`). σ shows the same
  thing: llj_dyn's default and soft-all at the same seed correlate at ρ = 0.45.
  Any arm-against-arm comparison that reuses seeds is paired, not independent.
  Closes when the probes and notes that compare maps or grids seed-for-seed
  (including note 37's "same-map seed controls") are audited for it.
- **The two backlog items' numbers are stale.** At the tip, neither blocker
  reproduces (table at the top). The manager should re-scope D1/V1 against the
  tip: flipping either default today needs no statistic change for these two
  cells. It does still need the banked layer re-run under both flips, because
  the τ rule reaches every hadronic row without a whole-state resonance, llj
  included.
- **Stale doc.** In `validate_samples_proton.rs`, the doc of
  `generated_dijet_events_agree_with_madgraphs_banked_ones` opens with
  "measured and recorded, not enforced, on one column". The row is
  `mode: "gate"`, its body asserts every column, and the same doc later says
  "all enforced".
- **Unconfirmed: the τ grid-against-grid SPINUP null reads high.** χ²/dof is
  1.25 (5 dof, 100 readings, KS p 4e-5, an overstatement because the readings
  share samples) against 1.10 under log. Against MadGraph it is clean. One more
  independent grid pair per map would settle whether the helicity composition
  depends on the grid under τ.
- **A finer reference for the jj composition is already banked.** Every banked
  event carries `IDPRUP = 1`, so the sample cannot be split by subprocess
  directory. But each `SubProcesses/P1_*/G*/run_01_log.txt` prints MadGraph's
  integrated per-channel cross section and error ("Accumulated results:
  Integral = …"; `P1_gg_gg/G1` reads `0.4058E+06`). Our per-directory σ against
  those numbers compares this side's composition with MadGraph's *integration*
  rather than with its one 10 000-event draw. That could settle the A.2 excess
  without new banking. Closes with that comparison recorded.
- **`pp_to_llj_fixed` under soft-all is unmeasured**, and so is its 20-seed
  pooled scatter. It needs the same probe on the fixed-scale run with `ScaleShape::FixedAtMz`.
