---
type: Backlog Item
title: "With no run card, mixed-multiplicity jet cards run unmatched where MadGraph turns on MLM"
description: "For p p > e+ e- plus add process p p > e+ e- j, MadGraph's default card sets ickkw = 1, xqcut = 30, drjl = 0 and --alps; vibegraph keeps ickkw = 0, a silent difference in σ and events."
area: validation
state: open
priority: high
closes_when: "The absent-run-card default follows create_default_for_process's matching rule for mixed jet multiplicities, a test pins ickkw/xqcut/drjl on one such card against MadGraph's written card, and a σ row with no run card matches MadGraph by the multi-seed criterion."
blocked_by: []
opened: 2026-11-05
tags: [run-card, defaults, mlm, madgraph-parity, silent-wrong-answer]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-05}
sources:
  - {id: hj1, resource: "../../sprints/cms-gridpack-coverage/sessions/H-J1-report.md", title: "H-J1 report, Found 2"}
---
Found by sprint H's H-J1. For `p p > e+ e-` with `add process p p > e+ e- j`
and no run card, MadGraph's `output` writes `ickkw = 1`, `xqcut = 30`,
`drjl = 0`, and adds `--alps` to the systematics arguments. vibegraph's
absent-card default keeps `ickkw = 0`, so the card runs unmatched and its σ and
events differ, with nothing to show for it.

The fix belongs in `RunCard::scattering_default`.
