---
type: Backlog Item
title: pp_to_jj's nine tie-break events have no clustering dump
description: K4 enforces pp_to_jj's 9 beam-crossing tie-break events by signature only; no instrumented dump shows their merge sequence.
area: hygiene
state: open
priority: low
closes_when: An instrumented p p > j j clustering dump is banked in kt_cluster_dump_manifest.json, and the 9 tie-break events' merge sequences are checked against it.
blocked_by: []
opened: 2026-08-06
tags: [kt-clustering, scales, oracle, pp-to-jj]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-06}
sources:
  - {id: todo, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L539-L544", title: "TODO.md entry T042"}
---
`vibegraph-lib/tests/validate_scales.rs:315` allows `TIE_BREAK_MISSES = [("pp_to_jj", 9)]`.
These events are enforced by signature: the only difference is the `√(1+1e-6)`
beam-crossing inflation, and the printed digits of `<rscale>` pin it. The test
also asserts the count. Only an instrumented dump of a `p p > j j` run would show
the merge sequence directly. `validation/madgraph/kt_cluster_dump_manifest.json`
banks eight runs and none of them is `pp_to_jj`. A future pass over the oracle
layer can add one with `validation/madgraph/gen_kt_cluster_dumps.sh`. Detail:
[note 28](../../history/notes/28-kt-spine-feature-sprint-plan.md) §K1.3 (tie-break order) and §K4.
