---
type: Backlog Item
title: LaneField lanes are unmeasured on ARM
description: The wide-backed LaneField has only been timed on x86; the last ARM numbers predate it and showed lanes 2.4–2.9x slower than scalar.
area: performance
state: needs-user
priority: low
closes_when: eval_strategies lanes2/4/8 vs forward per-event ratios for the LaneField build are recorded on an ARM host (e.g. the M3 Max).
blocked_by: []
opened: 2026-09-23
tags: [simd, lanes, arm, neon, bench]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-06}
sources:
  - {id: todo, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L1186-L1210", title: "TODO.md entry T110"}
---
On ARM, `wide` is NEON `f64x2`, and wider packs are pairs of it. The only ARM
lane measurement (M3 Max) used the earlier `NumericArray` lane field, whose
arithmetic failed to inline: best width 2.4–2.9x slower than scalar per event.
That failure was the cause of lanes losing on x86 too, and the `LaneField`
rewrite turned x86 into 1.75x / 3.1x / 4.0x wins at N = 2 / 4 / 8. Whether ARM
follows is unknown.

Needs an ARM host run of `eval_strategies` with `target-cpu=native`; the agent
hosts are x86, so this is a run on the user's machine.

Detail: [x86-avx2-perf-study-results](../../x86-avx2-perf-study-results.md),
"ARM (M3 Max) results" and "The fix: `LaneField<N>` over `wide`".
