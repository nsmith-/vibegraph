---
type: Backlog Item
title: "The amplitude probe never re-checks a 2 → 1 matrix element"
description: "amplitude_probe.rs:162 returns no points for fewer than two outgoing legs and check_with skips an empty probe, so a p p > z matrix element is never re-checked."
area: validation
state: open
priority: medium
closes_when: "The probe evaluates a 2 → 1 process at √ŝ = M at rest, a p p > z row is checked against its reference, and a mutation of the one-body amplitude fails it."
blocked_by: []
opened: 2026-11-19
tags: [amplitude-probe, one-body, oracle]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-19}
---
Found by K-H4 (hygiene pass on #142 and #155, PR #158), Found 1. The one-body final state (#142) is integrated, but its matrix element has no probe oracle.
