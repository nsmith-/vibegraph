---
type: Backlog Item
title: Long-tier gate runs carry no record of what was run
description: "Long-tier cells, which CI never runs, are filled from report rows that record no commit, tree state, refdata checksum, command or seeds, so a green cell rests on the runner's word."
area: validation
state: open
priority: medium
closes_when: "Every long-tier gate writes a receipt (commit, clean tree, refdata pin and checksum, exact command, seeds) beside its report rows, and the collator marks a long-tier cell green only when a receipt for the current manifest pin backs it."
blocked_by: []
opened: 2026-10-09
tags: [validation, report, long-tier, evidence]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: user, resource: "https://github.com/nsmith-/vibegraph/pull/18", title: "User decision dropping note 42 Phase 5, 2026-10-09"}
---
CI runs the hermetic and banked layers on every PR, so their cells are
evidenced by the PR's checks. The long tier (`validate-deep`: the 2→6 σ rows,
the kT-cluster replay, the MLM σ and dump gates, the Pythia comparison) runs
by hand or in agent sessions, and its cells are filled from the per-cell JSON
under `target/validation-report/` (`vibegraph-lib/tests/common/report.rs`).
Those rows record what was measured and `host.json` records the machine, but
nothing records which code, data and command produced them.

To close it, extend the row writer with a run record: commit, whether the tree
was clean, the refdata pin and its SHA-256, the exact pixi command, and the
seeds or budgets of statistical gates. Have the collator refuse to render a
long-tier cell green without a record matching the manifest's current pin.
A small check script can then confirm an agent's "the long-tier gate passed"
from the files alone ([AGENTS.md](../../../../AGENTS.md), "Subagent reports are
evidence, not truth"; [validation report](../../validation/validation-report.md)).

This is the useful remainder of the dropped attested-computation phase
([OKF migration record](../../sprints/okf-migration.md)); it needs no OKF
machinery.
