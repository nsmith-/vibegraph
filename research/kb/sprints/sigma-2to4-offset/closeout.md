---
type: Sprint Record
title: "2 → 4 σ offset against MadEvent"
description: "The +0.17 % fixed-beam 2 → 4 offset is MadEvent's mean²/variance iteration combination reading low; equal-weight recombination and an adaptation-free estimator agree with vibegraph. Nothing to fix here."
closed: 2026-10-15
status: draft
tags: [sigma, madevent-reference, 2to4, sprint-record]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-15}
sources:
  - {id: sprint, resource: "sprint.md", title: "Sprint overview"}
  - {id: log, resource: "log.md", title: "Sprint log"}
  - {id: v1, resource: "sessions/V1-report.md", title: "V1 report"}
  - {id: pr, resource: "https://github.com/nsmith-/vibegraph/pull/22", title: "nsmith-/vibegraph#22"}
---
One PR ([nsmith-/vibegraph#22](https://github.com/nsmith-/vibegraph/pull/22))
closing `generic-2to4-sigma-offset-vs-madevent` by attribution to MadEvent.
Chronology in [log.md](log.md).

## What was established

- **The offset belongs to MadEvent.** With 60 fresh directories it is +0.17 %
  (4σ), not the item's +0.29 %. MadEvent's mean²/variance iteration
  combination reads low. Equal-weight recombination of the same runs lands
  on vibegraph and on a fixed-density estimator that is never adapted (46
  seeds, χ²/dof 1.10). The paired shift is 4–10σ on every independent set,
  and it shrinks with budget
  ([madevent-iteration-weighting-bias](../../validation/madevent-iteration-weighting-bias.md)).
- **The item's tail surplus was a misreading.** It came from MadEvent's
  unweighted event sample, which is 5.4 % short above `m(ll) = 150`. Sliced σ
  shows no region either side misses.
- **This side is stable.** The split-angle maps, a 4× budget and slices in
  `m(ll)`/`m(gg)` all agree with the estimator within 1.7σ.
- **The closing condition** reads "attributed to one side and fixed there".
  MadEvent cannot be fixed here, so the attribution is recorded as a measured
  MadEvent property in [madgraph-defects](../../validation/madgraph-defects.md).
  Whether banked references or σ gates should correct for it is a design
  question. It goes to `sigma-reference-scatter`'s D1, which owns the
  reference-error design.

## Trust

- V1 is machine-confirmed by the manager: the commit; the `dsample.f` lines
  at the pin; the reported MadEvent and vibegraph means, recomputed from the
  raw logs.
- The paired re-combination and the estimator are the session's measurements.

## Filed

- [madevent-unweighted-sample-tail-deficit](../../backlog/validation/madevent-unweighted-sample-tail-deficit.md)
- [vegas-quoted-error-low-on-mgg-slices](../../backlog/validation/vegas-quoted-error-low-on-mgg-slices.md)
- [madevent-seed-drivers-drop-failed-compiles](../../backlog/hygiene/madevent-seed-drivers-drop-failed-compiles.md)
