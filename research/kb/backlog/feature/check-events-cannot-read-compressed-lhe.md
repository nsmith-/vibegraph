---
type: Backlog Item
title: "check-events cannot read the .gz and .zst LHE files generate now writes"
description: "generate writes compressed LHE by extension, but check-events reads its input with read_to_string, so it fails on .gz and .zst."
area: feature
state: open
priority: low
closes_when: "check-events decompresses by extension (.gz, .zst), with a CLI test over each."
blocked_by: []
opened: 2026-11-17
tags: [cli, lhef, generate]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-17}
---
Found by sprint K's K-B1 session
(its report, `sessions/K-B1-report.md` in the sprint folder).
`generate` now compresses by output extension; `check.rs` still uses
`std::fs::read_to_string`.
