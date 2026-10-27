---
type: Backlog Item
title: "The MultiplicitySum construction is written in four places"
description: "integrate.rs, generate.rs and validate_hadronic.rs (twice) each build per-part ProtonIntegrands with maps, vetoes and run-card scales, then call MultiplicitySum::new."
area: hygiene
state: open
priority: low
closes_when: "One library constructor on MultiplicityGroups builds the per-part integrands and the sum, and the four sites call it."
blocked_by: []
opened: 2026-10-26
tags: [duplication, multiplicity, proton]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-26}
sources:
  - {id: hy1, resource: "../../sprints/flavour-mixed-masses/sessions/HY1-report.md", title: "flavour-mixed-masses HY1 report, Found 1"}
---
Sites on `main` at `cfd9f7a`: `vibegraph-cli/src/integrate.rs:878`,
`vibegraph-lib/src/generate.rs:915`, `vibegraph-lib/tests/validate_hadronic.rs:1090`
(the mixed-mass gate) and `:5726` (the matched-sample gate). Each builds
`ProtonIntegrand::new_with_maps` per part with on-shell vetoes and run-card
scales, then `MultiplicitySum::new`
([HY1](../../sprints/flavour-mixed-masses/sessions/HY1-report.md), H13).
