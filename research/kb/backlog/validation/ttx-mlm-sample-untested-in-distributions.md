---
type: Backlog Item
title: "The ttx MLM sample is compared with MadEvent only as population shares"
description: "Top pT, the t t~ mass, jet pT and the clustering scales of the pp_to_ttx_0j1j_mlm sample are not compared with MadEvent; the banked run_01 is a default-survey sample."
area: validation
state: needs-user
priority: low
closes_when: "Binned χ² comparisons of the ttx MLM sample's top, pair, jet and clustering-scale distributions against a hard_survey = 2 MadEvent run of the row pass, in the style of validate_samples_proton."
blocked_by: []
opened: 2026-11-03
tags: [mlm, ttx, samples, distributions, banking]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-03}
sources:
  - {id: gv1, resource: "../../sprints/grid-worker-fixes/sessions/G-V1-report.md", title: "grid-worker-fixes G-V1 report, Found 2 and 3"}
---
`ttx_matched_sample_record_fractions_against_madevent` gates 18 population
shares. At 10 000 events a side, it resolves only label- and population-level
errors. The banked `run_01` used `hard_survey = 0`: its @1 is 572.5 pb,
against 578.3 pb with the hard survey, so a shape-level comparison against it
would inherit MadEvent's under-integration of `g q > t t~ q`. A
`hard_survey = 2` event run of the row is new reference data, and the user
banks it on the M3.
