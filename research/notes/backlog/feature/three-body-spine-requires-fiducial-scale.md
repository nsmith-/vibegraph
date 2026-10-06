---
type: Backlog Item
title: Peripheral spines past two outgoing legs are built only when the cuts supply a fiducial scale
description: With no active single-leg pT cut, a final state of more than two legs gets no spine at all; whether this conservative fallback is still right is unmeasured.
area: feature
state: open
priority: low
closes_when: A measurement on cut-free peripheral processes with more than two outgoing legs either justifies keeping the no-spine fallback or replaces it with a default that builds a spine safely.
blocked_by: []
opened: 2026-08-06
tags: [phase-space, spine, t-channel, conditioning]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-06}
sources:
  - {id: todo, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L814-L821", title: "TODO.md entry T066"}
---
`DiagramChannel::from_diagram_with` (`vibegraph-lib/src/phasespace/diagram_channel.rs:415`)
takes a fiducial scale of zero to mean that a final state of more than two legs gets
no spine, only the all-timelike tree. A scale of zero is what a run with no active
single-leg pT cut gets from `Cuts::spacelike_floor()` (`cuts.rs:640`).

The policy guarded against an unregulated spine whose collinear edge
(`t_max = 0`) sits on rounding noise. That defect makes the estimator biased, not
merely noisy. Two conditioning fixes now remove most of it: a grouped Källén form,
and boosts with `γ = E/√s` (module docs, `diagram_channel.rs:55-85`). With those,
the massless edge is an exact analytic zero whenever the emitted subsystem has a
fixed invariant. Only a **composite emitted subsystem** still puts the edge on
either side of zero at rounding scale.

So the open question is whether the no-spine fallback should be relaxed to "build
the spine unless the emitted side is composite". Settling it means measuring
variance and bias against flat RAMBO on cut-free peripheral processes, with the
density walk check (`WALK_DENSITY_TOL`) as the bias oracle. Related: the D3 decision
and its measurements in [note 28 §S2.5, §S4](../../28-kt-spine-feature-sprint-plan.md).
