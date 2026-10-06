---
type: Backlog Item
title: The schedule order fallback ignores lane width
description: SCHEDULE_BYTE_LIMIT's fallback counts f64 bytes, so it misses the lanes8 working-set loss on small-L2 hosts (2→6 is 15–18% faster in arena order on Cascade Lake).
area: performance
state: open
priority: low
closes_when: The payoff of a lane-aware order fallback is measured at the lane width production would ship, and the fallback is either implemented without regressing any row or recorded as not worth it.
blocked_by: []
opened: 2026-09-25
tags: [helas-eval, schedule, lanes, x86, measure-first]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-06}
sources:
  - {id: todo, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L731-L762", title: "TODO.md entry T060"}
---
Op-blocked instruction order is kept in production: it groups independent
instructions and stays predictable on long programs. On small-L2 hosts it
loses on one cell: lanes8 on the 2→6 is 15–18% faster in arena / `minlive`
order on Cascade Lake (2.4 MB of op-blocked arenas against a 1 MiB L2), and on
Emerald Rapids (2 MiB L2) the 2→6 gains only 1.03× from lanes4 to lanes8
against 1.25–1.42× on other rows. The M3 Max (16 MiB L2) shows none.
`SCHEDULE_BYTE_LIMIT` (`vibegraph-lib/src/helas/eval/layout.rs:729`) is
`f64`-byte and lane-blind.

Measure first, at the width lanes would ship (lanes4 on a v3 target, where the
effect is absent). A fix needs a per-width program or order, since `Program`
is shared by every `F`, and any fallback order must stay predictable. The 2→6
within-level shuffle has not been run on x86. Detail:
[threaded-dispatch study §7](../../threaded-dispatch-study-results.md),
[roofline census](../../roofline-census-results.md). Benchmarks carry an
up-to-22% memory-layout confound; use `scripts/bench_schedule.sh`.
