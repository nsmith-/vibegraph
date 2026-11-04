---
type: Backlog Item
title: "The llj matched-sample fraction test is ignored though its MadEvent run is banked"
description: "matched_sample_record_fractions_against_madevent says MadEvent's matched runs are outside the bundle, but pp_to_llj_mlm/Events/run_01 is in it and the test runs in 7 s."
area: validation
state: open
priority: low
closes_when: "The llj fraction test runs in the banked layer with the weighted estimator the ttx test uses, and validate-mlm-samples is retired or given another job."
blocked_by: []
opened: 2026-11-03
tags: [mlm, samples, banked-layer, non-vacuity]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-03}
sources:
  - {id: gv1, resource: "../../sprints/grid-worker-fixes/sessions/G-V1-report.md", title: "grid-worker-fixes G-V1 report, Found 1"}
---
The `#[ignore]` reason dates from before refdata cut 9. G-V1 ran the test here
(pulls +0.44, +0.01, −0.01, −0.00). Its manifest note records a Z share of
0.9547, against 0.9527 now. The difference is within errors, but the note
predates #23. The ttx test's unpooled weighted variance handles generate's
above-one weights, which a plain count does not.
