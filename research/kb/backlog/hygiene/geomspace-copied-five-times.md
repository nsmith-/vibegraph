---
type: Backlog Item
title: "geomspace is copied in five test and bench files"
description: "The log-spaced grid helper is written out in interp.rs and extrap.rs tests, the pdf_xfx bench, table_oracle.rs and tests/pdf_member_bytes.rs."
area: hygiene
state: open
priority: low
closes_when: "One shared test helper provides geomspace to the lib tests, the integration tests and the bench, or the copies are recorded as intended."
blocked_by: []
opened: 2026-11-04
tags: [duplication, tests, pdf]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-04}
sources:
  - {id: mhy1, resource: "../../sprints/cms-pdf-memory/sessions/M-HY1-report.md", title: "cms-pdf-memory M-HY1 report, F6"}
---
The copies span three kinds of target: the lib's `cfg(test)` modules, an
integration test and a bench. A shared helper needs a home all three can
reach, such as a `#[doc(hidden)]` module behind a test-support feature.
