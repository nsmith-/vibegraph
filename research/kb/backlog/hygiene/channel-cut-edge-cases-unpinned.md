---
type: Backlog Item
title: "get_channel_cut's tmin factor and a constant scale source's draw flag are unpinned"
description: "At SDE_strategy = 2 with tmin = -1 the tmin factor is believed unreachable but untested; EventScaleSource::constant hard-codes amp2_configuration_weights: false."
area: hygiene
state: open
priority: low
closes_when: "ChannelSet::channel_cuts documents or asserts the unreachable branch, and the constant source derives or documents its flag."
blocked_by: []
opened: 2026-10-10
tags: [sde-strategy, configuration-draw]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-10}
sources:
  - {id: f-e-report, resource: "../../sprints/hygiene/sessions/F-E-report.md", title: "F-E report (hygiene sprint)"}
---
F-E Found 3–4. Separately, MadGraph's own `get_channel_cut` at `sde_strat = 1` with `tmin_for_channel ≠ -1` reads an unassigned `t` (`genps.f:1878-1960` at the pin); vibegraph refuses that card.

Filed at the close-out of the [hygiene sprint](../../sprints/hygiene/sprint.md); evidence in [F-E](../../sprints/hygiene/sessions/F-E-report.md).
