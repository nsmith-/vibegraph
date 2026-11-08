---
type: Backlog Item
title: "gpdb computes σ/MG for runs whose param card was dropped or whose run card was patched"
description: "When the CLI refuses a param card, vgrun reruns without it (param-card:dropped); the comparison view still gives ratio and pull, so DarkPhoton at the model default MZd = 20 reads as a σ/MG."
area: validation
state: open
priority: medium
closes_when: "The comparison view (or the statistics read from it) gives no ratio or pull for an attempt whose patches change physics (param-card:dropped, and runpatch: entries unless listed as benign), reporting them as their own outcome, with a test_parse.py case."
blocked_by: []
opened: 2026-11-08
tags: [gpdb, cms, tooling, silent-physics-change]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-08}
sources:
  - {id: closeout, resource: "../../sprints/cms-gridpack-coverage/closeout.md", title: "Sprint H close-out draft: the closing rerun, DarkPhoton caveat"}
  - {id: vgrun, resource: "../../../../validation/cms_gridpacks/vgrun.py", title: "vgrun.py: the param-card:dropped and runpatch fallbacks"}
  - {id: schema, resource: "../../../../validation/cms_gridpacks/schema.sql", title: "schema.sql: the comparison view"}
---
`vgrun.py` passes the gridpack's param card with `--param-card`. When the CLI
refuses the card, the next attempt runs without it and records
`param-card:dropped` in `patches` (`vgrun.py`, docstring and about line 316).
The `comparison` view in `schema.sql` (about line 387) takes each card's final
attempt. It computes `ratio` and `pull` for any `ok` outcome, whatever its
`patches`.

In the closing rerun, DarkPhoton's card was refused for a `chi` mass the model
lacks
([param-card-entries-for-absent-particles-refused](../feature/param-card-entries-for-absent-particles-refused.md)).
gpdb then ran at the model's default MZd = 20 and reported a σ/MG that compares
different physics.

`runpatch:` attempts are the same kind of change. They reset a refused run-card
field to MadGraph's default, as idm's `small_width_treatment` presumably was
(inferred from its refused attempt 1). Some
may be benign; the view cannot tell. The `patches` column already marks every
such attempt (`patches = '[]'` is as-shipped), so the fix is in what the view
and the statistics read from it.
