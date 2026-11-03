---
type: Backlog Item
title: "The interned-SM row sweep passes on an empty set"
description: "manifest_rows_naming_the_interned_sm_resolve loads every row whose model is the interned SM and prints the count, but asserts none; two such rows exist since the CKM bank."
area: hygiene
state: open
priority: low
closes_when: "The sweep asserts a row count derived independently of the filter (at least the two ckm_no_b_mass rows), and dropping is_interned_sm's match fails it."
blocked_by: []
opened: 2026-10-26
tags: [tests, non-vacuity, manifest]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-26}
sources:
  - {id: hy1, resource: "../../sprints/ckm-no-b-mass/sessions/HY1-report.md", title: "ckm-no-b-mass HY1 report, H8 and Found 4"}
---
`vibegraph-lib/tests/interned_sm_rows.rs`, `manifest_rows_naming_the_interned_sm_resolve`:
it filters `row_models()` by `is_interned_sm()`, loads each, and prints
`{n} manifest rows name the interned SM`. When written no row named the
interned SM, and HY1 left it with the note that the banking commit should add a
non-zero count. The bank (`866c884`) added `ubx_to_epve_ckm_no_b_mass` and
`bbx_to_tapta_ckm_no_b_mass` (`model = "sm"`), and the assert never came
([HY1](../../sprints/ckm-no-b-mass/sessions/HY1-report.md)).
