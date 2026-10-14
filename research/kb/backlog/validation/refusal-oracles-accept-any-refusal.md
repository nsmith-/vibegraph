---
type: Backlog Item
title: "The census and grammar oracles accept a refusal for any reason"
description: "Where MadGraph refused, proc_grammar_oracle and three census tests count any refusal at any stage, so a missing earlier rule is masked by a later one."
area: validation
state: open
priority: medium
closes_when: "Each banked refusal case names the stage or variant it expects, and the oracles compare it."
blocked_by: []
opened: 2026-10-10
tags: [oracle, refusal, blind-spot]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-10}
sources:
  - {id: r-c-report, resource: "../../sprints/hygiene/sessions/R-C-report.md", title: "R-C report (hygiene sprint)"}
  - {id: r-g1-report, resource: "../../sprints/hygiene/sessions/R-G1-report.md", title: "R-G1 report (hygiene sprint)"}
---
58 refusal cases across `proc_grammar_oracle`, `schannel_census`, `decay_chain_census` and `polarization_census` (R-C.1, R-G1.14). The blind spot is now stated in each module doc, and the empty-census guards landed; the class matching is open.

Filed at the close-out of the [hygiene sprint](../../sprints/hygiene/sprint.md); evidence in [R-C](../../sprints/hygiene/sessions/R-C-report.md), [R-G1](../../sprints/hygiene/sessions/R-G1-report.md).
