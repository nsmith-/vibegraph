---
type: Design Decision
title: Op-blocked execution order within ASAP levels
description: "Program::build groups instructions by Instr variant inside each ASAP level: level grouping pays on every program, dispatch predictability on long ones; arena and shuffled orders lose."
status: draft
tags: [performance, evaluator, scheduling, branch-prediction, ilp]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: n31-e1, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/31-perf-sprint-3-plan.md#L517-L617", title: "Note 31 §E1/E1b (execution-order study and production pass)"}
  - {id: tds-summary, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/threaded-dispatch-study-results.md#L13-L48", title: "Threaded-dispatch study summary and Cascade Lake host"}
  - {id: tds-orders, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/threaded-dispatch-study-results.md#L123-L220", title: "Threaded-dispatch study §3 (dispatcher × execution order, Cascade Lake)"}
  - {id: tds-m3, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/threaded-dispatch-study-results.md#L221-L370", title: "Threaded-dispatch study §4 (M3 Max and the shuffle control)"}
  - {id: tds-layout, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/threaded-dispatch-study-results.md#L475-L549", title: "Threaded-dispatch study §6 (layout-randomised sweeps)"}
  - {id: tds-leaves, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/threaded-dispatch-study-results.md#L550-L582", title: "Threaded-dispatch study §7 (what this leaves)"}
  - {id: td-slots, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/topdown-zen4-results.md#L53-L127", title: "Top-down Zen 4 §2 (slot accounting, arena and shuffled controls)"}
  - {id: layout-order, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/vibegraph-lib/src/helas/eval/layout.rs#L690-L740", title: "layout.rs op_blocked_order and SCHEDULE_BYTE_LIMIT"}
  - {id: layout-build, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/vibegraph-lib/src/helas/eval/layout.rs#L1121-L1145", title: "layout.rs Program::build"}
  - {id: schedule-rs, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/vibegraph-lib/src/helas/eval/schedule.rs#L1-L80", title: "schedule.rs (study orders and metrics)"}
measured:
  - {commit: 052a00e, host: "Apple M3 Max", command: "cargo bench -p vibegraph-lib --bench eval_strategies (6-round round-robin, min over rounds)"}
  - {commit: 6bd7325, host: "Intel Xeon @ 2.8 GHz, Cascade Lake (family 6 model 85), 4-vCPU Firecracker VM", command: "VIBEGRAPH_EVAL_SCHEDULE sweep, eval-schedule-study feature"}
  - {commit: 2008fbf, host: "Apple M3 Max, macOS 15.7", command: "scripts/bench_dispatch.sh"}
  - {commit: 5ced9bb, host: "AMD EPYC 9534 (Zen 4), bare metal, RHEL 9", command: "scripts/topdown_kit.sh"}
---

# Op-blocked execution order within ASAP levels

**Decision.** `Program::build` emits instructions sorted by
`(ASAP dependency level, Instr kind, node id)`: level by level, and inside each
level grouped by instruction variant.[^layout-order]

```rust
pub(super) fn op_blocked_order(ast: &Ast<Const>, an: &NodeAnalysis) -> Vec<NodeId> {
    let level = asap_levels(ast);
    let kind = instr_kinds(ast, an);
    let mut order: Vec<NodeId> = (0..ast.len() as NodeId).collect();
    order.sort_unstable_by_key(|&id| (level[id as usize], kind[id as usize], id));
    order
}
```

Any topological order computes the same values with the same arithmetic; only
slot recycling, operand reuse distance and the variant sequence the dispatch
sees change. So every order is **bit-for-bit** gateable, which is how each was
tested. The grouping key is each node's true `Instr` discriminant, read by
running the lowering `match` against a null slot map, so it cannot drift from
the variant it groups.[^n31-e1]

## Why it wins: two jobs

1. **Level grouping, on every program.** Consecutive instructions in a level are
   independent, so an out-of-order core overlaps them across the dispatch jump.
   Interning ("arena") order places each producer right before its consumer and
   serialises the latency chain. Op-blocked beats arena order by **−19.2%**[^tds-summary] on
   scalar `forward` on Cascade Lake and **−19.3%** on the M3 Max (geomean over 8
   rows); lanes care less (1–10%), since a lane instruction carries 2–8× the
   arithmetic per dispatch.[^tds-orders][^tds-m3]
2. **Predictability, on long programs.** A history-based indirect predictor
   learns the whole dispatch sequence of a small program in any order. A program
   of tens of thousands of instructions exceeds what it can learn, and there the
   order inside a level must be predictable. Shuffling the 2→6 (36 523
   instructions) within its levels costs **2.2×** on the M3 Max under either
   dispatcher (`forward` 87.6 → 193.2 µs/event; lanes4 1.56×, lanes8 1.31×), with
   46.5% of cycles discarded against 1.3% op-blocked. On the seven small rows the
   same shuffle costs nothing (0.95–1.02).[^tds-m3]

Op-blocked runs are one way to be predictable; a periodic interleave is another.
The `LevelMix` control (same levels, variants dealt round-robin, runs of 2–3)
matches op-blocked within 1% on both hosts.[^tds-orders][^tds-m3]

The counters agree on both mechanisms. On the M3 Max (Instruments' CPU
Counters), arena order loses to processing stalls, not discards (processing
24.1 → 35.9% on the 2→6; useful-share ratios 1.29 and 1.22 match the timings
1.285 and 1.23), while the shuffle loses to discards. On Zen 4, arena order
costs 1.45× cycles on the 2→6, 1.38× on the 4-lepton row and 1.10× on `gg_to_gg`
at width 1, all in retire stalls (oldest-op-incomplete 42% → 58–75%, load waits
4% → 25%, cache misses barely moving); the level shuffle raises indirect
mispredicts from 0.011 to 0.396 per VM instruction and costs 1.57× cycles, about
25 cycles per extra mispredict.[^tds-m3][^td-slots]

The earlier attribution of the whole win to discriminant run length (from
`OpWindow` controls on the M3 Max) does not hold: run length was confounded with
predictability, and `LevelMix` separates them. On short programs run length does
not matter; on long ones, predictability does.[^tds-orders][^tds-m3]

## What was measured against it

Relative to `match@opblocked` (Cascade Lake, sweep B; M3 Max, sweep A), scalar
`forward` geomean over the 8 bench rows:

| order | what it does | Cascade Lake | M3 Max |
|---|---|--:|--:|
| `opblocked` (production) | variant runs inside ASAP levels | 1.000 | 1.000 |
| `levelmix` | same levels, variants round-robin | 1.007 | 1.005 |
| `opwin32` | op-blocking in sliding windows of 32 | 1.055 | 1.051 |
| `minlive` | greedy live-width minimisation | 1.205 | — |
| `arena` | interning order (a bottom-up DFS) | 1.238 | 1.239 |
| `dfs` | post-order chain following | 1.258 | — |
| `levelshuffle` | same levels, seeded shuffle | — | 1.089 overall; **2.21× on the 2→6** |

The M3 Max figures come from one memory layout per sweep; a single scalar row can
move 10–22% with the process's environment length on that host, so its scalar
geomeans carry a few points of layout error. The shuffle's 2.2× is far outside
that.[^tds-layout] Live-width minimisation shrinks the 2→6's peak bytes
(236 760 → 206 160) and buys nothing at scalar width; ILP depth is not a
constraint (11–23 levels against 36 523 instructions).[^n31-e1]

In production on the M3 Max the pass took the evaluator geomean −17.34% (14/14
rows) and the `MATRIX1` ratio from 1.21× to 1.00×; it costs +1.06 ms of build
time on the largest production program (+0.2% of evaluator
construction).[^n31-e1] The threaded dispatcher does not change which order
wins; see [threaded dispatch](../performance/threaded-dispatch-study.md).

## The arena-size fallback, and where it is blind

Grouping by variant stretches some lifetimes, so arenas grow: the pruned 2→6
goes 0.31 → 0.46 MB at `f64`, the unpruned one 3.66 → 5.84 MB. `Program::build`
falls back to interning order only when op-blocked arenas exceed
`SCHEDULE_BYTE_LIMIT` (16 MiB at `f64`) **and** interning order is actually
smaller. The limit sits a few times above the largest program built today and
fires on nothing measured.[^layout-build][^layout-order]

The limit counts `f64` bytes and is **lane-blind**. At eight lanes the 2→6's
op-blocked arenas are 2.4 MB against arena order's 1.9 MB and `minlive`'s 1.6 MB.
On Cascade Lake (1 MiB L2) lanes8 on the 2→6 is 15–18% *faster* in any
non-op-blocked order, under both dispatchers; on the M3 Max (16 MiB L2) there is
no such flip (0.980). Emerald Rapids (2 MiB L2) sat on the edge until the fused
sums shrank the arenas (see
[constant collection and fused sums](../performance/constant-collection-and-fused-sums.md)).
A lane-aware fallback needs a per-width program or order, since one `Program` is
shared by every `F`; it must be sized at the width production lanes would ship
(lanes4 on a v3 target, where the effect is absent), and any fallback order must
stay predictable on long programs, or it trades an L2 miss for a mispredict per
instruction. The within-level shuffle has not been run on x86. Open as
[schedule-fallback-lane-blind](../backlog/performance/schedule-fallback-lane-blind.md).[^tds-orders][^tds-leaves]
The [roofline census](../performance/roofline-census.md) and
[top-down counters](../performance/topdown-zen4.md) carry the per-host working-set
figures.

## The study hook

The alternative orders live in `helas::eval::schedule` under `cfg(test)` or the
`eval-schedule-study` feature; a release build carries the production order
alone. With the feature, `VIBEGRAPH_EVAL_SCHEDULE` (`arena`, `dfs`, `minlive`,
`opblocked`, `opwin<N>`, `levelmix`, `levelshuffle`) picks the order at program
build, so one binary can sweep every order. The module also computes the four
structural metrics an order trades between: producer→consumer distance, live-set
width, discriminant run length, and critical-path depth against stream
length.[^schedule-rs]

Sweep with `scripts/bench_schedule.sh` (round-robin arms, environment padding
varied per round, min over rounds); see
[benchmarking the evaluator](../performance/microbenchmark-protocol.md).
`alternative_orders_are_bit_identical` plus the banked-row digests are the
correctness gate for any new order.

[^n31-e1]: Note 31 §E1 and E1b.
[^tds-summary]: Threaded-dispatch study summary and Cascade Lake host block.
[^tds-orders]: Threaded-dispatch study §3, Cascade Lake sweeps A and B.
[^tds-m3]: Threaded-dispatch study §4, M3 Max sweeps, the shuffle control and the Instruments counters.
[^tds-layout]: Threaded-dispatch study §6, the environment-length layout effect.
[^tds-leaves]: Threaded-dispatch study §7, the lane-aware fallback.
[^td-slots]: Top-down Zen 4 §2, arena and level-shuffle control cells.
[^layout-order]: `op_blocked_order` and `SCHEDULE_BYTE_LIMIT` in `layout.rs`.
[^layout-build]: `Program::build` in `layout.rs`.
[^schedule-rs]: `schedule.rs` module doc and `Schedule`.
