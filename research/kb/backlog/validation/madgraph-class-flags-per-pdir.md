---
type: Backlog Item
title: "MadGraph's cut class flags are per P-dir, not per subprocess"
description: "setcuts.f fills is_a_j/is_a_b/is_a_l once per P-dir, so the last iproc's leg classes apply to the whole group; vibegraph classes each subprocess, so class cuts may differ."
area: validation
state: open
priority: medium
closes_when: "A census of grouped P-dirs (banked and campaign processes) where a leg position changes class across iproc, and for any found, σ against MadGraph showing which behaviour vibegraph should follow, recorded in run-card/madgraph-cut-conventions."
blocked_by: []
opened: 2026-11-19
tags: [cuts, madgraph-parity, grouped-subprocesses]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-19}
---
Found by K-C2 ([report](../../sprints/v022-smeft-and-refusals/sessions/K-C2-report.md), Found).
