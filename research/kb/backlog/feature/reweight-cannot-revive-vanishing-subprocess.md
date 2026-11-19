---
type: Backlog Item
title: "Reweighting to another param card cannot revive a subprocess that vanishes at the generation card"
description: "Groups whose couplings are all zero at the generation card carry no events, so a reweight to a card where they are non-zero (e.g. g2HDM ρ_tu ≠ 0) silently misses them."
area: feature
state: open
priority: low
closes_when: "A reweight request whose target card makes a vanishing group non-zero is refused by name or warned about, with a test."
blocked_by: []
opened: 2026-11-19
tags: [reweighting, param-card, silent-wrong-answer]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-19}
---
Found by K-M1 ([report](../../sprints/v022-smeft-and-refusals/sessions/K-M1-report.md), Found 5).
