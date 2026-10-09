---
type: Backlog Item
title: The schedule order fallback ignores lane width
description: "SCHEDULE_BYTE_LIMIT's fallback counts f64 bytes, so it misses the lanes8 working-set loss on small-L2 hosts (2→6 was 15–18% faster in arena order on Cascade Lake, before constant collection)."
area: performance
state: open
priority: low
closes_when: The payoff of a lane-aware order fallback is measured at the lane width production would ship, on the current tree, and the fallback is either implemented without regressing any row or recorded as not worth it.
blocked_by: []
opened: 2026-09-25
tags: [helas-eval, schedule, lanes, x86, measure-first]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: todo, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L731-L762", title: "TODO.md entry T060"}
  - {id: topdown, resource: "../../history/notes/topdown-zen4-results.md", title: "topdown-zen4-results §6, lane ratios after constant collection"}
---
Op-blocked instruction order is kept in production: it groups independent
instructions and stays predictable on long programs. On small-L2 hosts it lost
on one cell: lanes8 on the 2→6 was 15–18% faster in arena / `minlive` order on
Cascade Lake (2.4 MB of op-blocked arenas against a 1 MiB L2). The M3 Max
(16 MiB L2) showed none. `SCHEDULE_BYTE_LIMIT`
(`vibegraph-lib/src/helas/eval/layout.rs:729`) is `f64`-byte and lane-blind.

**Stale evidence (re-checked 2026-10-09).** The item also cited Emerald Rapids
(2 MiB L2), where the 2→6 gained only 1.03× from lanes4 to lanes8. That was the
pre-constant-collection program (db5fd03). After constant collection the 2→6
arenas shrank by 36% (2.3 MiB at 8 lanes), and width 8 beats width 4 on
Emerald Rapids (44.4 against 46.6 µs/event,
[topdown-zen4-results §6](../../history/notes/topdown-zen4-results.md)). The Cascade Lake
numbers have not been re-measured since, so the cell may have closed. Measure
on the current tree first.

Measure at the width lanes would ship (lanes4 on a v3 target, where the effect
was absent). A fix needs a per-width program or order, since `Program` is shared
by every `F`, and any fallback order must stay predictable. The 2→6
within-level shuffle has not been run on x86. Detail:
[threaded-dispatch study §7](../../history/notes/threaded-dispatch-study-results.md),
[roofline census](../../history/notes/roofline-census-results.md). Benchmarks carry an
up-to-22% memory-layout confound; use `scripts/bench_schedule.sh`.
