---
type: Session Report
title: "R-A report: the evaluator (helas/eval)"
description: "21 findings on helas/eval: a squared-norm current gate blind to a zero Z current, per-diagram probes that bypass the production runtime, parked egraph code with non-optional dependencies, an unpinned op-kind numbering, and a disposition (keep f64) for the coefficient item."
status: draft
tags: [hygiene, review, report]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-09}]
sources:
  - {id: commit, resource: "https://github.com/nsmith-/vibegraph/commit/f7efda6", title: "Reviewed at f7efda6, read-only"}
---
The reviewer's report, condensed by the manager. Paths are under
`vibegraph-lib/src/helas/eval/`. Mutations were reasoned from the code, not
run. This is the performance hot path, so no *fix here* proposal changes the
emitted op order or the kernel bodies.

**Backlog check:** about 20 filed items cite the cluster. `evaluator-doc-comments-stale`
misses sites (R-A.4).

## Findings

| id | point | site | finding | proposed | confidence |
|---|---|---|---|---|---|
| R-A.1 | 2 | `run.rs:2322-2325` | The jioxxx current gate is `bare_norm_sq() < 1e-8` at √s = 1, where \|J_Z\|² ≈ 3e-9, so a zero Z current passes. | fix here (relative tolerance on the norm) | checked by arithmetic |
| R-A.2 | 2 | `run.rs:916-927,1874-1891` and six doc sites | The per-diagram probes claim the "production `run_forward` path"; they run test-only `lower::lower` and `run_forward_slot`, never `fill_arenas`. | fix here (docs); file (route the probes through `Program`) | checked; coverage gap suspected |
| R-A.3 | 1, 4 | `mod.rs:34-38`; `Cargo.toml:50-51` | Parked `egraph.rs` (1544 lines) has no production consumer, yet `egglog` and `ordered-float` are non-optional dependencies. | file (feature-gate) | checked |
| R-A.4 | 1 | `mod.rs:6-42`; `error.rs:3` | The module map's dependency-order claim, the "leaf module" claim and the dispatch description are false; a plan is narrated. These are missing sites of the claimed item. | fix here (with evaluator-doc-comments-stale) | checked |
| R-A.5 | 1 | `mod.rs:29,84`; `analysis.rs:337`; `tree.rs:35-117` | Module-wide `dead_code` allows hide `analyze_sym` and `tree`'s linearize utilities, kept for a bench that does not exist. | fix here | checked |
| R-A.6 | 2 | `layout.rs:371-499` | `Instr::kind`, `NAMES` and `N_KINDS` are hand-synced. `kind` is the op-blocking sort key and no test pins it. | fix here (a bijection test) | checked |
| R-A.7 | 4 | seven pool-leaf `Op` sets; two root read-out walks in `layout.rs` | Repeated compile-time patterns. | fix here (`Op::is_pool_leaf`; one iterator) | checked |
| R-A.8 | 1 | `root_lorentz.rs:605-893` | `build_at_leg` is 289 lines, with four identical arms and a shadowed `term`. | fix here | checked |
| R-A.9 | 1 | `root_diagram.rs:382-521` | `bake_node` takes 8 parameters (six invariant) under `too_many_arguments`. | fix here (a context struct) | checked |
| R-A.10 | 1 | `compile.rs:127-355` | `compile` is 229 lines; five extractions are named. | fix here | checked |
| R-A.11 | 3 | `compile.rs:127`; `rescale.rs:49,91,266-271` | `pub fn`s return types from a private module (`EvalError`, `RescaleFallback`, `PoolTagCensus`) that no caller can name. | file (surface decision) | checked |
| R-A.12 | 1, 3 | `compile.rs:430-492` plus links in `hadronic.rs`, `proton.rs`, `flow_tags.rs`, the book and the kb | The `select_color_flow` contract sits on a test-only API. | fix here (move to `select_config_and_flow`) | checked |
| R-A.13 | 2 | `run.rs:5218-5255` | An asserting gate (hash-cons sharing, arena bound) under a bare `#[ignore]`; four other ignores carry no reason. | fix here | checked; coverage suspected |
| R-A.14 | 2 | `run.rs:4344-4374` | The Z path checks one component at 2e-3 where the γ path checks four at 1e-6; a history comment. | fix here | suspected |
| R-A.15 | 2 | `compile.rs:1157-1184` | `lowered_add_mul_are_binary` checks the test-only lowering, not `lower_flows`. | fix here (or delete with `lower::lower`) | checked |
| R-A.16 | 1 | `waveform_slot.rs:27`; `run.rs:3687,3792` | Comments name `GammaJout` and `probe_2to5_momentum`, which no longer exist. | fix here | checked |
| R-A.17 | — | lorentz-coefficients-still-f64 | Keep f64 with a recorded decision: coefficients are folded in f64 in the UFO parser; migration crosses modules, changes the CSE keys and so the emitted program, and gains no consumer; fermion signs are already exact `i8`. | leave, with a recorded decision | checked |
| R-A.18 | 4 | `root_lorentz.rs:353,627,1517,1722`; `lower.rs:47,103` | ±1 signs flip between `f64` and `i8`. | file (`Sign` type) | checked |
| R-A.19 | 1 | `schedule.rs:744-792` | A rescale study test in the execution-order module; its result is in the kb. | fix here (move or delete) | checked |
| R-A.20 | 2 | `kernel.rs:1437` | Uses 1e-12 beside `CLIFFORD_TOL = 1e-13`. | fix here | checked |
| R-A.21 | 1 | `layout.rs:776-1121`; `run.rs:1076-1482` | `lower_node` and `fill_arenas` are long flat matches on the hot path. | leave | checked |

**Also seen:**
- `lanes.rs` aliases could merge into `lane_field.rs`;
- a bit-exact test at 1e-11 duplicating `prop_harness`;
- `analyze` runs twice;
- an `n_ext` check reads one arbitrary map entry;
- an order-dependent allowlist compare;
- speculation and plan comments;
- a guard contradicting its comment (`run.rs:3233`).

**Cross-cluster patterns:**
- unnameable public types stringified by callers;
- test-only contract anchors;
- `GammaJout` also at `helas/mod.rs:261`;
- parked code under module-wide allows (`vectorspace.rs`);
- absolute or squared-norm tolerances on quantities spanning orders of
  magnitude.

## Method

| step | share | produced |
|---|---|---|
| backlog check | 10% | — |
| length scan, long functions, `use super::` import map, backticked-name sweep | 40% | R-A.4, R-A.8–10, R-A.16 |
| tolerance survey, path tracing, diffing declaration order against `kind()` | 30% | R-A.1, R-A.2, R-A.6, R-A.13–15, R-A.20 |
| visibility | 10% | R-A.11 |
| reuse and study features | 10% | R-A.3, R-A.5, R-A.7 |

**Dead ends:**
- `pub(crate)` items are capped by private modules anyway;
- the study features all have a recorded decision or a live consumer.

## Found

- **`ufo/lorentz.rs:468` `div_terms` drops a parenthesised divisor.** This is
  the same as R-C.5, found independently.

## Brief corrections

- **Only one kernel test is far above `FUSED_TOL`.** The loose gates are in
  `run.rs`.
- **The study features have not outlived their studies;** the parked egraph and
  tree utilities have.
- **`compile.rs` is mostly accessors.**
- **`git branch` is empty on a detached checkout.**

## Manager check (2026-10-09)

Worktree clean.
- **R-A.3:** `egglog`/`ordered-float` are at `Cargo.toml:50-51`, and only
  `egraph.rs` uses them.
- **R-A.1:** the `bare_norm_sq() < 1e-8` assert is at `run.rs:2322-2325`.
- **R-A.6:** `N_KINDS = 54` and `AddScaled => 53` are at `layout.rs:373,434`.
