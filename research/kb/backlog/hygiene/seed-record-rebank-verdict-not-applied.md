---
type: Backlog Item
title: "A ladder's re-bank verdict has no command that applies it"
description: "seed-record prints 're-bank at 4x' for a 20 + 20 ladder, but nothing turns the 4x seeds into the record's budget rung, and the SEED_PLAN recipes have not run end to end."
area: hygiene
state: open
priority: low
closes_when: "One documented command turns a ladder's 4x seeds into the record's budget rung (a re-run of seed-record on them, or a record field the gate reads), and the first M3 run of SEED_PLAN=confirm20 or ladder20 is recorded as having produced the record its recipe names."
blocked_by: []
opened: 2026-10-16
tags: [seed-record, ladder, refdata, tooling]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-16}
sources:
  - {id: f1, resource: "../../sprints/seed-record-tooling/sessions/F1-report.md", title: "seed-record-tooling F1 report, Found"}
---
D1's fourth amendment in `sprints/sigma-reference-scatter` says a row whose
20 + 20 ladder shifts beyond 2σ is re-banked at 4×, with its budget rung
becoming the 4× seeds. `seed-record` now prints that verdict
([F1](../../sprints/seed-record-tooling/sessions/F1-report.md)), but the
record's budget rung stays 1× and `bank_seeded_sigma.sh` has no step that
applies it. The driver's `SEED_PLAN=confirm20|ladder20` recipes were checked
only for seed-set resolution, since they need MadEvent on the M3; the first
real run (the 2 → 6 confirmation requested on PR #47) is their test.
