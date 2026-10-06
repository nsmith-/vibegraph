---
type: Backlog Item
title: Lane evaluation can only batch points sharing one αs
description: eval_m2_lanes rescales the constant pools once per call, so a batch must share one αs and a dynamic-scale integrator cannot batch its points.
area: performance
state: open
priority: low
closes_when: A lane-batched evaluation accepts a per-lane αs (the coupling scaling fused into the constant loads) and matches per-point scalar evaluation.
blocked_by: []
opened: 2026-08-06
tags: [simd, lanes, alpha-s, dynamic-scales]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-06}
sources:
  - {id: todo, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L1351-L1353", title: "TODO.md entry T117"}
---
`ScaleAwareAmplitude` (`vibegraph-lib/src/helas/eval/rescale.rs`) moves a bound
amplitude to a new `αs` by rewriting its constant pools (`consts[i] ←
base[i]·rⁿⁱ`). One pool serves the whole `eval_m2_lanes` call, so every lane
shares one `αs`. A lane-batched integrator with dynamic scales would need the
`rⁿⁱ` factor applied per lane, fused into the constant loads.

Nothing needs it today; it becomes a prerequisite once lane batching gets a
dynamic-scale consumer (see
[lane-batching-not-in-production](lane-batching-not-in-production.md)).
