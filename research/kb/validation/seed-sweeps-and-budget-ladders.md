---
type: Validation Methodology
title: Seed sweeps, budget ladders and when a pull is asserted
description: "A fixed-seed pull is not evidence: sweep at least five seeds and read spread and chi2/dof, add a budget ladder for shared bias, calibrate rung differences on measured spread, and assert a pull only for fluctuations."
status: draft
tags: [validation, seed-sweep, budget-ladder, vegas, statistics]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: n21-prod, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/21-resonance-sampling-and-events-plan.md#L300-L381", title: "Note 21 addendum (sampler in production: two defects found by seed sweeps)"}
  - {id: n24-p3, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/24-user-distribution-and-proton-events-plan.md#L1671-L1772", title: "Note 24 P3 (five-seed sweep necessary, not sufficient)"}
  - {id: n27-b1, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/27-v3-backlog-plan.md#L44-L211", title: "Note 27 B1 (Higgs pole: MadGraph confidently wrong)"}
  - {id: n28-s6, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/28-kt-spine-feature-sprint-plan.md#L2711-L2745", title: "Note 28 S6 (ud row: two axes)"}
  - {id: n28-k5b1, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/28-kt-spine-feature-sprint-plan.md#L3059-L3089", title: "Note 28 K5b.1 (four partonic rows, both axes)"}
  - {id: n28-k65, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/28-kt-spine-feature-sprint-plan.md#L3382-L3433", title: "Note 28 K6.5 (channel partition residual)"}
  - {id: n28-c4, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/28-kt-spine-feature-sprint-plan.md#L3663-L3700", title: "Note 28 C.4 (pp_to_jj σ, both axes)"}
  - {id: n28-c24, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/28-kt-spine-feature-sprint-plan.md#L3918-L3984", title: "Note 28 C2.4 (pp_to_jj gated)"}
  - {id: n29-d4, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/29-v01-validation-sprint-plan.md#L3041-L3057", title: "Note 29 D.4 (seed and budget protocol)"}
  - {id: n29-b8, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/29-v01-validation-sprint-plan.md#L5168-L5197", title: "Note 29 B.8 (tolerance decision rule, pre-registered)"}
  - {id: n29-bres, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/29-v01-validation-sprint-plan.md#L5523-L5756", title: "Note 29 chain B results"}
  - {id: n29-g10, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/29-v01-validation-sprint-plan.md#L5947-L5988", title: "Note 29 G.10 (re-carded rows: the second axis)"}
  - {id: n34-floor, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/34-draw-followup-plan.md#L56-L136", title: "Note 34 §1.2 (the gate cascade)"}
  - {id: n34-w1, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/34-draw-followup-plan.md#L145-L250", title: "Note 34 wave 1 (S1 survey seeds; S2 the climb was a misread)"}
  - {id: vegas, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/vibegraph-lib/src/vegas.rs#L121-L135", title: "vegas.rs IterationCombination"}
  - {id: vhad, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/vibegraph-lib/tests/validate_hadronic.rs#L412-L423", title: "validate_hadronic.rs combine_seeds"}
---

`AGENTS.md` ("Samplers gate statistically, and a fixed-seed pull is not
evidence") states the binding rule. This concept holds the reasons and the
cases that taught it, so a session can recognise which failure it is looking at.

## Why a fixed-seed pull is not evidence

With a pinned seed and an unchanged sampling order an integral is
deterministic, so a one-seed gate is reproducible but says nothing about the
distribution it was drawn from. VEGAS makes this worse than ordinary noise: an
iteration that misses a narrow region reports a small integral **and** a small
variance, and a `1/σ²` combination of iterations weights it up. The result is a
*confidently* wrong σ. The worst observed case was 25× low, quoted as
`5.48e-5 ± 2.79e-6`, a 5% error bar; a single-seed pull reads that as a mild
few-σ miss.[^n21-prod]

The first resonant row put into production read pull +3.19 at its one seed,
which looks like an ordinary miss. Sweeping seeds exposed two defects, each
diagnosed by refuting cheaper hypotheses:[^n21-prod]

| hypothesis | test | result |
|---|---|---|
| α-adaptation collapsed a channel | `probe_alpha_collapse` | refuted: no channel at the floor, and uniform α collapsed too |
| survey budget too small | 30k → 300k survey | refuted: rescued one seed, broke another. **The failure moved instead of converging** |
| the low-`m_ll` photon pole | `probe_photon_pole_is_the_instability` | confirmed: `mmll` 0 → 20 GeV took the five-seed spread from 24.96× to 1.01×, χ²/dof 1159 → 1.18 |

The cause was a massless timelike pole drawn flat against a `1/(s−m²)²` rise.
A second defect remained under the fixed map: one seed's grid collapsed into a
corner at iteration 4 (`probe_vegas_iteration_path`), fixed by damping the grid
adaptation on mapped channels (`VEGAS_ALPHA_MAPPED = 0.5`; Lepage's 1.5 assumes
the grid must discover structure the map already removed). See
[VEGAS integrator](../phase-space/vegas-integrator.md) and
[resonance and pole maps](../phase-space/resonance-and-pole-maps.md).

MadGraph is not exempt. Three fresh MadEvent seeds of the `h → ττ` row agreed
with each other and with the banked run while all missed the Higgs pole by 2.3%
under a 0.2% quoted error; the windowed re-integration found it (MadGraph 3.5.7's
`sde_strategy = 2` defect).[^n27-b1] See
[windowed partition closure](windowed-partition-closure.md) and the
[MadEvent seed policy](madevent-reference-seed-policy.md), which applies the
same rule to references.

## The second axis: budget

A seed sweep detects a seed that missed a region. It cannot detect a bias every
seed shares, because the seeds do not disagree about it.[^n24-p3] The case: the
`p p > l+ l- j` gate at `neval = 60 000` passed a single-seed check, a
five-seed check and the scatter check (pulls −2.36 … −1.08, χ²/dof 1.55) while
being 1.0% low. The budget scan showed steps halving as the budget doubled
(−3.18, −1.19, −0.67 pb), an `O(1/N)` bias. Its source was VEGAS putting every
iteration, including the unadapted first ones, into an inverse-variance
mean.[^n24-p3] VEGAS now combines iterations by **unweighted mean**
(`IterationCombination::Unweighted`, the `#[default]` in `vegas.rs`): weights
fixed before sampling cannot correlate with what they weight.[^vegas] See
[iteration combination](../phase-space/vegas-iteration-combination.md). The
same reasoning sets how a gate combines its seeds: `combine_seeds` in
`validate_hadronic.rs` takes the unweighted mean with `err = √(Σσᵢ²)/n`, and
the seeds' scatter, not the error, is what shows a missed region.[^vhad]

So any σ gate on a many-channel or high-dimensional integrand records a budget
ladder (typically five seeds per rung over an eightfold range, 75k–600k) beside
its seed sweep, and the budget it settles on is recorded with the ladder that
chose it, in the constant's doc comment.[^n24-p3] Choosing budgets against the
reference's precision is [budget alignment](budget-alignment-rule.md).

### Reading a ladder

| ladder shape | reading | action |
|---|---|---|
| flat, scatter near χ²/dof 1 | converged | gate at the lowest flat rung with clean scatter |
| increments shrinking (halving), crossing the reference | convergence from one side | gate at a rung past the climb, never at a tolerance wide enough to admit a moving number |
| a failure that moves between seeds as budget grows | **a bug**, not statistics | diagnose |
| residual fixed at a value the reference's error floors | not resolvable from this side | `rel_tol` from the seed spread |

Worked cases:

- The re-carded `pp_to_llj` (low-`m_ll` photon-pole side open) read −2.07%,
  −0.80%, −0.30%, +0.02% over 75k–600k: increments shrinking, scatter clean,
  convergence rather than a defect. Gating at the 300k reading would have been
  enforcing the climb.[^n29-g10] (Its current budget and reading are in the
  manifest; see the caveat on rung differences below.)
- `ud_to_epemud_qcd0`: at 4× budget our error is a third of MadGraph's, so the
  pull is floored by the reference and the ladder cannot shrink the residual.
  What the ladder rules out is a defect, which would migrate between seeds at
  fixed size rather than scatter inside a band. `rel_tol` was set at 3.8× the
  worst seed.[^n28-s6]
- `pp_to_jj` climbs 0.11% over an eightfold budget, half the reference's own
  error, without resolving an asymptote: converged *at the scale the comparison
  is made at*, not demonstrably asymptotic. The ladder is kept so a later
  session can say more.[^n28-c4][^n28-c24]
- Nothing in the two low llj partonic rows was sampling: quadrupling the budget
  left them at −5.5% on all ten runs. The ladder, not the sweep, said
  so.[^n28-k5b1]
- The 2→6 rows' five-seed means agree with the bank, but single seeds swing
  ±4–5% at both ends of a 300k–1.2M ladder without shrinking. That was
  classified as a heavy-tailed estimator; a fix must make the swings shrink with
  budget, not merely move them. See
  [sigma-row gating exceptions](sigma-row-gating-exceptions.md).

## Rung-to-rung differences need the measured spread

Five seeds detect a missed region; they cannot calibrate a difference between
two rungs. The re-carded `pp_to_llj` ladder was later read as a monotone climb
(+0.04% → +0.21%) and treated as an unexplained drift. A 40-seed ensemble per
rung (`probe_llj_seed_ensemble`) put the estimator's expectation flat from 150k
up, killing the drift hypothesis at 7.3σ: the recorded ladder had been one
five-seed draw, 2.3σ low at its bottom rung. Five-seed scatter understated that
row's measured per-seed spread by 2× at 150k and 5× at 600k, where it produced
a χ²/dof of 0.03, as loud a warning as 4.0 and read as reassurance. So a
ladder comparison is read against the estimator's measured seed spread, from 20
or more seeds on the two rungs that matter.[^n34-w1] The falsifier for that
verdict is recorded: about 0.1% between 40-seed rung means would overturn it.
The gate comments that still quote the older ladder are
[llj gate comments quote pre-floor ladders](../backlog/hygiene/llj-gate-comments-quote-pre-floor-ladders.md).

Survey seeds count too: a "wide rows degrade above the cap" reading on the 2→6
α survey was one unlucky α draw, killed by sweeping three survey
seeds.[^n34-w1]

## A gate statistic is formed on as many seeds as calibrated it

A change to the sampling stream re-rolls every one-seed or three-seed gate. When
cut-implied floors landed, three gate cells failed one after another (cargo
stops at the first failing binary), all the same defect: a statistic formed on
fewer seeds than its threshold was calibrated on, sitting at the
threshold.[^n34-floor]

- `pp_to_llj_dyn`'s three-seed scatter read χ²/dof 4.24 against a 4.0 bound
  calibrated on five-seed ladders, while the five-seed reading was 2.49 and σ
  moved *closer* to MadGraph. Both llj gates now form over the five calibration
  seeds.
- `ee_to_mumua`'s σ pull failed at 3.56 on the gate's one seed; five seeds
  showed a fixed +1.04% on both arms, which moved the row to the reported-pull
  list (below).
- `ee_to_mumua`'s samples KS crossed the floor on one seed of three with the
  shape unchanged; the pre-registered prescription moved the cell to `info`.

Each was measured two-arm (with and without the change) before anything was
proposed, and none was widened. The standing census of how many seeds form and
calibrate each statistic is
[seed headroom census](seed-headroom-census-2026-09.md); thresholds are
[gate thresholds](gate-thresholds.md). Extremum-vs-floor statistics (the samples
`P_FLOOR`) are the exception: more seeds there raise the false-flag rate.

## When a pull is asserted

A pull bounds a disagreement only while the residual is Monte Carlo, shrinking
as either side's budget grows. A systematic of measured size drives the pull to
infinity as `err_vg` falls, so asserting it would assert a precision the
comparison does not have and would eventually fail a row that had not moved.
Such rows report the pull and enforce `rel_tol`
(`PULL_REPORTED_NOT_ASSERTED`).[^n28-k65] The decision rule, fixed before the
measurement that applies it:[^n29-b8]

- A row leaves the reported list only if its residual shrinks with budget across
  the ladder **and** its five-seed scatter sits at χ²/dof ≈ 1. A residual of
  fixed size that merely got smaller is still a systematic.
- `rel_tol` is then the larger of the reference's own error with headroom and
  the measured five-seed spread, never fitted to the achieved central value.
- If the residual does not become Monte Carlo, nothing moves, and that is
  reported.

The worked case is the reason the clustering configuration is now drawn per
point `∝ AMP2_c`: while the cluster scale was read in the channel the sampler
drew the point in, σ depended on the channel partition (`gu_to_epemu` moved by
1.5e-2 between converged and uniform `αⱼ`, at 9σ, while the two rows whose
scale no configuration moves were the negative control). After the draw the
partition gap fell to Monte Carlo on all four rows, the rows' residuals shrank
with budget, and both left the list under the rule above.[^n28-k65][^n29-bres]
See [the configuration draw](../scales-pdf/clustering-configuration-draw.md).
A draw like that is noisier at low budget (`pp_to_llj_dyn`'s 75k rung scattered
at χ²/dof 6.38), so a row gated near such a rung would feel it.[^n29-bres]

## Protocol for an attribution measurement

When a disagreement is being attributed to one side, both sides are swept and
the spread reported, never a headline pull:[^n29-d4]

- ours: five seeds (e.g. `{20260719, 11, 22, 33, 44}`, the
  `probe_resonant_seed_stability` set, so the sweep compares with the recorded
  one) at the gate budget and at 4×;
- MadGraph: five seeds with the banked run's own cards, only `nevents` and
  `iseed` changed (MadEvent's refine targets scale with `nevents`);
- report spread and χ²/dof on both sides.

A heavy shared host does not move σ, χ² or α statistics, but timings taken
while another heavy suite runs are not quotable.[^n34-w1]

[^n21-prod]: Note 21, "Addendum — putting the sampler into production".
[^n24-p3]: Note 24 P3: the five-seed sweep was necessary and not sufficient; plan corrections.
[^n27-b1]: Note 27 B1 outcome.
[^n28-s6]: Note 28 S6, "The row, promoted".
[^n28-k5b1]: Note 28 K5b.1.
[^n28-k65]: Note 28 K6.5.
[^n28-c4]: Note 28 C.4.
[^n28-c24]: Note 28 C2.4.
[^n29-d4]: Note 29 D.4.
[^n29-b8]: Note 29 B.8.
[^n29-bres]: Note 29 chain B results.
[^n29-g10]: Note 29 G.10.
[^n34-floor]: Note 34 §1.2.
[^n34-w1]: Note 34 wave 1, S1 and S2 close-outs.
[^vegas]: `vibegraph-lib/src/vegas.rs`, `IterationCombination`.
[^vhad]: `vibegraph-lib/tests/validate_hadronic.rs`, `combine_seeds`.
