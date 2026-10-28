---
type: Backlog Item
title: Peripheral spines past two outgoing legs are built only when the cuts supply a fiducial scale
description: With no active single-leg pT cut, a final state of more than two legs gets no spine at all; whether this conservative fallback is still right is unmeasured.
area: feature
state: open
priority: high
closes_when: "Peripheral processes with more than two outgoing legs get spine channels when the card has no single-leg pT cut, with a scale taken from cuts the card does set (e.g. mmjj) or a safe default, and no bias on the density walk check (WALK_DENSITY_TOL). The three CMS VBF/EWK Drell-Yan gridpacks below then agree with MadGraph on a 5-seed ladder whose medians have stopped moving, with χ²/dof near 1."
blocked_by: []
opened: 2026-08-06
tags: [phase-space, spine, t-channel, conditioning, cms-survey, vbf]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-06}
sources:
  - {id: pr71-sweep, resource: "https://github.com/nsmith-/vibegraph/pull/71#issuecomment-6438701885", title: "PR #71: the VBF/EWK Drell-Yan seed × budget sweep, traced to this fallback"}
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
and its measurements in [note 28 §S2.5, §S4](../../history/notes/28-kt-spine-feature-sprint-plan.md).

**Production cards hit it (2026-10-27).** Three CMS VBF/EWK l+l- jj gridpacks
(`DYto2L-2Jets-EWK_Bin-MLL-50`, `VBFto2L_Bin-MLL-50`, `VBFto2L_Bin-MLL-4to50`) set
`ptj = ptl = 0` and cut only on `mmjj`. Every run logs "0 peripheral, 36/40
all-timelike". The 5-seed median σ climbs from 0.22–0.62 of MadGraph at 10k × 3 to
0.62–0.81 at 160k × 3, with seed χ²/dof 3–92 and single seeds 1.3–3.7× high: a
heavy-tailed, confidently wrong estimate. The same process class with `ptj = 10`
(`ewkzlljj_5f_LO`) builds 56 peripheral channels of 68 and reads 1.005. The
massless and τ mass parts are equally low, so #72's split is not involved; no
matrix-element, CKM-restriction or PDF cause was found.
On these cards the fallback is not conservative; it is wrong, so the question
above is now a fix.
