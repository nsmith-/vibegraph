---
type: Backlog Item
title: "The cache root and VIBEGRAPH_UFO_DIR are resolved two ways"
description: "locate_from_env has an unused None branch, and resolve_ufo_search_path reads $VIBEGRAPH_UFO_DIR itself rather than through locate_from_env."
area: hygiene
state: open
priority: low
closes_when: "One resolution path reads each environment variable."
blocked_by: []
opened: 2026-10-10
tags: [cache, consistency]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-10}
sources:
  - {id: r-f-report, resource: "../../sprints/hygiene/sessions/R-F-report.md", title: "R-F report (hygiene sprint)"}
---
R-F.28.

Filed at the close-out of the [hygiene sprint](../../sprints/hygiene/sprint.md); evidence in [R-F](../../sprints/hygiene/sessions/R-F-report.md).
