---
type: Backlog Item
title: "vibegraph's matched @1 files scatter beyond their quoted errors"
description: "On today's twenty pp_to_ll_0j2j_mlm samples, the per-file @1 Pythia acceptance scatters at χ²/dof 1.87 on 19 dof (0.88 on the 2026-10-01 samples), and the vibegraph null's @1 pull reads −2.96."
area: validation
state: open
priority: medium
closes_when: "The extra @1 scatter is attributed (the <scales> change of PR #23, overweight files, or the gate's error = quoted), and the user decides whether a per-file χ²/dof band joins the MLM Pythia gate."
blocked_by: []
opened: 2026-10-17
tags: [mlm, pythia, matching, scatter, error-calibration]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-17}
sources:
  - {id: v1, resource: "../../sprints/mlm-pythia-tolerance/sessions/V1-report.md", title: "mlm-pythia-tolerance V1 report, Found 1"}
  - {id: hy1, resource: "../../sprints/mlm-pythia-tolerance/sessions/HY1-report.md", title: "mlm-pythia-tolerance HY1 report, Found 5"}
---
Per-file χ²/dof about the mean on today's vibegraph samples is `@0` 1.54,
`@1` 1.87 (19 dof, p ≈ 0.01) and `@2` 0.59. The 2026-10-01 samples read
0.75 / 0.88 / 0.92, and MadEvent reads 0.65 / 0.96 / 0.66 today. The
vibegraph-against-vibegraph null's `@1` pull is −2.96, against +0.23 before
([V1](../../sprints/mlm-pythia-tolerance/sessions/V1-report.md)).

- **Candidate causes.** PR #23 (`9e141de`) changed what is written in
  `<scales>` after the old record. Separately, two files carry `@1` errors of
  0.0079 and 0.0069 against 0.0050 typical.
- **Why the gate is blind to it.** The gate takes each side's error as the
  quoted one, which is the same fact as
  [mlm-match-py-error-not-seed-policy](mlm-match-py-error-not-seed-policy.md).
- **Effect on the verdict.** If the scatter is real, the B-vs-A `@1` pull
  reads about −1.5 instead of −1.79, so the verdict holds.
