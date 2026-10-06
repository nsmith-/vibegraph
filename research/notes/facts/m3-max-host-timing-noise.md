---
type: Measurement
title: Run-to-run timing spread on the M3 Max performance host
description: "The performance layer's run-to-run spread is 0.8% median / 3.4% worst on rows above 1 s, so sub-1% claims are not measurable there; a noisy host costs ~12.7%."
tags: [performance, measurement-method]
status: draft
measured:
  host: "M3 Max, macOS"
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-06}
sources:
  - {id: todo, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L62-L74", title: "TODO.md standing measurement facts"}
---

The performance layer's numbers live in the notes: the
[note 30](../30-perf-baseline-timings.md) baseline and the close-outs in
[note 31 §6](../31-perf-sprint-3-plan.md) and
[note 32 §5](../32-perf-addendum-plan.md) carry the per-category timings, the
per-point ratios against MadGraph and the parallel-scaling figures, all on one
M3 Max host.

Two properties of that host matter when reading a new measurement against
them:

- the layer's own run-to-run spread is **0.8% median / 3.4% worst** on rows
  above 1 s, so a sub-1% claim is not measurable there;
- a noisy host costs **~12.7%**, which is why CPU time is the more reliable
  instrument than wall time.
