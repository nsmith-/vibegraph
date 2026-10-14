---
type: Backlog Item
title: "IsBra and SpinorRepr have single implementors and LorentzRepr is never a bound"
description: "The traits act as method namespaces over Bispinor<F, Bra>."
area: hygiene
state: open
priority: low
closes_when: "The traits are replaced by inherent methods (or given a second implementor that needs them), and LorentzRepr is removed."
blocked_by: []
opened: 2026-10-10
tags: [helas, abstraction]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-10}
sources:
  - {id: r-b-report, resource: "../../sprints/hygiene/sessions/R-B-report.md", title: "R-B report (hygiene sprint)"}
---
R-B.16. Benches and tests import `SpinorRepr`/`VectorRepr`, so this is a multi-file change.

Filed at the close-out of the [hygiene sprint](../../sprints/hygiene/sprint.md); evidence in [R-B](../../sprints/hygiene/sessions/R-B-report.md).
