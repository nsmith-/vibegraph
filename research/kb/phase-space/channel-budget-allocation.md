---
type: Algorithm
title: "Per-channel budget allocation: α split, Neyman re-split and coverage floors"
description: "Deterministic per-channel N_j (ByAlpha, or Neyman ∝ s_j); floors of MIN_CHANNEL_NEVAL/acceptance capped at 4× that override --neval on wide splits; the starvation and redundancy they caused."
status: draft
tags: [phase-space, budget, neyman, multichannel, performance]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-09}]
sources:
  - {id: n32-s7, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/32-perf-addendum-plan.md#L865-L955", title: "Note 32 §7 (time to target; the floor overrides --neval)"}
  - {id: n34-s3, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/34-draw-followup-plan.md#L251-L325", title: "Note 34 S3 (floor counts accepted points)"}
  - {id: n41-m3, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/41-mlm-feature-sprint-plan.md#L1014-L1414", title: "Note 41 M3 (budget across multiplicities; Neyman starvation found)"}
  - {id: n41-fb, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/41-mlm-feature-sprint-plan.md#L2006-L2298", title: "Note 41 F-B (merging removes redundant floors)"}
  - {id: n41-m6, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/41-mlm-feature-sprint-plan.md#L2702-L2915", title: "Note 41 M6 (Neyman handed the α split's spend)"}
  - {id: n41-z, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/41-mlm-feature-sprint-plan.md#L2916-L3489", title: "Note 41 Z close-out (mixed-row gate budget)"}
measured:
  - {landed_in: b6a0b88, command: "five-seed sweeps on the 2→6 rows and pp_to_llj"}
  - {commit: 069a951, host: "4-core container shared with another session", command: "vibegraph integrate pp_to_ll_0j2j_mlm --fixed-budget --allocate neyman --neval 200000 --niter 8"}
  - {commit: ef660f3, host: "4-core container shared with another session (load 7-12)", command: "vibegraph integrate pp_to_ll_0j2j_mlm --fixed-budget --allocate neyman --neval 200000 --niter 8, seeds 20260928-32 (the unmerged base; the merged arm, seeds 20260928-37, ran on F-B's build, whose commit note 41 does not name)"}
---

# Per-channel budget allocation

A multichannel integration is a **hard split**: the integral is a sum of
per-channel terms, each sampled over its own grid with deterministic point
counts `N_j` per iteration, rather than a per-point channel draw
([phase-space/per-channel-vegas-grids](per-channel-vegas-grids.md),
[phase-space/multichannel](multichannel.md)). `vibegraph-lib/src/budget.rs`
decides `N_j` every iteration, and whether to run another iteration
([phase-space/convergence-stop-rule](convergence-stop-rule.md)).

## The two rules

`BlockAllocation` (`budget.rs:179`):

| Rule | `N_j` | CLI default under |
|---|---|---|
| `ByAlpha` | `round(α_j · neval)`, floored, fixed for the run | `--fixed-budget` (reproducible) |
| `Neyman` | `∝ s_j`, the running per-point standard deviation of channel `j`'s term, floored, recomputed from completed iterations from the second iteration on | `--target-rel` (the default mode) |

`--allocate by-alpha|neyman` overrides the default
(`vibegraph-cli/src/integrate.rs:389`). `s_j` is the spread of *this
estimator's* term, which already carries `α_j`; on the unweighted channel
integrand `h_j = f/g` the rule reads `N_j ∝ α_j · sd(h_j)`.

The α split is not naive: Kleiss–Pittau adaptation sets `α_j` from a variance
survey, so it is most of a variance estimate already. On `p p > l+ l- j`, `α_j`
spans ~100× across 24 channels while `s_j/α_j` spans 2–4.5, and re-splitting a
fixed budget by Neyman buys only 1.00–1.22× in variance (1.000 on
`p p > e+ e-`'s four channels). Neyman pays against a **target**: the channels
it feeds are the ones whose iterations disagree with themselves, and the
stopping rule's consistency factor charges exactly those. On `p p > l+ l- j`
at a 0.179% target over 8 seeds it took 2.94M evaluations against 6.41M, and
the seed-to-seed spread of the spend narrowed from 2.75M–12.4M to 1.97M–4.26M,
with σ agreeing in both arms.

## Coverage floors

Allocation splits the spend, never the coverage. A channel whose map is the
only one with density on some structure must keep sampling it even when its
variance estimate says it is cheap: a channel that stops covering its own
region is how a multichannel integral becomes confidently wrong.

- **`MIN_CHANNEL_NEVAL = 512`** (`budget.rs:124`), denominated in **accepted**
  points. A point the cuts reject contributes exactly zero to the term and to
  its variance, so it covers nothing; untrained `p p > l+ l- j` channels accept
  under a quarter of their draws, so 512 drawn points were ~120 of coverage.
- **`floor_for_acceptance(acceptance_j)`** (`budget.rs:150`) returns
  `⌈512 / acceptance_j⌉`, capped at
  **`MAX_FLOOR_ACCEPTANCE_SCALE = 4`** times the floor (`budget.rs:139`), i.e.
  at most **2048** points. Cold start (no acceptance measured yet) gets 512; a
  channel that has accepted nothing gets the cap. The cap of 4 sits at the
  wide rows' measured median acceptance (~0.25); a channel accepting less
  buys fewer than 512 accepted points (on `pp_to_ll_0j2j_mlm` before merging,
  144 of the 336 two-jet channels sat at the cap).[^n41-m6]
- `acceptance_j` is read from the channel's own **completed** iterations,
  never from the draws it sizes. Counting draws until enough are accepted was
  rejected: that count correlates with the iteration's own estimate, the bias
  the point-count combination rule exists to avoid.
- Where the cap binds, the coverage promise is not kept, and the run says so
  (`report_floor_correction`).

### The floor overrides `--neval` on wide splits

When `Σ_j max(share_j, floor_j)` exceeds `--neval`, coverage wins and the run
warns: the spend is then set by the channel count. Before any acceptance is
measured an iteration costs `Σ_j max(share_j, 512)`, at least
`n_channels × 512`, and the acceptance correction can raise that by at most
the factor 4, a bound computable before spending anything (the warning in
`integrate_channels` prints both). Above roughly `neval / 512` channels it is
the channel count, not `--neval`, that sets the budget. Measured on
2026-08-06, before the floor was acceptance-scaled: on the 579-channel
`u u~ > c c~ e+ e- mu+ mu-` row, 554 channels sat at the floor and an
iteration spent 399,217 points against `--neval 120000` (≈230
channels).[^n32-s7] A single-channel integration has no floor; it spends
`neval`.

Measured when the floor became acceptance-scaled:[^n34-s3] tail suppression,
not cheaper points. The worst single-seed relative error on the `2 → 6` `bbx`
row fell from +3.82% to +0.46% and its over-seed χ²/dof from 13.5 to 0.33;
`p p > l+ l- j`'s 512-accepted promise, which a draw-denominated floor misses by 2×, was met; wide
rows went from 0–3 to 5–14 accepted points per channel per iteration. The
cost: wide-row oracle runs spend about 2×, and the full `validate` run +1.9%.

## Neyman is handed the α split's own spend

The re-split is given `Σ_j max(share_j, floor_j)`, the total the α split
spends *in the same iteration*, floors included, and moves only the points
above the floors (`integrate_channels`, `budget.rs`, the block that calls
`neyman_allocation`). The acceptance correction is thus a coverage cost both
rules pay alike. `neyman_allocation` (`budget.rs:548`) pins channels whose
proportional share falls below their floor and re-splits the rest
repeatedly, so floors are exact rather than a clamp that overspends; if every
channel is pinned the total rises to `Σ floor_j`.

Handed the uncorrected total instead, the raised floors of a wide,
low-acceptance channel set absorb the budget and starve everything else. On
`pp_to_ll_0j2j_mlm` before channels were merged, 336 two-jet floors raised up
to 2048 (~570k points) exceeded that total (364k): every channel sat at its
floor and the four zero-jet channels drew ~2k points instead of ~162k, putting
`@0` at ±6.6 pb against ±0.70 by α at the same seed.[^n41-m3] With the fix
(`budget::a_neyman_split_spends_what_the_alpha_split_spends_past_raised_floors`,
a 41-channel toy that fails on the old total):[^n41-m6]

- by-α runs are bit-identical, and so are Neyman runs whose floors never rise
  above their shares;
- fixed Neyman against by α at equal points: quoted variance 1.00–2.44× lower,
  mean 1.69×;
- against the starved Neyman: 13× less rel²·CPU (0.143 to 0.0110 s).

## Merged channels pay one floor

Floors are per sampling channel, so redundant channels multiply the floor
spend. Merging channels with identical maps
([phase-space/channel-set](channel-set.md)) took `pp_to_ll_0j2j_mlm` from 364
channels to 43. Measured against the unmerged base at
`--fixed-budget --allocate neyman --neval 200000 --niter 8`:[^n41-fb]

| | unmerged (5 seeds) | merged (10 seeds) |
|---|---|---|
| floor points added per iteration | ~443k | ~43k |
| evaluations over 8 iterations | 4.52M | 1.85M |
| CPU per point (loaded host) | 1.20 ms | 0.48 ms |
| `@2` σ (pb), sd, χ²/dof | 132.06, 0.51, 0.32 | 131.26, 2.07, 1.89 |
| total rel²·CPU, by quoted error / by seed spread | 1.10e-2 / 4.37e-3 s | 3.64e-3 / 2.89e-3 s |

At the same `--neval` the floors no longer buy `@2` ~600k points an iteration
against the ~13k its share asks for, and `@2`'s heavy tail makes its quoted
spread (which the part split and Neyman both read) an underestimate. So by
seed spread the gain is 1.5×, not the 3× the quoted errors claim. At the same
point count (`--neval 600000`, three seeds) every part improves, 8.6× in
rel²·CPU by quoted error. `--neval` is therefore the knob that sets `@2`'s
precision until the allocation prices the tail
([backlog](../backlog/performance/mlm-at2-tail-unpriced-in-allocation.md)).
On `pp_to_llj_mlm` (24 → 6 channels; by α, `--fixed-budget --neval 150000
--niter 10`, ten seeds) no floor binds after merging; seed χ²/dof fell from
3.06 to 0.80 and rel²·CPU by seed spread 5.7×.

The banked mixed-row σ gate integrates at exactly that configuration (ten
seeds, `--fixed-budget --allocate neyman --neval 200000 --niter 8`); `@2`'s
error is then its seed spread, 0.66 pb.[^n41-z]

## Across multiplicities

A sum over `@N` parts ([phase-space/mixed-multiplicity-integrand](mixed-multiplicity-integrand.md))
first splits the per-iteration budget between parts by `n_k ∝ s_k`, the
standard deviation of part `k`'s undivided mixture estimator,
`√(Σ_j α_j W_j − σ_k²)` from its α survey (`MultiplicitySum::adapt_alphas`,
`multiplicity.rs:307`); each channel then gets `α_j · share_k`. `s_k` ranks the
parts rather than predicting their errors, and under Neyman it sets only the
first iteration.

## Open

- The part split and the Neyman re-split read quoted spreads that heavy tails
  understate (backlog item above).
- The α survey is iteration-limited on wide splits
  ([backlog](../backlog/performance/alpha-survey-iteration-limited-on-wide-splits.md)),
  and the adapt-survey bounds are untested at hundreds of channels
  ([backlog](../backlog/performance/adapt-survey-bounds-untested-at-hundreds-of-channels.md)).

Time-to-accuracy against MadGraph, which these rules feed, is
[performance/integration-vs-madgraph](../performance/integration-vs-madgraph.md).

[^n32-s7]: Note 32 §7: the floor overriding `--neval` on the 579-channel row.
[^n34-s3]: Note 34 S3 close-out: the acceptance-scaled floor and its measurement.
[^n41-m3]: Note 41 M3: the Neyman starvation as found, and the part split.
[^n41-m6]: Note 41 M6: the fix and its measurements.
[^n41-fb]: Note 41 F-B: the floor spend after merging.
[^n41-z]: Note 41 Z close-out: the mixed-row gate's budget.
