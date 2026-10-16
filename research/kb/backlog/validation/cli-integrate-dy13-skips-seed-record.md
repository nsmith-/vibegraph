---
type: Backlog Item
title: "cli_integrate reads the dy13 references past the seed record"
description: "cli_integrate.rs reads the dy13 σ references straight from hadronic_sigma_reference.json, so a banked dy13 seed record never reaches it."
area: validation
state: open
priority: low
closes_when: "cli_integrate's dy13 comparisons read the seeded σ record when the bundle carries one, as validate_sigma and validate_hadronic do."
blocked_by: []
opened: 2026-10-16
tags: [validation, refdata, seeds, cli]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-16}
---
`vibegraph-cli/tests/cli_integrate.rs` (`banked`) reads `dy13_default` and
`dy13_mmll_60_120` from `hadronic_sigma_reference.json`. The σ gates read a
row's `output/seeded_sigma/<row>/seed_record.json` where the bundle has one;
this test would keep comparing against the single-run value. To resolve:
include `vibegraph-lib/tests/common/seed_record.rs` by path, as the CLI tests
already include `seeds.rs`. Found in the
[F1 report](../../sprints/sigma-reference-scatter/sessions/F1-report.md).
