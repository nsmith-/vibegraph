---
type: Backlog Item
title: "A per-event scale error on a fixed-beam sampled point panics"
description: "hadronic.rs apply_scale unwraps event_scales_of with panic!; #153 made the proton path's equivalent a named SampledPointFault refusal, the fixed-beam path still panics."
area: validation
state: open
priority: high
closes_when: "A scale error on a fixed-beam sampled point gives that point zero weight and integrate/generate refuse by name (the SampledPointFault path the proton integrand uses), with a failing-first test; fixed-beam and hadronic banked cards stay bit for bit."
blocked_by: []
opened: 2026-11-19
tags: [panic, refusal, fixed-beam, scales]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-19}
---
`FixedBeamIntegrand::apply_scale` (`vibegraph-lib/src/hadronic.rs`, about
line 2746 at f78fb60b) calls
`event_scales_of(...).unwrap_or_else(|e| panic!("per-event scale on a sampled point: {e}"))`.
A panic is never an acceptable outcome. The proton path's sites became
`SampledPointFault` records in #153; this one was left.

Found by K-M1 ([report](../../sprints/v022-smeft-and-refusals/sessions/K-M1-report.md), Found 4).
