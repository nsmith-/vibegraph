---
type: Backlog Item
title: "CLI stale-artifact tests decode only because bincode ignores trailing bytes"
description: "cli_generate and cli_generate_proton stamp old format_version numbers onto current-schema bytes; the old readers accept them only because trailing bytes are ignored."
area: hygiene
state: open
priority: low
closes_when: "The tests build real old-schema fixtures, and write_to_path refuses a version below what the channels need."
blocked_by: []
opened: 2026-10-10
tags: [artifact, tests]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-10}
sources:
  - {id: f-f-report, resource: "../../sprints/hygiene/sessions/F-F-report.md", title: "F-F report (hygiene sprint)"}
---
F-F Found 2.

Filed at the close-out of the [hygiene sprint](../../sprints/hygiene/sprint.md); evidence in [F-F](../../sprints/hygiene/sessions/F-F-report.md).
