---
type: Backlog Item
title: Timing baseline leaves three stages unmeasured
description: The refs reference-generation stage is untimed, MadEvent's results.dat point count may include the survey, and rows have no per-phase duration_s.
area: performance
state: open
priority: low
closes_when: The refs stage is timed without writing into the reference bank, the results.dat survey question is settled, and report rows carry per-phase duration_s covering evaluator construction.
blocked_by: []
opened: 2026-08-04
tags: [timing, madevent-parity, validation-report]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-06}
sources:
  - {id: todo, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L1099-L1104", title: "TODO.md entry T101"}
---
Three gaps from the per-stage timing baseline:
- The `refs` stage of `generate_references.sh` (f2py modules, amplitude tables,
  α_s and PDF oracles) is untimed, because timing it means writing into the
  reference bank. The regeneration figure covers the `madgraph` stage only.
- Whether MadEvent's `results.dat` point count includes the survey pass is
  unresolved. Settling it tightens the throughput denominators but does not move
  the trend.
- Evaluator construction time sits inside the integrals and samples rows, so
  nothing on our side faces MadGraph's `output` + `compile` column. A
  `duration_s` per phase inside a report row, rather than one per row, would
  give that counterpart.

Detail: [note 30 §8](../../history/notes/30-perf-baseline-timings.md).
