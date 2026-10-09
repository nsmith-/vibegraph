---
type: Backlog Item
title: Complex::mul_add for the lane field is unmeasured
description: Complex primitives share one real-FMA path because the lane field lacked num_traits::MulAdd; whether Complex::mul_add beats it on x86 and ARM is unmeasured.
area: performance
state: open
priority: low
closes_when: num_traits::MulAdd is implemented for LaneField<N> and Complex::mul_add is measured against the shared real-FMA path on x86 and ARM, with the faster one kept and lane-vs-scalar identity intact.
blocked_by: []
opened: 2026-08-01
tags: [simd, lanes, fma, complex, arm]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-06}
sources:
  - {id: todo, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L1173-L1185", title: "TODO.md entry T109"}
---
`cmul`/`cmul_add` in `vibegraph-lib/src/helas/repr/lorentz.rs` express the
complex multiply-add through the real `Real::mul_add_fast`, one code path for
every `F: Real`. It exists because the lane type had no `num_traits::MulAdd`,
so `Complex::mul_add` was unavailable there. When introduced it cost scalar
`forward` +3.5% while speeding lanes 22–35%, and the scalar path is the
production one.

The lane field is now the local `LaneField<N>`
(`vibegraph-lib/src/helas/eval/lane_field.rs`), so implementing
`num_traits::MulAdd` for it is no longer an orphan-rule problem, and
`Complex<LaneField<N>>` can get `Complex::mul_add`. Whether that beats the
shared path is a measurement on both x86 and ARM.

Constraints: the shared path is also what keeps every lane bit-identical to
scalar (`eval_m2_lanes_match_scalar`); a replacement must keep that. The
packed-complex idiom is x86-specific: forced on ARM it cost 8–9%, which killed
the in-house workaround trait. Its design stays at
[note 32 §2 S9](../../history/notes/32-perf-addendum-plan.md).
