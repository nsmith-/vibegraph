---
type: Backlog Item
title: "Hadronic σ cells do not say which MadGraph reference they read"
description: "validate_hadronic's σ cells do not record in their note whether the MadGraph side came from a seed record or the single-run bank; only stderr says so."
area: hygiene
state: open
priority: low
closes_when: "Each validate_hadronic σ cell (single-run, dy13 and MLM) names its MadGraph reference's source in its note, as validate_sigma's cells do."
blocked_by: []
opened: 2026-10-16
tags: [hygiene, validation-report, refdata, seeds]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-16}
---
`validate_sigma` puts the record's description into each cell's note;
`validate_hadronic` prints it to stderr only. A report read after a re-bank
cannot tell which reference a hadronic cell used. Found in the
[F1 report](../../sprints/sigma-reference-scatter/sessions/F1-report.md).
