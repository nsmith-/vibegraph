---
type: Backlog Item
title: "A stale σ-row cost note, and a report column that mixes references"
description: "A doc says gg_to_ttx_smlimit_qcd2 runs in 17.3 s (measured 69–108 s a seed), and the report's integrals column shows a row's worst variant, mixing references."
area: hygiene
state: open
priority: low
closes_when: "The cost note is re-timed on an idle host and corrected (or the row's cost cut), and the integrals column names the reference each cell reads or shows the row's own variant."
blocked_by: []
opened: 2026-10-17
tags: [sigma-gate, validation-report, docs]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-17}
sources:
  - {id: v1, resource: "../../sprints/sigma-pulls-lean-high/sessions/V1-report.md", title: "V1 report, Found"}
  - {id: f1, resource: "../../sprints/sigma-pulls-lean-high/sessions/F1-report.md", title: "F1 report, Found"}
---
- **The cost note.** `gg_to_ttx_smlimit_qcd2` took 69–91 s a seed in V1 and
  102–108 s under load in F1, against the 17.3 s its doc states. That is about
  47 % of the five-seed `validate_sigma`
  ([F1](../../sprints/sigma-pulls-lean-high/sessions/F1-report.md)).
- **The report column.** `report.md`'s integrals column shows the worst
  |pull| over a row's variants. `ee_to_ee_nsz`, `ee_to_wp0wm` and `pp_to_ll`
  show a control, and two grammar controls read `grammar_sigma_reference.json`
  ([V1](../../sprints/sigma-pulls-lean-high/sessions/V1-report.md)). A
  population read off the report is therefore not a seed-record population;
  `sigma-pulls` reads the row files instead.
