---
type: Measurement
title: "MadEvent's iteration combination reads low"
description: "MadEvent weights VEGAS iterations by mean²/variance, so an iteration that under-samples a peak counts extra and σ reads 0.1–0.25 % low on fixed-beam 2 → 4 and 2 → 3 rows; adaptation-free estimators land on its equal-weight recombination."
status: draft
tags: [madevent-reference, sigma, vegas, iteration-combination, 2to4, 2to3]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-15}
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-15}, {by: claude-code/claude-opus-5-5, at: 2026-10-17}]
measured:
  host: "cloud container, 4 vCPU, linux-64, 2026-10-15"
  command: "fixed-beam u u~ > e+ e- g g at √s = 500 GeV (pp_to_ll_0j2j_mlm card at fixed beams, fixed scales, ickkw = 0, xqcut = 0, mmjj = 20); MadEvent pinned 3.7.1 via mg5_pinned.sh, one fresh never-run directory per seed, nb_core 2; vibegraph 2M × 8, -j 2; a hand-written fixed-density estimator linked to MadGraph's standalone matrix.o"
sources:
  - {id: v1, resource: "../sprints/sigma-2to4-offset/sessions/V1-report.md", title: "sigma-2to4-offset V1 report"}
  - {id: v2-2to3, resource: "../sprints/sigma-pulls-lean-high/sessions/V2-report.md", title: "sigma-pulls-lean-high V2 report: the 2 → 3 fixed-density estimate"}
  - {id: mg-dsample, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/Template/LO/Source/dsample.f#L1995-L1997", title: "MadEvent dsample.f: iterations accumulated with weight mean²/variance"}
---
# MadEvent's iteration combination reads low

MadEvent combines its VEGAS iterations with the weights `mean**2/sigma`, where
`sigma` is that iteration's variance (`dsample.f:1995-1997` at the pin). If an
iteration under-samples a peak, it reads both low and precise, so it counts
extra. The combined σ is biased low, and the bias shrinks as the points per
iteration grow. This is the same failure `AGENTS.md` names for a `1/σ²`
combination of seeds, here inside one MadEvent run.

## The measurement

The reproducer is fixed-beam `u u~ > e+ e- g g` at √s = 500 GeV. |M|² agrees
point by point to 3e-14, and every event of both samples passes the card's
cuts.

| estimate | σ (pb) | seeds |
|---|---|---|
| MadEvent as reported | 0.527760 ± 0.000170 | 60 fresh directories |
| MadEvent, 10× events | 0.527975 ± 0.000155 | 10 |
| MadEvent, 10× events, iterations re-combined with equal weight | 0.528511 ± 0.000145 | 10 |
| vibegraph, 2M × 8 | 0.528663 ± 0.000052 | 10 |
| vibegraph, 8M × 8 | 0.528643 ± 0.000049 | 5 |
| fixed-density estimator, never adapted | 0.528408 ± 0.000130 | 46, χ²/dof 1.10 |

- **Paired per seed**, MadEvent's equal-weight mean minus its reported σ:
  - +0.18 % (4.1σ) on 60 default seeds, again on 36 more;
  - +0.10 % (4.9σ) at 10× events;
  - +0.09 % with `hard_survey = 2`;
  - zero on the `m(ll) > 150` slice.
- **Slices** in `m(ll)` and `m(gg)` close to the inclusive σ on both sides. No
  slice shows a region one side misses.
- **Verification:** the manager recomputed the reported and vibegraph means
  from the raw per-seed logs. The paired numbers are the session's
  measurements.

## The 2 → 3 case: `q q̄ > e⁺ e⁻ g` (2026-10-17)

The banked fixed-beam rows `uux_to_epemg` and `ddx_to_epemg`, read against
their refdata-10 seed records (five fresh-directory MadEvent seeds,
`hard_survey = 2`, 10 000 events), with this crate's own |M|² and cuts.
The adaptation-free estimate is `probe_fixed_density_llj_annihilation`
(`validate_sigma.rs`, `#[ignore]`d). It draws each point from an analytic
density fixed before the first point, with no iteration and no grid
refinement, and reports the plain mean of f/p over 20 seeds × 3M points
(seed χ²/dof 0.60 and 0.53 on 19 dof; 1.02 at 2000 × 10k).

| row | adaptation-free σ (pb) | pull vs MadEvent reported | pull vs MadEvent equal-weight | pull vs vibegraph, 5 seeds |
|---|---|---|---|---|
| `uux_to_epemg` | 0.5564310 ± 0.0000915 | +3.01 | +0.45 | −0.74 |
| `ddx_to_epemg` | 0.6203869 ± 0.0001118 | +2.67 | −0.19 | −0.28 |

- MadEvent's reported value reads 0.20–0.25 % low on both rows at
  `hard_survey = 2` and 10 000 events. On `uux` the estimate minus reported is
  +0.001377 ± 0.000457 pb, against MadEvent's own paired equal-weight minus
  reported of +0.001123 ± 0.000324 on the same seeds.
- A second density (the production mixture, adapted once on a survey and then
  frozen, with no VEGAS grid) agrees within 1.03σ on both rows.
- **Verification:** the manager read the pulls above from the run log
  (`sigma-pulls-lean-high` V2, the 20 × 3M run). The density's design and
  its calibration runs are the session's.

## Consequences

- A banked MadEvent σ can read low by about 0.1–0.2 % at default budgets on
  processes with peaked integrands. A σ gate that sizes its tolerance on the
  reference's quoted error alone is exposed to it. The reference error
  design is
  [D1](../sprints/sigma-reference-scatter/decisions/D1-reference-error-design.md):
  seeded fresh-directory references, banked in refdata-10.
- MadEvent's unweighted samples also under-populate high-weight regions. On
  `m(ll) > 150` they are 5.4 % short of the run's own sliced σ, and still 4σ
  short with the final truncation disabled. A reference built as inclusive σ ×
  event fraction inherits that. Not located in code.
- It is a MadEvent property, recorded in
  [madgraph-defects](madgraph-defects.md). It is never reproduced on this side.

## What would overturn it

- An estimator with an honest error agreeing with MadEvent's reported value.
- A second fixed-density parametrisation disagreeing with this one.
- The paired shift not shrinking as the points per iteration grow.
