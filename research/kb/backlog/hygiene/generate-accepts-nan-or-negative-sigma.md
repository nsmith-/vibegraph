---
type: Backlog Item
title: "generate refuses an artifact at σ = 0 but not one recording a NaN or negative σ"
description: "LO weights are non-negative, so a NaN or negative recorded σ is corruption; generate checks only σ == 0, and check_support refuses a NaN effective sample size but not a NaN σ."
area: hygiene
state: open
priority: low
closes_when: "generate refuses an artifact whose σ is NaN, infinite or negative, and integrate refuses to write one, each pinned by a test."
blocked_by: []
opened: 2026-11-04
tags: [sigma, refusal, robustness, sprint-g]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-04}
---
G-F4 Found 4. Report: [G-F4](../../sprints/v02-refusals/sessions/G-F4-report.md).
