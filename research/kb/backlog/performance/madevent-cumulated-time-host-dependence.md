---
type: Backlog Item
title: MadEvent's cumulated_time denominator halves between hosts
description: Time-to-accuracy and throughput ratios against MadEvent halve from M3 Max to Cascade Lake because MadEvent's non-MATRIX1 job CPU barely scales; the M3 figure may be inflated.
area: performance
state: needs-user
priority: medium
closes_when: The M3 Max pass is re-run with nb_core = 12 (or pinned to performance cores) and its cumulated_time compared with the 16-job bank, and the non-MATRIX1 share of a MadEvent job is attributed.
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

Most of that CPU is per-job work other than the matrix element. Two questions
are open:
1. Is the M3 figure inflated by 16 concurrent jobs over 12 performance and 4
   efficiency cores? Re-run the M3 pass with `nb_core = 12`, or pinned to
   performance cores, and compare `cumulated_time`. This needs the user's M3
   Max host.
2. Which part of a MadEvent job is the non-`MATRIX1` CPU? A profile of one job
   answers this on any host.

Detail: [mg-comparison-cascade-lake-results.md §3](../../history/notes/mg-comparison-cascade-lake-results.md).
