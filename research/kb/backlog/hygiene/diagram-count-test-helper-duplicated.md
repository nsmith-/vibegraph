---
type: Backlog Item
title: The diagram-count test helper is written four times
description: "diagram_count (toy_models.rs, smeftsim.rs, identical bodies), total_diagrams (diagrams.rs) and a local try-generate in weighted_search.rs do the same thing; tests/common should hold one."
area: hygiene
state: open
priority: low
closes_when: "tests/common provides diagram_count and try_generate (returning the Result), and no test file re-implements them."
blocked_by: []
opened: 2026-10-20
tags: [tests, duplication, diagrams]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-20}
sources:
  - {id: hy1, resource: "../../sprints/weighted-cap/sessions/HY1-report.md", title: "weighted-cap HY1 report, H10"}
---
Sites: `vibegraph-lib/tests/toy_models.rs` ~:59, `tests/smeftsim.rs` ~:48,
`tests/diagrams.rs` ~:7, and `tests/weighted_search.rs`. Found in the
[weighted-cap hygiene review](../../sprints/weighted-cap/sessions/HY1-report.md).
