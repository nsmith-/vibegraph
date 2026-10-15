---
type: Measurement
title: "MadEvent's iteration combination reads low"
description: "MadEvent weights VEGAS iterations by mean²/variance, so an iteration that under-samples a peak counts extra and σ reads 0.1–0.2 % low on a fixed-beam 2 → 4; equal-weight recombination lands on this side and an adaptation-free estimator."
status: draft
tags: [madevent-reference, sigma, vegas, iteration-combination, 2to4]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-15}
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-15}]
measured:
  host: "cloud container, 4 vCPU, linux-64, 2026-10-15"
  command: "fixed-beam u u~ > e+ e- g g at √s = 500 GeV (pp_to_ll_0j2j_mlm card at fixed beams, fixed scales, ickkw = 0, xqcut = 0, mmjj = 20); MadEvent pinned 3.7.1 via mg5_pinned.sh, one fresh never-run directory per seed, nb_core 2; vibegraph 2M × 8, -j 2; a hand-written fixed-density estimator linked to MadGraph's standalone matrix.o"
sources:
  - {id: v1, resource: "../sprints/sigma-2to4-offset/sessions/V1-report.md", title: "sigma-2to4-offset V1 report"}
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

## Consequences

- A banked MadEvent σ can read low by about 0.1–0.2 % at default budgets on
  processes with peaked integrands. A σ gate that sizes its tolerance on the
  reference's quoted error alone is exposed to it. The reference error
  design is
  [reference-sigma-seed-scatter-untested](../backlog/validation/reference-sigma-seed-scatter-untested.md).
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
