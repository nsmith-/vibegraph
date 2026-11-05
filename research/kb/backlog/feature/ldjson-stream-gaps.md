---
type: Backlog Item
title: "An ldjson stream carries unthrottled progress and ends with no final object on a panic"
description: "A 4000-point run emitted 450 progress objects against 239 log objects; and a panic leaves the stream without a result or refusal, so a reader must also watch stderr."
area: feature
state: open
priority: low
closes_when: "The ldjson progress layer has a documented minimum interval per stage, and a panic in ldjson mode writes a refusal of kind run before exiting, each pinned by a CLI test."
blocked_by: []
opened: 2026-11-05
tags: [cli, ldjson, progress, sprint-g]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-05}
---
G-F6 Found 3 and G-HY5 Found 4 ([G-F6](../../sprints/v02-output-modes/sessions/G-F6-report.md), [G-HY5](../../sprints/v02-output-modes/sessions/G-HY5-report.md)). The survey reads panics from stderr today.
