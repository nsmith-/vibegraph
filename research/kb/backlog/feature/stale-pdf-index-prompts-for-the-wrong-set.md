---
type: Backlog Item
title: "A stale local pdfsets.index can prompt to download the wrong set"
description: "try_table locates each index's candidate through every tier, download included, so an old local index whose candidate is an older set not on disk asks to fetch that set first."
area: feature
state: needs-user
priority: low
closes_when: "Local indices are read in a first pass that locates candidates without the download tier, a download is offered only for the current index's candidate, and a test pins the order through the injected fetcher."
blocked_by: []
opened: 2026-10-26
tags: [pdf, lhapdf, network-consent, index]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-26}
sources:
  - {id: f1, resource: "../../sprints/pdf-from-run-card/sessions/F1-report.md", title: "pdf-from-run-card F1 report, Found 2"}
  - {id: f2, resource: "../../sprints/pdf-from-run-card/sessions/F2-report.md", title: "pdf-from-run-card F2 report, Found 1"}
---
`try_table` (`vibegraph-lib/src/pdf/select.rs`) takes an index's candidate set
for an lhaid and calls `locate_set` with every tier, the consented download
included. When a local `pdfsets.index` predates a newer set, its candidate for
the newer id is an older set whose range cannot be checked without that set's
`.info`. If that older set is not on disk, the lookup asks to download it before
the next, fresher index is read
([F1](../../sprints/pdf-from-run-card/sessions/F1-report.md),
[F2](../../sprints/pdf-from-run-card/sessions/F2-report.md)).

Which index counts as current is a consent-flow choice, so this needs the user
before a session starts.
