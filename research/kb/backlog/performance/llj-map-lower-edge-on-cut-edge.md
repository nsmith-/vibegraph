---
type: Backlog Item
title: The phase-space map's lower edge lands on the fiducial cut edge
description: With a small timelike floor the map edge coincides with the cut boundary and concentrates pp_to_llj's ΔR/pT weight there (m_ll [0,5) var/σ 24.5 → 55.7).
area: performance
state: open
priority: medium
closes_when: pp_to_llj's time-to-accuracy against MadGraph is remeasured under the current map rules, and the remaining cut-edge share is either removed by a map change that passes the floor bias oracle or recorded as not worth a session.
blocked_by: []
opened: 2026-08-06
tags: [phase-space-map, cut-edge, llj, timelike-floor, madevent-parity]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-06}
sources:
  - {id: todo, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L1021-L1035", title: "TODO.md entry T096"}
---
With a small timelike floor, the map's lower edge coincides with the fiducial
boundary and concentrates the residual `ΔR`/`pT` weight there: `pp_to_llj`'s
`m_ll [0,5)` bin goes from var/σ 24.5 to 55.7 post-floor. The time-to-accuracy
remeasure localised `pp_to_llj`'s gap to this. It converges at 0.99× parity with
MadGraph. The whole residual is 9.0× the points for the same accuracy, plus a
stable χ²/dof ≈ 1.4 priced into the stop. Both are signatures of the
cut-boundary map edge, and neither comes from the evaluator.

Part of this is already answered. Confining the lepton pair's decay angle to the
cuts' energy window, and shaping it (`windowed`, `soft-all`), halved llj's
evaluations to 0.1 % (0.50 ± 0.02 of baseline, twenty seeds). That figure
predates any same-host time-to-accuracy comparison with MadGraph under the new
rules.

Next:
1. Remeasure llj's time-to-accuracy against MadGraph under the current map rules.
2. Size what remains. The candidate fixes are bounds in the other cut
   coordinates or a softened lower edge.

The bias oracle for any future floor is
`no_accepted_configuration_sits_below_a_subsystem_floor`. The `mmll = 50` bound
is attained within 1.0002, so it cannot be tightened.

Detail: [note 34 §2 S5 and §3](../../history/notes/34-draw-followup-plan.md),
[note 37 §6](../../history/notes/37-madevent-map-survey-and-soft-angle.md).
