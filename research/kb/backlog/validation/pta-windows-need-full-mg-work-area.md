---
type: Backlog Item
title: "gen_pta_windows.sh runs only on a full MadGraph work area"
description: "The ee_to_mumua window generator compares its template's configs.inc with output/ee_to_mumua's, which the refdata bundle does not carry, so it cannot run from a fetched checkout."
area: validation
state: open
priority: low
closes_when: "gen_pta_windows.sh (and the @ee-mumua banking group) runs from a checkout with the fetched refdata bundle, by bundling configs.inc or comparing against configs.json."
blocked_by: []
opened: 2026-10-16
tags: [validation, refdata, madgraph, ee-mumua]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-16}
---
`gen_pta_windows.sh` checks that its template matches
`validation/madgraph/output/ee_to_mumua/SubProcesses/*/configs.inc`. The bundle
does not carry `configs.inc`, so the `@ee-mumua` banking group, and any
diagnostic run of the script, works only where MadGraph generated the
directory (the M3 bank host). To resolve: bundle `configs.inc` or compare
against `configs.json`. Found in the
[F1 report](../../sprints/sigma-reference-scatter/sessions/F1-report.md).
