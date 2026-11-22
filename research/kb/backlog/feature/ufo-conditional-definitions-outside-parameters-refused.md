---
type: Backlog Item
title: "Conditional definitions outside parameters.py are still refused"
description: "The configuration.py evaluator covers parameters.py and coupling orders; an if-block or conditional value in particles, couplings, vertices, lorentz or propagators files is still refused."
area: feature
state: open
priority: low
closes_when: "Either a model that needs it is found and the evaluator flattens that file's conditional definitions with a test against MadGraph's import, or the refusal is documented as deliberate in release-scope.md."
blocked_by: []
opened: 2026-11-22
tags: [ufo, model-loading, refusal]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-22}
sources:
  - {id: u1, resource: "../../sprints/v023-orders-majorana-timeouts/sessions/L-U1-report.md", title: "Sprint L PR 7 report, Found 2"}
---
`vibegraph-lib/src/ufo/mod.rs` `refuse_conditional_definitions` refuses
conditional definitions in `particles.py`, `couplings.py`, `vertices.py`,
`lorentz.py`, `coupling_orders.py` and `propagators.py`. The evaluator in
`ufo/config_eval.rs` could flatten them the same way it does for
`parameters.py`. No store or gridpack model is known to need it, so this stays
low until one does.
