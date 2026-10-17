---
type: Sprint Record
title: "ee_to_ee_nsz under-samples below the plateau"
description: "The low-budget scatter of e+ e- > e+ e- $$ z is a thin strip just inside the lepton rapidity cut, which the t-channel map's pT-only bound places at its sampling peak; the CLI default reaches the plateau on 20 of 20 seeds, and the map fix is filed."
closed: 2026-10-17
status: draft
tags: [process-grammar, vegas, t-channel, bhabha, sprint-record]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-17}
sources:
  - {id: v1, resource: "sessions/V1-report.md", title: "V1 report"}
  - {id: pr, resource: "https://github.com/nsmith-/vibegraph/pull/46", title: "nsmith-/vibegraph#46"}
---
One PR ([nsmith-/vibegraph#46](https://github.com/nsmith-/vibegraph/pull/46))
closing `ee-ee-nsz-sigma-low-at-low-budget`. Its one session,
[V1](sessions/V1-report.md), ran as validation-dev on Opus.

## What was established

- **The region.** All of the excess scatter is in the t-channel (`ch1`).
  Resampling each seed's frozen grid on 200k points locates it in
  cos θ ∈ [0.9863, 0.986614), |t| from 1673 to 1713 GeV². That strip holds
  about 2.2 % of σ, just inside |η| < 2.5. On the low ×¼ seed it got 31 of
  200k draws and carried 80 % of the variance. The frozen pass over that grid
  reads 157.83 ± 0.93, on the plateau, against the run's 155.675 ± 0.248.
- **The cause.** The map's |t| bound is pT_min² = 100 GeV², not the rapidity
  edge at 1673.3 GeV², which falls at map coordinate 0.640. That is the
  sampling peak. Moving only the bound, through `ptl` 20 and 40 with the
  accepted region unchanged, takes the ×¼ χ²/dof from 3.84 to 1.43 and 0.77,
  with σ unchanged.
- **The CLI default.** It reaches the plateau on every one of 20 seeds:
  157.5771 ± 0.0318 pb, χ²/dof 1.13, worst seed +2.2σ, pull +0.71 against the
  ×16 plateau and +0.76 against refdata-10.
- **The grammar gate is unaffected.** It runs on the plateau, and refdata-10's
  `hard_survey = 2` seeds (157.44–157.62) do not show the default-survey
  record's 156.48.

## Trust

Machine-confirmed by the manager: the rung table and per-channel χ²/dof from
the probe's log, and the CLI's 20-seed mean, error and χ²/dof recomputed from
its log; the commits. The frozen-grid localisation, the falsifier cards and
the fix design are the session's own.

## Filed

- [t-channel-map-ignores-rapidity-cut](../../backlog/performance/t-channel-map-ignores-rapidity-cut.md)
- [vegas-seed-chi2-low-on-clean-cards](../../backlog/validation/vegas-seed-chi2-low-on-clean-cards.md)
