---
type: Backlog Item
title: validate_scales' replay cannot see the permuted first call
description: The scale replay searches configurations until one reproduces SCALUP, and the permuted twin is always in the set, so a full pass says nothing about H1; it is also blind to first-call rejections.
area: validation
state: open
priority: low
closes_when: validate_scales' "What this cannot see" names the permuted first call and first-call rejections, or the replay pins the configuration MadEvent clustered and reports a permuted match separately.
blocked_by: []
opened: 2026-10-15
tags: [h1, scales, oracle-blind-spot]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-15}
sources:
  - {id: census, resource: "../../sprints/sigma-h1-ickkw0-census/sessions/V1-report.md", title: "sigma-h1-ickkw0-census V1 report, Found 1"}
---
`replay()` in `vibegraph-lib/tests/validate_scales.rs` tries each integration
configuration until one reproduces the event's `SCALUP`. MadEvent's first-call
scale on the unpermuted `PP` equals this crate's clustering of `P1` under the
permuted twin configuration, which is in the searched set. So the replay
passes whether or not [H1](../../validation/madgraph-permuted-first-call.md) is
present. Rejected points never reach the event file, so the replay cannot see
first-call rejections either. A 10000/10000 pass is therefore not evidence
about H1. The test's "What this cannot see" section should say so.
