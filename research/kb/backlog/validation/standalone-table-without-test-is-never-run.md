---
type: Backlog Item
title: A banked standalone table with no #[test] is never compared
description: "Nothing ties validation/madgraph/standalone/*.json to the tests in standalone_jamps.rs; a table banked without a test (as W1's eleven probe tables are today) is silently unchecked."
area: validation
state: open
priority: low
closes_when: "standalone_jamps.rs fails when a table under validation/madgraph/standalone/ has no test (or a declared informational listing), e.g. from one ROWS list the directory check and the tests both read."
blocked_by: []
opened: 2026-10-15
tags: [standalone-jamps, coverage, non-vacuity]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-15}
sources:
  - {id: gh, resource: "../../sprints/amplitude-signs/sessions/G-H-report.md", title: "G-H report, Found 4"}
---
Found by the hygiene review of the `t t~ + jet` gate. A row added to
`gen_standalone_jamps.py` and banked without a matching `check()` test reaches
no comparison and no report cell.
