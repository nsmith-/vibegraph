---
type: Backlog Item
title: "RNG stream families claim disjointness that no test pins"
description: "Six stream-id constants across rng.rs, unweight.rs, lhef/emit.rs, hadronic.rs and proton.rs are documented as disjoint; nothing checks their ranges."
area: hygiene
state: open
priority: low
closes_when: "One registry in phasespace/rng.rs holds every stream family with its maximum index, and a test asserts the ranges are pairwise disjoint."
blocked_by: []
opened: 2026-10-10
tags: [rng, determinism, convention-claim]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-10}
sources:
  - {id: r-d-report, resource: "../../sprints/hygiene/sessions/R-D-report.md", title: "R-D report (hygiene sprint)"}
---
Today the closest families are 12 835 apart, so nothing collides; the claim is unpinned (R-D.12, AGENTS.md "Convention claims are hypotheses").

Filed at the close-out of the [hygiene sprint](../../sprints/hygiene/sprint.md); evidence in [R-D](../../sprints/hygiene/sessions/R-D-report.md).
