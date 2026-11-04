---
type: Backlog Item
title: "gen_standalone_jamps.py imports pylhe needlessly, and its standalone build misses param_read.inc edits"
description: "The generator cannot run without pylhe, which it never uses; and standalone output precompiles the model without tracking param_read.inc, so a Source include edit gives a stale, falsely passing row."
area: hygiene
state: open
priority: low
closes_when: "The generator runs without pylhe installed, and a change to a Source include forces a clean model rebuild (a make-clean step), shown by a row that fails on the stale build."
blocked_by: []
opened: 2026-11-04
tags: [tooling, madgraph, sprint-g]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-04}
---
G-F5 Found 4–5; the false pass was seen on `ee_to_mumu_negmz`. Report: [G-F5](../../sprints/v02-refusals/sessions/G-F5-report.md).
