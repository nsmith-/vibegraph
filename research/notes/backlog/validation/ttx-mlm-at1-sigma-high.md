---
type: Backlog Item
title: pp_to_ttx_0j1j_mlm @1 reads 1.28 % above MadEvent
description: The t t~ MLM row's one-jet part is +1.28 % (+8.85σ) high against MadEvent on all ten seeds while @0 agrees; the cause is undiagnosed.
area: validation
state: open
priority: high
closes_when: The @1 excess is attributed to a cause (reference, H1 or this side) and, if this side's, fixed; the integrals cell then gates on a five-seed agreement with |pull| < 3.
blocked_by: []
opened: 2026-10-03
tags: [mlm, sigma, ttx, h1, madevent-reference]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-06}
sources:
  - {id: todo, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L196-L202", title: "TODO.md entry T000"}
  - {id: todo-h1, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L220-L228", title: "TODO.md entry T004 (H1 share on this row)"}
---
`sigma_ttx_0j1j_mlm_vs_madevent` (`validate_hadronic`, run by
`pixi run -e madgraph --skip-deps validate-mlm-sigma`) reads, on `refdata-9`
(2026-10-03), ten seeds 20260928–37 at `--fixed-budget --allocate neyman
--neval 200000 --niter 8`:

| part | vibegraph (pb) | MadEvent (pb) | rel | pull |
|---|---|---|---|---|
| `@0` | 513.222 ± 0.177 | 512.898 ± 0.181 | +0.06 % | +1.28 |
| `@1` | 583.192 ± 0.297 | 575.836 ± 0.777 | +1.28 % | +8.85 |
| total | 1096.413 ± 0.381 | 1088.640 ± 0.797 | +0.71 % | +8.80 |

Every seed (581.9–584.5) sits above MadEvent's whole range (572.5–579.7). The
cell is `info` in `validation/manifest.toml`.

Diagnose in order:
1. Regenerate the reference from independent MadEvent directories: nine of its
   ten seeds share one directory, and on `pp_to_ll_0j2j_mlm` that moved `@1` by
   −0.43 % (the other direction).
2. Compare `@1` per event, including H1's first-call rejections (the share of σ
   through points only one side's first `setclscales` call rejects), unmeasured
   on this row. 3744 of 10000 dumped events have a permuted `P1` (t ↔ t̄); all
   agree on every record field, but rejected points never reach the dump.

Detail: [note 41 §4 Z2](../../41-mlm-feature-sprint-plan.md) ("Z2 Landed"), H1 in
[note 41 D2 and R1](../../41-mlm-feature-sprint-plan.md).
