---
type: Backlog Item
title: "A SIGKILLed generate leaves its hidden .partial file behind"
description: "generate writes to .<name>.<pid>.partial and renames on success; a killed run leaves the partial file, and nothing cleans stale ones up."
area: feature
state: open
priority: low
closes_when: "generate removes or reports stale .<name>.*.partial files for its own output name at start, with a test; or the behaviour is documented as the user's to clean."
blocked_by: []
opened: 2026-11-17
tags: [cli, generate, batch]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-17}
---
Found by sprint K's K-B1 session
(its report, `sessions/K-B1-report.md` in the sprint folder).
