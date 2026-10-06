---
type: Backlog Item
title: The merged configuration's forest is not compared with configs.inc on merging runs
description: validate_kt_cluster's derived-forest oracle covers only non-merging runs, so a merging run's channel forest is never compared line by line with MadGraph's.
area: validation
state: open
priority: medium
closes_when: derived_channel_forests_match_the_generated_ones (or a sibling) also runs on the merging single-subprocess runs and passes.
blocked_by: []
opened: 2026-09-07
tags: [mlm, kt-clustering, channel-forest, oracle]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-06}
sources:
  - {id: todo, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L414-L442", title: "TODO.md entry T029"}
---
On a merging run, two things are compared against MadGraph today:
- the channel count, against `coloramps.inc`;
- the configuration partition, by `amplitude_oracle` against `matrix1.f`.

The channel forests themselves are not. `derived_channel_forests_match_the_generated_ones`
(`vibegraph-lib/tests/validate_kt_cluster.rs`) iterates `DERIVED_FOREST_RUNS`,
and those are six non-merging runs only. Both sides now have one channel count
on the merging single-subprocess runs, so the same whole-forest comparison can
extend to them:
- every line's leg set and its daughters' leg sets;
- both propagator codes, the mass and the width;
- the bijection between the two sets of channels.

One further difference is not measured. The representative diagram is
ours-first, where MadGraph's is its-first. The two coincide wherever
`MG_DIAGRAM_ORDER` is the identity, and nothing measures the rest. A difference
there could cost sampling efficiency at most, never give a wrong answer.

Detail: [note 36 §7](../../36-banked-open-ends-plan.md).
