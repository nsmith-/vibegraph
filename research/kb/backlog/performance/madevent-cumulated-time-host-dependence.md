---
type: Backlog Item
title: MadEvent's cumulated_time denominator halves between hosts
description: "Time-to-accuracy and throughput ratios against MadEvent halve from M3 Max to Cascade Lake because MadEvent's non-MATRIX1 job CPU barely scales; which part of a job that CPU is remains unattributed."
area: performance
state: open
priority: medium
closes_when: The non-MATRIX1 share of a MadEvent job's CPU is attributed by a profile of one job.
blocked_by: []
opened: 2026-09-26
tags: [madevent-parity, timing, host, readme-headline]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-06}
sources:
  - {id: todo, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L988-L999", title: "TODO.md entry T092"}
---
On a 4-core Cascade Lake VM, MadGraph's `MATRIX1` and this integrand both run
3.0× slower than on the M3 Max (2.75–3.33× per process), so the per-point ratio
holds at 0.87×. MadEvent's summed job CPU (`<cumulated_time>`) grows only
1.1–1.6×. That halves both end-to-end ratios:

| geomean | Cascade Lake | M3 Max |
|---|--:|--:|
| time to 0.1 % on σ, MG/ours (5 rows) | 1.67× | 3.84× |
| integrand throughput, ours/MG (26 rows) | 3.97× | 8.76× |

Most of that CPU is per-job work other than the matrix element.

**The M3 figure is not inflated by its job layout** (measured 2026-10-16, PR
#25's banking batch): the M3 pass re-run at `nb_core = 12` gives each of the 31
processes a `<cumulated_time>` within 0.68–1.06× of the 16-job bank (geomean
0.98, median 1.00, 28 of 31 within ±6 %). The end-to-end ratios above stand.

**Open:** which part of a MadEvent job is the non-`MATRIX1` CPU. A profile of
one job answers it on any host. On the M3 at `nb_core = 12` a `gg_to_gg` job
lasts under 0.4 s, too short for `sample` to attach (most samples land in
`_dyld_start`), and replaying refine jobs re-uses converged grids. It needs a
profiler in the job launch, or one long-running job.

Detail: [mg-comparison-cascade-lake-results.md §3](../../history/notes/mg-comparison-cascade-lake-results.md).
