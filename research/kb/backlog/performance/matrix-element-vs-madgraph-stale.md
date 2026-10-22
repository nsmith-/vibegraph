---
type: Backlog Item
title: The matrix-element comparison against MadGraph is stale and ignores MadGraph's compiler flags
description: "The MATRIX1 table predates a dozen evaluator changes and compares default builds only; re-measure on x86 at default and x86-64-v3 flags on both sides, beside an M3 run."
area: performance
state: open
priority: medium
closes_when: "matrix-element-vs-madgraph.md carries a fresh table at one recorded commit on an x86 host in two configurations: defaults on both sides, and both sides built for x86-64-v3 (MadGraph through the run card's global_flag/aloha_flag/matrix_flag or the equivalent make_opts for the output the benchmark uses, vibegraph through -C target-cpu=x86-64-v3). An M3 run at the same commit sits beside them. Each row records the exact flags."
blocked_by: []
opened: 2026-10-21
tags: [performance, madgraph-comparison, matrix-element, benchmarks, x86-64-v3, compiler-flags, user-requested]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-21}
sources:
  - {id: me, resource: "../../performance/matrix-element-vs-madgraph.md", title: "Matrix element per point against MadGraph's MATRIX1"}
---
**Stale.** The x86 table was measured at `cf8b2b7` (2026-09-25, Cascade Lake),
and the M3 Max table on 2026-08-06 without a recorded commit. Since `cf8b2b7`,
twelve merges have touched `vibegraph-lib/src/helas/`, including the evaluator
headroom work (#17) and the triple-gluon sign (#39), so the `gg_to_gg` ratio
(1.37×) and the 0.87× geomean are not current
([MATRIX1 comparison](../../performance/matrix-element-vs-madgraph.md)).

**Unfair in a direction nobody has measured.** MadGraph's run card has
compiler-flag fields (`global_flag`, `aloha_flag`, `matrix_flag`). Neither
recorded table sets them, so both compare default builds only. A user tuning
MadGraph would set them, and our own release asset has an x86-64-v3 build.

**The user's request (2026-10-21).** Run once all current sprint work has
landed:
- an x86 host, defaults on both sides;
- the same host, both sides at x86-64-v3;
- the M3 at the same commit.

**To settle first.**
- Whether the run-card flags reach the output `scripts/mg_perf_compare.sh`
  builds, or that path needs its `make_opts` edited instead.
- MadGraph's default value for each field at the pinned checkout.
- Whether to include any `-ffast-math`-style flag. If one is included, record
  it and keep it out of the like-for-like row.
