---
type: Backlog Item
title: "The internal spin-3/2 refusal has no test, and the duplicate-subprocess check gives spin 3/2 an empty state set"
description: "Sprint G's spin refusal also refuses internal spin 3/2, untested; summed_states gives a spin-3/2 leg no states, so two duplicate gravitino subprocesses look disjoint."
area: hygiene
state: open
priority: medium
closes_when: "A spin-3/2 propagator on a toy gravitino vertex is refused at compile with a named error, pinned by a test; and summed_states either refuses spin 3/2 or lists its states, with a test."
blocked_by: []
opened: 2026-11-04
tags: [spin, refusal, sprint-g]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-04}
---
G-F1 Found 1–2. `diagrams/mod.rs` `summed_states` uses `helicity_states().unwrap_or_default()`; compile refuses those processes today, so it is latent. Report: [G-F1](../../sprints/v02-refusals/sessions/G-F1-report.md).
