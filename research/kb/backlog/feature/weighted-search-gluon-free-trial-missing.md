---
type: Backlog Item
title: The WEIGHTED search does not run MadGraph's gluon-free trial
description: "MadGraph tries each WEIGHTED bound on the process with final-state gluons removed; vibegraph tries the full process, so on SMEFTsim e+ e- > z g g it returns a diagram MadGraph refuses."
area: feature
state: open
priority: medium
closes_when: "The automatic WEIGHTED search reproduces diagram_generation.py:2040-2059 (final-state gluons in the lowest class removed from each trial, the empty-final-state case included), and the ignored a_final_state_gluon_pair_without_a_qcd_vertex_is_refused test runs and passes."
blocked_by: []
opened: 2026-10-20
tags: [diagrams, coupling-orders, weighted, madgraph-parity, smeftsim]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-20}
sources:
  - {id: v1, resource: "../../sprints/weighted-cap/sessions/V1-report.md", title: "weighted-cap V1 report, Found 1 and its follow-up"}
---
MadGraph's `find_optimal_process_orders` (`diagram_generation.py:2040-2059`
at b7687064) generates each trial with the final-state gluons removed (at the
bound minus one per removed gluon). On SMEFTsim `restrict_SMlimit_massless`,
`e+ e- > z g g` therefore stops at 6 and MadGraph refuses; vibegraph climbs
to 107 and returns one diagram (Z* → Z h*, h* → g g via SMHLOOP); `vg_cG` and
`vg_cHG` behave the same. The gap predates the
[weighted-cap sprint](../../sprints/weighted-cap/sprint.md), which pinned it
as an ignored test. MadGraph's trial also builds legs from particle ids only,
dropping polarizations (`g0_g_ttx` reports bound 2 against our 4), which can
change only the bound quoted in a refusal.

MadGraph's refusal is confirmed by a run: on the M3, `e+ e- > z g g` raises
`NoDiagramException ... WEIGHTED=6` at the first trial
([#64](https://github.com/nsmith-/vibegraph/pull/64#issuecomment-6320700423)).
