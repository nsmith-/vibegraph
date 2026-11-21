---
type: Backlog Item
title: "Each channel's configuration-share vector spans every configuration of the layout"
description: "After #155 the share history is bounded, but each of 1068 channels on GJ-4Jets still holds 24,297 shares (~400 MiB fixed); the cost grows as channels × configurations."
area: performance
state: open
priority: low
closes_when: "Each channel's shares and tiles are sized to its own configuration range, peak RSS on the GJ-4Jets card is measured before and after, and σ is bit for bit."
blocked_by: []
opened: 2026-11-19
tags: [performance, memory, multichannel, mlm]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-19}
---
Found by K-G1 ([report](../../sprints/v022-smeft-and-refusals/sessions/K-G1-report.md), Found 2).
