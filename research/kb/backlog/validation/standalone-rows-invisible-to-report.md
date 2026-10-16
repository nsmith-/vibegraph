---
type: Backlog Item
title: Standalone amplitude rows have no manifest row and no report cell
description: "The standalone_jamps rows (now over 45) gate in a hermetic test but appear nowhere in validation/manifest.toml or the validation report, so their conventions and deviations live only in test code."
area: validation
state: open
priority: low
closes_when: "Standalone tables have a manifest home (or a standalone-gate entry listing them) and standalone_jamps writes report rows for them."
blocked_by: []
opened: 2026-10-16
tags: [validation-report, manifest, standalone-jamps]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-16}
sources:
  - {id: q1, resource: "../../sprints/amplitude-signs/sessions/Q1-report.md", title: "Q1 report, Found 1"}
---
Related: [standalone-table-without-test-is-never-run](standalone-table-without-test-is-never-run.md).
