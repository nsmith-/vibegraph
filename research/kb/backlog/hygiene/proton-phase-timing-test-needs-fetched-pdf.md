---
type: Backlog Item
title: "The proton phase-timing test needs the fetched NNPDF set and so skips the default suite"
description: "a_proton_integration_streams_its_phases_in_order sits behind extended-validation for its PDF set; the stand-in PDF set cli_progress_position.rs writes would let it run by default."
area: hygiene
state: open
priority: low
closes_when: "The test uses the stand-in PDF set and runs in the default cargo test suite, still failing if the phases stream out of order."
blocked_by: []
opened: 2026-11-23
tags: [tests, progress-records, pdf, hermetic]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-23}
sources:
  - {id: ly1, resource: "../../sprints/v023-orders-majorana-timeouts/sessions/L-Y1-report.md", title: "L-Y1 report: Found"}
---
`a_proton_integration_streams_its_phases_in_order` only needs a PDF that
loads; it checks phase order, not PDF values. `cli_progress_position.rs`
already writes a smooth stand-in set, so the test can use it and leave the
`extended-validation` gate.
