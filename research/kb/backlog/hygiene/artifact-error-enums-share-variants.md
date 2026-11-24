---
type: Backlog Item
title: "ArtifactError and DiagramArtifactError share ten variants"
description: "The two artifact error enums duplicate ten header and digest variants, so the CLI's error.rs maps each twice."
area: hygiene
state: open
priority: low
closes_when: "One shared header-and-digest error type serves both artifacts."
blocked_by: []
opened: 2026-11-24
tags: [artifact, errors, duplication]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-24}
sources:
  - {id: report, resource: "../../sprints/v023-orders-majorana-timeouts/sessions/L-Z5-report.md", title: "L-Z5 report: Found 6"}
---
Found by sprint L's hygiene pass L-Z5 (Found 6); the evidence is in
[its report](../../sprints/v023-orders-majorana-timeouts/sessions/L-Z5-report.md).
