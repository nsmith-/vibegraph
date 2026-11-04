---
type: Backlog Item
title: "CI never runs the no-home-directory CLI tests"
description: "The two cli_pdf_cache tests reach a lookup with no home directory by running the binary as a uid without a passwd entry; on CI's non-root runner they skip, and cargo hides the skip line."
area: hygiene
state: open
priority: low
closes_when: "CI exercises the CLI's no-home-directory PDF lookup (a sudo step, or a route that needs no uid change), and a skipped run is visible as skipped."
blocked_by: []
opened: 2026-11-04
tags: [ci, cache, grid-worker, non-vacuity]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-04}
sources:
  - {id: ghy1, resource: "../../sprints/grid-worker-fixes/sessions/G-HY1-report.md", title: "grid-worker-fixes G-HY1 report, F7 and Found 1"}
---
`cache_root()` falls back to the passwd database, so unsetting `HOME` does
not remove the home directory. Only a uid change does, and that needs
privileges. On CI only the lib test
`no_cache_root_serves_local_tiers_and_refuses_at_the_cache` covers the fix.
The skip message also misnames `PermissionDenied` from an unsearchable binary
path as "may not change user". Related:
[cache-root-resolved-two-ways](cache-root-resolved-two-ways.md).
