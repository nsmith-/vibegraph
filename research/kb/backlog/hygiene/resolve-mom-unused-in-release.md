---
type: Backlog Item
title: "NodeAnalysis::resolve_mom is unused in release builds"
description: "helas/eval/analysis.rs:208 warns never-used in a release-debug build of vibegraph; clippy --all-targets is clean because tests use it."
area: hygiene
state: open
priority: low
closes_when: "resolve_mom is either given a production caller or made test-only, and the release-debug build is warning-free."
blocked_by: []
opened: 2026-11-24
tags: [helas, dead-code]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-24}
sources:
  - {id: report, resource: "../../sprints/v023-orders-majorana-timeouts/sessions/L-Z8-report.md", title: "L-Z8 report: Found"}
---
Found by sprint L's hygiene pass L-Z8 (Found); the evidence is in
[its report](../../sprints/v023-orders-majorana-timeouts/sessions/L-Z8-report.md).
