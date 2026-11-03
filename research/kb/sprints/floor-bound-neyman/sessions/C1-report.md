---
type: Session Report
title: "C1 report: channel terms are plain means of their iterations; Neyman's bbx gain was the combine, its target-mode gain is small and real"
description: "combine_kept and pooled_scale now take the plain mean of a channel's kept iterations whatever counts they drew. Replayed on P1's traces it removes Neyman's bias (paired t −2.40/−2.77 → −0.35/−0.22) at no variance cost on the α arm; the α-share weights the brief proposed are the plain mean exactly. The fixed-budget path is statistically gated: 18 banked cells and 8 long-tier MLM cells moved, all keep their status, pp_to_ll_0j2j_mlm @2 by +0.18% (pull +1.90 → +2.08). On p p > l+ l- j (6 channels) at a 0.179% target Neyman stops after 1.49M evaluations against ByAlpha's 1.58M over 20 paired seeds, the same before and after the change."
status: draft
tags: [multichannel, allocation, neyman, iteration-combination, bias, budget-ladder, report]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-26}
measured:
  host: "cloud container, 4 vCPU, linux-64, 2026-10-26, one long job at a time"
  commit: "replays on P1's traces (ByAlpha: B1's main tree e5a09a8; Neyman: P1's probe on a79753c); every run on 2d37f4c (tree 570dd4c) before the change and on 2d37f4c plus this session's diff after it (see Method for which diff each run carried)"
  command: "see each section; scripts, TSVs and logs under the session scratchpad, prefixed fbn-C1-"
sources:
  - {id: brief, resource: "C1.md", title: "C1 brief"}
  - {id: p1, resource: "P1-report.md", title: "P1 report"}
  - {id: budget, resource: "../../../../../vibegraph-lib/src/budget.rs", title: "ChannelHistory::combine_kept, pooled_scale, BlockAllocation::Neyman"}
---
## Verdict

- **The combine is now the plain mean of a channel's kept iterations, whatever
  counts they drew.** `ChannelHistory::combine_kept` hands every channel to
  `combine_iterations` (the arithmetic `VegasGrid::adapt` uses), and `pooled_scale`
  forms `quoted = Σσᵢ²/k²` and `emp = Σ(Iᵢ − Ī)²/k² · k/(k−1)` about that same plain
  mean, so the target mode stops on the estimate it quotes. The point count no longer
  enters `ChannelHistory::kept` at all.
- **Why it is outcome-independent.** Every weight is 1. An iteration's count is fixed
  before its points exist (from earlier iterations' sd under Neyman, from earlier
  acceptance under the floor), so each iteration's estimate is unbiased given its
  count, and a mean with weights fixed in advance stays unbiased. A count-weighted
  mean is not, because its normaliser `Σnᵢ` contains counts chosen after the
  iterations it weights.
- **The α-share weights the brief proposed are the plain mean.** `neval` is the same
  every iteration, so a channel's α share is one number for all of its iterations.
  The replay agrees with the plain mean to 3.8e-16 relative.
- **On the replay the plain mean removes Neyman's bias at no variance cost on the
  outcome-independent arm.** Neyman − ByAlpha goes from −1.71e-9 / −2.41e-9 pb
  (t −2.40 / −2.77) to −0.27e-9 / −0.29e-9 (t −0.35 / −0.22). ByAlpha's own sd moves
  2.689e-9 → 2.714e-9 and 3.342e-9 → 3.330e-9. The variance the old combine "saved"
  under Neyman was bought with the bias (step 1).
- **The fixed-budget path is not bit-identical; it is statistically gated.** On bbx,
  ByAlpha drew unequal kept counts on 219–268 of 615 channels per seed (all 20 seeds,
  both rungs), almost all floor-bound channels whose floor was re-read from acceptance.
  `pixi run --skip-deps validate` before and after: 18 enforced cells moved, all
  `pass → pass`. The long-tier MLM gates, which run Neyman at a fixed budget and are
  not in `validate`, were run before and after too: 8 cells moved, all `pass → pass`.
  `pp_to_ll_0j2j_mlm` @2 rose +0.245 pb (+0.18 %, 10 paired seeds, t +1.85), pull
  +1.90 → +2.08 (step 3).
- **Neyman on bbx under the new combine is the plain-mean replay**, checked on seven reruns
  (Neyman 600k ×3 and 1.2M ×2, ByAlpha 600k ×2), which reproduce the replay to every
  printed digit (step 4): 600k mean 1.689739e-6, sd 3.356e-9, χ²/dof 1.51; 1.2M mean
  1.691395e-6, sd 3.960e-9, χ²/dof 1.67; sd ratio 1.180. ByAlpha under the new
  combine: sd 2.714e-9 → 3.330e-9, ratio 1.227.
- **The CLI's Neyman target mode keeps a small, real advantage, and its answer did
  not move.** On `p p > l+ l- j` (now 6 channels) at a 0.179 % target over 20 paired
  seeds: Neyman 1.494M evaluations (1.20M–1.80M), ByAlpha 1.584M (1.32M–2.04M), Neyman
  fewer on 15 of 20 (paired t +2.26). The new combine leaves every Neyman seed at the
  same iteration count and moves σ by +0.006 pb on average (sd 0.034 pb). ByAlpha is
  bit-identical, since no floor binds there. `p p > e+ e-` integrates one channel, and
  the two rules give the same run. The documented 2.94M against 6.41M (24 channels,
  older stop rule) no longer describes the code; both docs carry the new figures. No
  default changes (step 6).

## Step 1: offline replay of candidate combines

Inputs: P1's traces, read in place. `t26-B1-main-trace.txt` is ByAlpha (B1's
main-equivalent tree), and `fbn-P1-neyman-trace.txt` is Neyman. Each has 20 seeds × 2
rungs × 615 channels × 8 iterations, with warm-up 2, so 6 kept iterations per channel.
Each run's σ is the sum of its channel terms, and its error is their quadrature sum.
The candidates are the per-channel weights wᵢ over a channel's kept iterations:

| candidate | wᵢ | outcome-independent? |
|---|---|---|
| `prod` | the old code: 1 if the counts are equal, else nᵢ | no: nᵢ is set from earlier iterations' sd (Neyman) or acceptance (floor) |
| `plain` | 1 | yes: fixed before the run |
| `share` | `round(αⱼ · neval)`, the α share after the survey | yes, since the survey precedes every draw it weights. It is constant per channel, so it equals `plain` |
| `afloor` | `max(share, floorᵢ)`, ByAlpha's count, its floor re-read from acceptance | no: the floor follows the acceptance that earlier iterations measured |
| `index` | i (1…6), a schedule that favours later grids | yes: fixed before the run |

```
$ python3 -I fbn-C1-scripts/replay.py t26-B1-main-trace.txt fbn-P1-neyman-trace.txt   (output: fbn-C1-replay.txt)
```

Per arm, rung and candidate, over 20 seeds (pb). "pull" is against MadEvent's 20-seed
reported reference (1.686455e-6 ± 1.31e-9) with the measured spread/√n as this side's
error, as in P1:

| arm | candidate | rung | mean | per-seed sd | RMS quoted | sd / RMS quoted | χ²/dof | pull | sd ratio 1.2M/600k |
|---|---|---|---|---|---|--:|--:|--:|--:|
| ByAlpha | prod | 600k | 1.689996e-6 | 2.689e-9 | 3.103e-9 | 0.867 | 0.79 | +2.46 | |
| ByAlpha | prod | 1.2M | 1.691693e-6 | 3.342e-9 | 3.665e-9 | 0.912 | 1.44 | +3.47 | 1.243 |
| ByAlpha | plain = share | 600k | 1.690011e-6 | 2.714e-9 | 3.109e-9 | 0.873 | 0.80 | +2.46 | |
| ByAlpha | plain = share | 1.2M | 1.691689e-6 | 3.330e-9 | 3.662e-9 | 0.909 | 1.41 | +3.47 | 1.227 |
| ByAlpha | afloor (= prod) | 600k / 1.2M | as prod | | | | | | 1.243 |
| ByAlpha | index | 600k | 1.690222e-6 | 3.308e-9 | 3.669e-9 | 0.902 | 0.94 | +2.50 | |
| ByAlpha | index | 1.2M | 1.691849e-6 | 3.068e-9 | 3.447e-9 | 0.890 | 1.17 | +3.65 | 0.928 |
| Neyman | prod | 600k | 1.688286e-6 | 2.712e-9 | 2.552e-9 | 1.063 | 1.14 | +1.27 | |
| Neyman | prod | 1.2M | 1.689281e-6 | 1.758e-9 | 2.114e-9 | 0.832 | 0.76 | +2.07 | 0.648 |
| Neyman | plain = share | 600k | 1.689739e-6 | 3.356e-9 | 3.002e-9 | 1.118 | 1.51 | +2.17 | |
| Neyman | plain = share | 1.2M | 1.691395e-6 | 3.960e-9 | 3.995e-9 | 0.991 | 1.67 | +3.12 | 1.180 |
| Neyman | afloor | 600k | 1.689721e-6 | 3.346e-9 | 2.988e-9 | 1.120 | 1.50 | +2.16 | |
| Neyman | afloor | 1.2M | 1.691413e-6 | 3.958e-9 | 4.002e-9 | 0.989 | 1.68 | +3.14 | 1.183 |
| Neyman | index | 600k | 1.689781e-6 | 3.791e-9 | 3.479e-9 | 1.090 | 1.40 | +2.13 | |
| Neyman | index | 1.2M | 1.691134e-6 | 3.804e-9 | 4.276e-9 | 0.890 | 1.15 | +3.00 | 1.003 |

Paired Neyman − ByAlpha per seed, both arms under the same candidate (20 seeds, pb):

| candidate | 600k | 1.2M |
|---|---|---|
| prod | −1.711e-9, sd 3.181e-9, **t −2.40** | −2.412e-9, sd 3.897e-9, **t −2.77** |
| plain = share | −2.723e-10, sd 3.465e-9, t −0.35 | −2.941e-10, sd 5.872e-9, t −0.22 |
| afloor | −2.756e-10, sd 3.470e-9, t −0.36 | −2.796e-10, sd 5.863e-9, t −0.21 |
| index | −4.412e-10, sd 3.739e-9, t −0.53 | −7.152e-10, sd 5.711e-9, t −0.56 |

```
max |candidate − plain| / σ over the 20 seeds:   share 2.5e-16 (600k), 3.8e-16 (1.2M) on both arms
                                                  afloor on ByAlpha = prod: 8.2e-5, 1.5e-4
```

Reading:
- Every candidate other than `prod` removes the shift. `afloor` does so as well as
  `plain` does, but its weights move with acceptance, the outcome dependence the brief
  rules out, and it buys nothing over `plain` (sd 3.346e-9 / 3.958e-9 against
  3.356e-9 / 3.960e-9).
- `index` is outcome-independent, but its variance is worse than `plain` at 600k on
  both arms (ByAlpha 3.308e-9 against 2.714e-9) and better at 1.2M (3.068e-9 against
  3.330e-9). It is an iteration-age bet, not a fix, and it is not the rule
  `VegasGrid::adapt` combines its own iterations by.
- **The variance cost of `plain` is nil where the variance is honest.** On ByAlpha,
  whose counts move only through the floor re-read, sd goes 2.689e-9 → 2.714e-9
  (+0.9 %) and 3.342e-9 → 3.330e-9 (−0.4 %). The quoted error tracks the spread as
  before (sd / RMS quoted 0.87 / 0.91). Under Neyman the plain mean's sd is larger than
  `prod`'s, but `prod`'s smaller spread came with its bias: per seed at 1.2M the
  biased combine's sd is 1.76e-9, while its RMS error against the ByAlpha arm is
  √(1.76² + 2.41²)e-9 = 2.98e-9 against a quoted 2.11e-9. A bias does not average
  away over seeds.
- The plain mean is therefore chosen. It is also the one the stop rule can mirror
  exactly, and it is what equal counts already took.

## Step 2: the change

`vibegraph-lib/src/budget.rs`:
- `ChannelHistory::kept` holds `(integral, variance)`; the point count is not stored.
- `combine_kept` maps the variances through the `f64::MIN_POSITIVE` floor and calls
  `combine_iterations(…, rule)` for every channel. The count-weighted branch is gone.
- `pooled_scale` reads the plain mean (`quoted = Σσᵢ²/k²`, `emp = Σ(Iᵢ−Ī)²/k² ·
  k/(k−1)`).
- `sequential_channel_reference` (the bit-for-bit reference that `hadronic.rs` and
  `proton.rs` check the parallel path against) takes the plain mean too.
- Docs: the module's floor and stop-rule paragraphs, `kept`, `combine` (which claimed
  the counts cannot correlate with the estimates), `pooled_scale`, and
  `BlockAllocation::Neyman`'s target-mode figures. `vibegraph-cli/src/integrate.rs`
  gets the `--allocate` help text, and `docs/src/cli/reference.md` is regenerated
  from the binary (`scripts/gen-cli-docs.sh`).

Tests:
- `counts_set_from_earlier_variances_do_not_bias_the_term` (new) is the brief's toy.
  A channel of Pareto points (tail index 2.2, mean 1) draws each iteration's count ∝
  the per-point sd its iterations have measured so far (`point_sd()`, as Neyman reads
  it), clamped to 16–1024. It runs 4000 seeds of 8 iterations with warm-up 2. The test
  asserts that the count-weighted mean of the same iterations is below −5 standard
  errors (the toy reproduces the mechanism) and that the reported term is within ±4.
- `unequal_iterations_combine_as_their_plain_mean` replaces
  `unequal_iterations_are_weighted_by_their_point_counts`. It records iterations of
  100 and 300 points and pins the integral at 1.5, the quoted variance at `Σσ²/4`,
  and the stop scale at the plain mean's emp/quoted.

On the old code (2d37f4c with only the new test added):

```
$ pixi run cargo test -p vibegraph-lib --profile release-debug --features extended-validation --lib counts_set_from_earlier_variances
the channel term is -6.6 standard errors from the truth (the count-weighted mean: -6.6)
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1092 filtered out; finished in 0.23s
```

On the new code the same test passes. With its tolerance forced to 0 for one run,
to print the figure:

```
the channel term is +1.9 standard errors from the truth (the count-weighted mean: -6.6)
$ pixi run cargo test -p vibegraph-lib --profile release-debug --features extended-validation --lib budget::
test result: ok. 35 passed; 0 failed; 0 ignored; 0 measured; 1058 filtered out; finished in 0.19s
```

## Step 3: transformation class

**Statistical.** ByAlpha takes the old count-weighted branch wherever the floor re-read
changes a floor-bound channel's count between kept iterations:

```
$ python3 -I fbn-C1-scripts/nonuniform.py t26-B1-main-trace.txt ByAlpha
ByAlpha 600000: 20 seeds, 12300 channel-runs; non-uniform kept counts per seed mean 247.7 (range 219-268), of which every kept count at its floor: mean 245.3; seeds with >=1: 20/20; sigma(plain)-sigma(prod) per seed mean +1.459e-11 sd 5.022e-11 max|.| 1.391e-10 pb
ByAlpha 1200000: 20 seeds, 12300 channel-runs; non-uniform kept counts per seed mean 242.1 (range 215-259), of which every kept count at its floor: mean 240.9; seeds with >=1: 20/20; sigma(plain)-sigma(prod) per seed mean -4.019e-12 sd 6.466e-11 max|.| 2.509e-10 pb
$ python3 -I fbn-C1-scripts/nonuniform.py fbn-P1-neyman-trace.txt Neyman
Neyman 600000: ... non-uniform kept counts per seed mean 276.2 (range 240-293) ...; sigma(plain)-sigma(prod) per seed mean +1.453e-09 sd 1.177e-09 max|.| 4.080e-09 pb
Neyman 1200000: ... non-uniform kept counts per seed mean 280.6 (range 247-303) ...; sigma(plain)-sigma(prod) per seed mean +2.113e-09 sd 2.655e-09 max|.| 1.028e-08 pb
```

On bbx the ByAlpha move is 1.5e-11 ± 5e-11 pb per seed, at most 1.5e-4 of σ, with no
common sign. Under Neyman it is +1.5e-9 / +2.1e-9 pb, the bias. Wherever no count
varies (equal kept counts, a single channel, or ByAlpha with no binding floor) the
arithmetic is the old uniform branch's, so the output is bit-identical. The target-mode
ByAlpha and `p p > e+ e-` arms of step 5 show this.

**Banked layer before and after**, run detached, one at a time:

```
before: 2d37f4c, tree 570dd4c, 2026-10-26T14:39:53Z – 15:53:30Z
after:  2d37f4c + the change (budget.rs as committed except the BlockAllocation::Neyman doc paragraph; integrate.rs untouched), 16:14:36Z – 17:24:02Z
$ CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 pixi run --skip-deps validate
before: 70 rows × 4 categories = 280 cells: 211 measured in the layers this run drove (206 ✅, 5 ⚠️, 13 ⏳, 56 — / uncovered). ... exit 0
after:  70 rows × 4 categories = 280 cells: 211 measured in the layers this run drove (206 ✅, 5 ⚠️, 13 ⏳, 56 — / uncovered). ... exit 0
test results summed: before 2032 passed, 0 failed, 98 ignored; after 2033 passed, 0 failed, 98 ignored
$ python3 -I fbn-C1-scripts/cells.py fbn-C1-report-before fbn-C1-report-after   (every cell file, durations and host stripped)
cells identical 199, moved 18
```

Every moved cell (all enforced, all `pass → pass`):

| cell | seeds moved | largest per-seed \|Δσ/σ\| | pull before → after |
|---|---|---|---|
| integrals/ee_to_ee | 5/5 | 1.5e-8 | +0.5799 → +0.5799 |
| integrals/ee_to_ee_nsz | 10/10 | 7.2e-9 | −0.1957 → −0.1957 |
| integrals/ee_to_ee_nsz (unrestricted) | 10/10 | 2.2e-8 | +1.8037 → +1.8037 |
| integrals/ee_to_mumu_tata_qcd0 | 5/5 | 2.2e-5 | +1.055 → +1.065 |
| integrals/ee_to_tatah | 5/5 | 4.5e-7 | +0.1102 → +0.1103 |
| integrals/gg_to_gg | 5/5 | 2.6e-10 | +1.7642 → +1.7642 |
| integrals/gg_to_gg_cg | 5/5 | 6.7e-9 | +0.6784 → +0.6784 |
| integrals/pp_to_llj_dyn | 2/5 | 6.6e-7 | +1.0101 → +1.0101 |
| integrals/pp_to_llj_fixed | 2/5 | 8.3e-7 | +1.2915 → +1.2915 |
| integrals/ud_to_epemud_qcd0 | 5/5 | 1.4e-5 | +1.470 → +1.472 |
| integrals/uux_to_uux | 5/5 | 3.6e-11 | +0.4782 → +0.4782 |
| samples/pp_to_bb | 3/3 | 5.9e-7 | (KS gate) |
| samples/pp_to_bb_fixed | 3/3 | 2.0e-6 | |
| samples/pp_to_bb_qcd2 | 3/3 | 2.7e-7 | |
| samples/pp_to_jj | 3/3 | 4.9e-6 | |
| samples/pp_to_llj | 3/3 | 2.1e-5 | |
| samples/pp_to_llj_dyn | 3/3 | 5.4e-6 | |
| samples/pp_to_llj_fixed | 3/3 | 3.8e-6 | |

Each integrals cell that moved runs 5 or 10 seeds. The samples cells run the gate's
three generation seeds; their gate statistics moved by under 1e-7 relative (the
same events, reweighted by the σ that normalises them; `fbn-C1-samplescells.txt`, e.g.
`pp_to_llj` min KS p 2.320883e-02 → 2.320883e-02). For the five-seed reading AGENTS.md
asks of a moved enforced cell, the proton samples headroom probe ran after the change:

```
$ pixi run cargo test -p vibegraph --profile release-debug --features extended-validation --test validate_samples_proton -- --ignored --nocapture --exact probe_proton_samples_p_floor_headroom
  ROW pp_to_llj_fixed  mode gate | 3-seed min p 8.627e-2 (862.7x floor) | 5-seed min p 6.722e-2 (672.2x floor)
  ROW pp_to_llj_dyn    mode gate | 3-seed min p 1.910e-3 (19.1x floor)  | 5-seed min p 1.910e-3 (19.1x floor)
  ROW pp_to_bb_fixed   mode gate | 3-seed min p 8.349e-2 (834.9x floor) | 5-seed min p 6.147e-2 (614.7x floor)
  ROW pp_to_jj         mode gate | 3-seed min p 6.114e-3 (61.1x floor)  | 5-seed min p 6.114e-3 (61.1x floor)
  ROW pp_to_bb         mode gate | 3-seed min p 9.827e-2 (982.7x floor) | 5-seed min p 8.272e-2 (827.2x floor)
  ROW pp_to_bb_qcd2    mode gate | 3-seed min p 3.150e-2 (315.0x floor) | 5-seed min p 3.150e-2 (315.0x floor)
  ROW pp_to_llj        mode gate | 3-seed min p 2.321e-2 (232.1x floor) | 5-seed min p 1.301e-2 (130.1x floor)
HEADROOM samples_proton P_FLOOR 1e-4 | 3 seeds over 10 gating rows: min p 1.910e-3 (19.1x) ... | 5 seeds: min p 1.910e-3 (19.1x) on pp_to_llj_dyn seed 0x5a4d1002 KS y(l-)
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 16 filtered out; finished in 727.84s
```

Every moved samples row holds its floor over five seeds. The lepton-beam
`probe_samples_p_floor_headroom` ran too (`fbn-C1-samples5.log`); none of its rows
moved.

**Long tier: the Neyman fixed-budget gates.** `sigma_ll_0j2j_mlm_vs_madevent` and
`sigma_ttx_0j1j_mlm_vs_madevent` integrate at `--fixed-budget --allocate neyman`, the
path the change moves most. They are not in `validate`. So `pixi run --skip-deps
validate-mlm-sigma` ran after the change and then before it, on the same tree with the
change stashed (`fbn-C1-mlmbefore.sh`: stash, run, pop; the restored diff was checked
byte-equal to the saved patch):

```
$ pixi run --skip-deps validate-mlm-sigma      (after: 2098.83 s; before: 2074.28 s; both "test result: ok. 5 passed")
$ python3 -I fbn-C1-scripts/mlmcmp.py fbn-C1-mlmcells-before fbn-C1-mlmcells-after
pp_to_ll_0j2j_mlm__at0   gate pass->pass σ 665.4279±0.2311 -> 665.4327±0.2298 pull +0.49 -> +0.50 χ²/dof 0.94 -> 0.94 | seeds moved 10/10 paired Δ +0.0048 sd 0.0130 t +1.16
pp_to_ll_0j2j_mlm__at1   gate pass->pass σ 268.8866±0.3234 -> 268.8927±0.3230 pull +0.36 -> +0.37 χ²/dof 0.68 -> 0.67 | seeds moved 10/10 paired Δ +0.0061 sd 0.0369 t +0.52
pp_to_ll_0j2j_mlm__at2   gate pass->pass σ 132.4486±0.3836 -> 132.6936±0.4754 pull +1.90 -> +2.08 χ²/dof 0.97 -> 1.09 | seeds moved 10/10 paired Δ +0.2449 sd 0.4177 t +1.85
pp_to_ll_0j2j_mlm__total gate pass->pass σ 1066.7632±0.5524 -> 1067.0190±0.5811 pull +1.34 -> +1.54 χ²/dof 0.39 -> 0.46 | seeds moved 10/10 paired Δ +0.2558 sd 0.4144 t +1.95
pp_to_llj_mlm            gate pass->pass σ 268.7417±0.1155 -> 268.7417±0.1155 ... seeds moved 0/10
pp_to_llj_mlm_alps2      gate pass->pass σ 240.9774±0.1439 -> 240.9774±0.1439 ... seeds moved 0/5
pp_to_llj_xqcut_only     gate pass->pass σ 212.7019±0.1527 -> 212.7019±0.1528 ... seeds moved 1/5 (20260945: 212.24188849 -> 212.24168390)
pp_to_ttx_0j1j_mlm__at0  gate pass->pass σ 513.1846±0.1283 -> 513.1785±0.1293 pull +0.31 -> +0.28 ... paired Δ -0.0061 sd 0.0091 t -2.12
pp_to_ttx_0j1j_mlm__at1  gate pass->pass σ 579.1429±0.2574 -> 579.1536±0.2571 pull +0.70 -> +0.71 ... paired Δ +0.0107 sd 0.0106 t +3.20
pp_to_ttx_0j1j_mlm__total gate pass->pass σ 1092.3276±0.2876 -> 1092.3321±0.2878 pull +0.75 -> +0.75 ... paired Δ +0.0046 sd 0.0127 t +1.14
```

`pp_to_ll_0j2j_mlm` @2, the heavy-tailed two-jet part, moved most: it reads 0.18 %
higher on the same draws. Its error, the larger of
the quoted one and spread/√n under the seed policy, rose 0.38 → 0.48 pb, as the replay predicts
for a combine that stops down-weighting tail iterations. It now sits +0.88 % from
MadEvent at pull +2.08. The registered deviations account for about +0.7 %, and the
gate's 3σ is now 1.67 pb (1.3 %) against the 1.16 pb offset. Its headroom shrinks
(Found 2). The ttx shifts are 1e-5 of σ, with t +3.2 on @1 over 10 seeds, 4 % of
the cell's own error.

## Step 4: Neyman on bbx under the new combine

Fixed-budget Neyman draws the same iterations under both combines: allocation reads
`point_sd()` and the acceptance floors, never the combined term. The new
`combine_iterations` sums a channel's kept integrals in iteration order and divides by
k, the arithmetic `plainarm.py`/`perseed.py` replay. So the replay is the measurement.
Rerun with the probe built on the changed tree (`fbn-C1-bbx.sh`, `fbn-C1-bbx2.sh`;
binary `validate_sigma-c4ba13cafd06e45b`, built from the committed source; the
diff of `budget.rs` and `integrate.rs` it was built from hashes to `1444c201…`, as at
commit):

```
VG_2TO6_ROWS=bbx VG_2TO6_SAMPLERS=multi VG_2TO6_ALLOC=neyman VG_2TO6_RUNGS=<rung> VG_2TO6_SEEDS=<seeds> \
  VG_2TO6_OUT=fbn-C1-bbx.tsv validate_sigma-c4ba13cafd06e45b --ignored --nocapture --exact probe_2to6_budget_ladder
$ python3 -I fbn-C1-scripts/perseed.py fbn-P1-neyman-trace.txt     (replay, plain mean, pb)
```

| arm | rung | seed | rerun σ ± err (pb) | plain-mean replay σ ± err | old combine (P1 / B1 TSV) | points (rerun = old) |
|---|---|---|---|---|---|---|
| Neyman | 600k | 20260719 | 1.692208997e-6 ± 3.076791e-9 | 1.692208997e-6 ± 3.076791e-9 | 1.689621711e-6 ± 2.510791e-9 | 13 058 597 |
| Neyman | 600k | 11 | 1.693466235e-6 ± 3.114807e-9 | 1.693466235e-6 ± 3.114807e-9 | 1.692839994e-6 ± 3.197715e-9 | 12 967 689 |
| Neyman | 600k | 22 | 1.687975059e-6 ± 2.153577e-9 | 1.687975059e-6 ± 2.153577e-9 | 1.687805031e-6 ± 2.178610e-9 | 13 098 240 |
| Neyman | 1.2M | 20260719 | 1.691803723e-6 ± 2.460291e-9 | 1.691803723e-6 ± 2.460291e-9 | 1.688976331e-6 ± 1.702116e-9 | 17 796 879 |
| Neyman | 1.2M | 11 | 1.689328036e-6 ± 1.631516e-9 | 1.689328036e-6 ± 1.631516e-9 | 1.688336323e-6 ± 1.548998e-9 | 17 636 255 |
| ByAlpha | 600k | 20260719 | 1.692998322e-6 ± 3.232010e-9 | 1.692998322e-6 ± 3.232010e-9 | 1.692984697e-6 ± 3.231842e-9 | 13 058 820 |
| ByAlpha | 600k | 11 | 1.692570819e-6 ± 2.925258e-9 | 1.692570819e-6 ± 2.925258e-9 | 1.692529615e-6 ± 2.922915e-9 | 12 967 695 |

Every rerun draws the old run's points, and its σ and error equal the replay to every
printed digit (10 and 7 significant figures). The two ByAlpha seeds show the
fixed-budget move on bbx directly: +1.36e-11 and +4.12e-11 pb, 8e-6 and 2.4e-5 of σ.

The Neyman arm under the new combine is therefore the `plain` rows of step 1:

| arm | rung | mean | per-seed sd | RMS quoted | χ²/dof | pull vs reported / equal-weight | sd ratio |
|---|---|---|---|---|--:|---|--:|
| Neyman | 600k | 1.689739e-6 | 3.356e-9 | 3.002e-9 | 1.51 | +2.17 / +1.17 | |
| Neyman | 1.2M | 1.691395e-6 | 3.960e-9 | 3.995e-9 | 1.67 | +3.12 / +2.07 | 1.180 |
| ByAlpha | 600k | 1.690011e-6 | 2.714e-9 | 3.109e-9 | 0.80 | +2.46 / +1.38 | |
| ByAlpha | 1.2M | 1.691689e-6 | 3.330e-9 | 3.662e-9 | 1.41 | +3.47 / +2.32 | 1.227 |

Paired Neyman − ByAlpha: −2.72e-10 (t −0.35) and −2.94e-10 (t −0.22). Neyman spends the
same points (P1) and leaves the per-seed sd no smaller than ByAlpha at either rung. At
1.2M its variance × time is (3.960e-9)² × 1173 s = 1.84e-14 against ByAlpha's
(3.330e-9)² × 1297 s = 1.44e-14, using P1's wall times. P1's verdict stands on the
unbiased reading: Neyman does not fix the ladder.

## Step 5: the CLI's target mode

`vibegraph integrate <card> --run-card <run card> --target-rel 0.00179 --allocate
<neyman|by-alpha> --seed <1..20> --no-network`, all other flags at their defaults
(`--neval 120000`, `--min-iters 6`). The cards are `output/pp_to_llj/Cards/
{proc_card_mg5,run_card}.dat` (6 channels after merging) and `dy13_proc_card.dat` with
`dy13_default_run_card.dat` (`p p > e+ e-`, one channel). The binaries are
`fbn-C1-vibegraph-before` (2d37f4c, sha256 22667838…) and `fbn-C1-vibegraph-after`
(the change, sha256 471714228…), copied out of the two `validate` builds. Driver:
`fbn-C1-cli.sh`/`fbn-C1-cliarms.sh`. References are MadEvent's seed records:
`pp_to_llj` 504.322 ± 0.876 pb and `dy13_default` 933.578 ± 0.184 pb (5 seeds, reported).

```
$ python3 -I fbn-C1-scripts/cliana.py 504.322 0.8758 fbn-C1-cli-old-llj.tsv fbn-C1-cli-new-llj.tsv
arm                      n  evals mean         evals min-max   iters  met  sigma mean       sd RMS quoted sd/RMSq chi2/dof   pull  wall s
old-llj neyman          20      1.494M     1.200M-1.800M    12.4   20    505.3289   0.8158     0.8119   1.005     1.03  +1.13    18.1
old-llj by-alpha        20      1.584M     1.320M-2.040M    13.2   20    505.3145   0.8834     0.8014   1.102     1.22  +1.11    19.7
new-llj neyman          20      1.494M     1.200M-1.800M    12.4   20    505.3346   0.8082     0.8128   0.994     1.00  +1.13    18.7
new-llj by-alpha        20      1.584M     1.320M-2.040M    13.2   20    505.3145   0.8834     0.8014   1.102     1.22  +1.11    20.2
  new-llj neyman - old-llj neyman: n 20 dσ +0.0057 sd 0.0335 t +0.76 | d evals +0.000M | identical seeds 0
  new-llj by-alpha - old-llj by-alpha: n 20 dσ +0.0000 sd 0.0000 t +nan | d evals +0.000M | identical seeds 20
  new-llj by-alpha - new-llj neyman: n 20 dσ -0.0201 sd 0.4125 t -0.22 | d evals +0.090M | identical seeds 0
paired evaluations ByAlpha − Neyman (both binaries): mean +0.090M sd 0.178M t +2.26; iterations +0.75; Neyman fewer on 15, more on 5
Neyman seeds with the same iteration count old/new: 20/20
$ python3 -I fbn-C1-scripts/cliana.py 933.578 0.1836 fbn-C1-cli-old-dy13.tsv fbn-C1-cli-new-dy13.tsv
old-dy13 neyman / by-alpha, new-dy13 neyman / by-alpha: n 8 each, 0.855M (0.720M-1.200M), 7.1 iterations, 8/8 met, σ 934.1798, sd 1.0062, RMS quoted 1.4451, χ²/dof 0.40, pull +1.50 — all four arms identical on every seed
```

Per arm, the new binary:

| process | arm | evaluations to target, mean (range) | met | σ (pb) | pull vs seed record | per-seed sd / RMS quoted | χ²/dof |
|---|---|---|---|---|--:|---|--:|
| `p p > l+ l- j` | Neyman | 1.494M (1.200M–1.800M) | 20/20 | 505.335 | +1.13 | 0.808 / 0.813 | 1.00 |
| `p p > l+ l- j` | ByAlpha | 1.584M (1.320M–2.040M) | 20/20 | 505.315 | +1.11 | 0.883 / 0.801 | 1.22 |
| `p p > e+ e-` | Neyman = ByAlpha | 0.855M (0.720M–1.200M) | 8/8 | 934.180 | +1.50 | 1.006 / 1.445 | 0.40 |

- **Evaluations.** Neyman's advantage is ×0.943, paired t +2.26 over 20 seeds, with
  Neyman fewer on 15. The change moves no stop. Each Neyman seed stops at the same
  iteration under both combines, and ByAlpha's outputs are byte-identical.
- **σ.** The two arms agree (paired −0.020 pb, sd 0.41) and sit +1.1σ from the seed
  record. The new combine moves Neyman's σ by +0.006 pb (sd 0.034, largest +0.106 pb
  on seed 14), a 25th of the quoted error.
- **What changed since the documented figures** (2.94M against 6.41M, 8 seeds): the
  channel merge took `p p > l+ l- j` from 24 channels to 6 and `p p > e+ e-` from 4 to
  1. The stop rule now uses the pooled factor rather than the per-iteration χ². The
  old binary, which still has the old combine, reads ×0.943 too. So the shrinkage
  predates this change: the combine was not what made Neyman's target mode pay.

Both docs are rewritten with the 20-seed figures: `BlockAllocation::Neyman` in
`budget.rs` and `--allocate` in `integrate.rs`, plus the regenerated
`docs/src/cli/reference.md`.

## Step 6: the default

Neyman's target-mode advantage did not disappear (×0.943, t +2.26), and its answer did
not move (Δσ +0.006 pb, t +0.76). Nothing calls for a D1, and no default, tolerance or
budget changed. For the user's information: the advantage is now 6 % of evaluations,
not the documented factor of two.

## Gates

On the committed tree (the source the bbx reruns were built from, plus the regenerated
`docs/src/cli/reference.md`, the sessions index and this report), detached
(`fbn-C1-gates.sh`, log `fbn-C1-gates.log`), with `CARGO_INCREMENTAL=0
CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0`:

```
$ pixi run bash scripts/gen-cli-docs.sh                                     --> exit 0 (rewrites the --allocate paragraph only)
$ pixi run cargo fmt --all --check                                          --> exit 0
$ pixi run cargo clippy --workspace --all-targets -- -D warnings            --> exit 0
$ pixi run cargo clippy --workspace --all-targets \
    --features vibegraph/extended-validation,vibegraph-lib/extended-validation -- -D warnings   --> exit 0
$ pixi run cargo test --workspace                                           --> exit 0
  47 test-result lines: 1647 passed, 0 failed, 19 ignored   (P1's tree: 1646; one test replaced, one added)
$ pixi run kb-index                                                         --> exit 0
$ pixi run --skip-deps kb-lint                                              --> exit 0
  928 concepts, 302 backlog items; 0 errors, 4 warnings   (the four pre-existing `state: blocked` items P1 listed)
```

The statistical gates are step 3's: `pixi run --skip-deps validate` before and after
(both exit 0, 206 ✅ / 5 ⚠️ / 13 ⏳, 18 cells moved, none changed status),
`validate-mlm-sigma` before and after (5 passed each, 8 cells moved, none changed
status), and the five-seed proton samples probe after (every row above its floor).

## Method

- Scripts: `fbn-C1-scripts/replay.py`, `nonuniform.py`, `perseed.py`, `cells.py`,
  `mlmcmp.py`, `cliana.py`, `toy.py` (the Python prototype of the test's toy). They
  read P1's traces in place through `fbn-P1-scripts/ana.py`'s loader.
- Drivers, each launched with `nohup setsid`: `fbn-C1-validate.sh <before|after>`,
  `fbn-C1-task.sh`, `fbn-C1-mlmbefore.sh`, `fbn-C1-cliarms.sh <binary> <old|new>`,
  `fbn-C1-bbx.sh`, `fbn-C1-bbx2.sh`. Logs and TSVs are `fbn-C1-*.log` / `*.tsv`, and the
  saved report directories are `fbn-C1-report-{before,after}` and
  `fbn-C1-mlmcells-{before,after}`.
- Every cargo invocation ran with `CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0
  CARGO_PROFILE_TEST_DEBUG=0`.
- `fbn-C1-bbx.sh` was first launched with `VG_2TO6_SEEDS=20260719` for one seed. The
  probe reads a lone integer as a seed *count*, so it ran 600k seeds 20260719, 11 and 22
  in order before I stopped it by PID. All three agree with the replay and are kept.
  The rest ran with explicit lists.

## Brief corrections

- **"Weights from counts fixed before the run's draws (for example the α shares
  after the survey)"** is not a different candidate. The budget's `neval` is the same
  every iteration, so any pre-run weight is constant per channel and the weighted
  mean is the plain one (replay: 3.8e-16). The only non-trivial outcome-independent
  alternatives are schedules in iteration index, and the one tried (`index`) trades
  variance between rungs without removing anything `plain` leaves.
- **Step 3's validate pair does not cover the gates the change moves most.** The
  Neyman fixed-budget rows (`pp_to_ll_0j2j_mlm`, `pp_to_ttx_0j1j_mlm`) are long tier.
  I ran `validate-mlm-sigma` before and after as well.
- **Step 5's process descriptions are out of date.** `p p > l+ l- j` integrates 6
  channels and `p p > e+ e-` one, so the second has nothing to reallocate.
- **Step 4's "exact" holds to every printed digit**, not only to the 2.9e-10 P1 quoted
  for its production replay: σ and its error agree to 10 and 7 significant figures on
  every rerun seed.

## Found

1. **The kb still describes the count-weighted combine and the old target figures.**
   - `research/kb/phase-space/convergence-stop-rule.md` gives `quoted = Σ n_i² σ_i² /
     W²` and `emp = Σ n_i² (I_i − Ī)² / W² · k/(k − 1)`.
   - `research/kb/phase-space/channel-budget-allocation.md` says the acceptance
     re-read avoids "the bias the point-count combination rule exists to avoid". It
     quotes 2.94M against 6.41M for target-mode Neyman, and its fixed-Neyman variance
     gains on `pp_to_ll_0j2j_mlm` ("quoted variance 1.00–2.44× lower, mean 1.69×")
     were quoted errors under the biased combine.
   - To close it: the close-out re-records these from this report (step 5's table)
     and re-measures the fixed-Neyman gain on the mixed row under the plain mean.
2. **`pp_to_ll_0j2j_mlm` @2's gate headroom shrank.**
   - The unbiased combine reads @2 +0.245 pb higher (t +1.85, 10 paired seeds): 132.694
     ± 0.475 against MadEvent's 131.532 ± 0.291, +0.88 %, pull +2.08.
   - The registered deviations explain about +0.7 %, and the gate's 3σ is 1.67 pb.
   - The manifest note's figures (132.4486 ± 0.3836, +1.90) are the old combine's.
   - To close it: re-record the cell's note, and decide whether the mixed row's
     fixed-budget allocation should stay Neyman. Under the plain mean, Neyman's
     fixed-budget variance gain there is unmeasured (Found 1).
3. **Fixed-budget Neyman is not shown to beat ByAlpha anywhere under an unbiased
   combine.**
   - bbx: sd no smaller at either rung, variance × time 1.84e-14 against 1.44e-14.
   - The mixed row's measured gain was the biased combine's quoted error.
   - The banked mixed-row gates and the Pythia matching record use `--fixed-budget
     --allocate neyman`.
   - To close it: one paired 20-seed comparison (Neyman against ByAlpha,
     `pp_to_ll_0j2j_mlm`, the gate's budget) read through the plain mean.
4. **bbx's plain-mean Neyman reads χ²/dof 1.51 and 1.67 over 20 seeds** (p ≈ 0.07 and
   0.035 on 19 dof). Its quoted error is 0.89 of the spread at 600k and matches it at
   1.2M. ByAlpha reads 0.80 and 1.41. The heavy-tailed iterations Neyman feeds are
   counted at full weight now, and their quoted errors are optimistic. To close it:
   this is the floor-bound item's measurement (`floor-bound-channels-ignore-budget`),
   which can now run. Its `blocked_by` on this item can be cleared.
