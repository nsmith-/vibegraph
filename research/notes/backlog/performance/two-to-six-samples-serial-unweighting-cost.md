---
type: Backlog Item
title: The 2→6 samples cells cost ~40 minutes of serial accept/reject
description: Efficiency is fine (117 and 45 trials per event) but the accept/reject loop advances one RNG trial at a time, so both 2→6 samples cells need ~40 serial minutes.
area: performance
state: open
priority: medium
closes_when: Accept/reject parallelises (or otherwise shrinks) so both 2→6 samples cells run inside a budget the long layer accepts, and the cells are run and recorded.
blocked_by: []
opened: 2026-08-05
tags: [two-to-six, unweighting, parallelism, samples-gate]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-06}
sources:
  - {id: todo, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L1076-L1098", title: "TODO.md entry T100"}
---
`probe_2to6_sample_cost` drives the real accept/reject loop off its own scanned
maxima, under the production truncation rule:

| row | acceptance | trials/event | accept/reject, 3 × 20000 events | grids + w_max scan |
|---|---|---|---|---|
| `uux_to_ccx_emmm_qcd0` | 8.561e-3 | 117 | 1577 s | 148 s |
| `bbx_to_ccx_emmm_qcd0` | 2.244e-2 | 45 | 817 s | 175 s |

Both are well inside the samples gate's 400-trial budget. The cost is wall time:
the loop advances one RNG trial at a time, so it is serial by construction. The
pair takes about 40 minutes against a 391 s banked layer, and no core count
helps. The two `samples` cells therefore stay unrun.

One candidate lever is accept/reject on deterministic parallel substreams, as
the integration already does for its points.

Detail: the cells' notes in `validation/manifest.toml`;
[note 32 §5.1](../../32-perf-addendum-plan.md).
