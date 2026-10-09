---
type: Measurement
title: Where integrate and sample time goes, and per-point allocations
description: "samply profiles: evaluator 50–63% of self time, PDF interpolation 14–19% on hadronic paths, allocator 6–18%; the scale-draw cost and the per-event clustering allocations hoisted since."
status: draft
tags: [performance, profiling, pdf, allocation, scales]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: n30-draw, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/30-perf-baseline-timings.md#L408-L459", title: "Note 30 §6 (chain B's live-draw cost)"}
  - {id: n30-profiles, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/30-perf-baseline-timings.md#L460-L478", title: "Note 30 §7 (profiles: method and paths)"}
  - {id: n30-grouped, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/30-perf-baseline-timings.md#L479-L498", title: "Note 30 §7.1 (where the time sits, grouped)"}
  - {id: n30-each, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/30-perf-baseline-timings.md#L499-L540", title: "Note 30 §7.2 (one paragraph each)"}
  - {id: n30-agree, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/30-perf-baseline-timings.md#L541-L551", title: "Note 30 §7.3 (what the four agree on)"}
  - {id: n32-s5-plan, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/32-perf-addendum-plan.md#L164-L353", title: "Note 32 §2 Wave 1 (S1 and S5 briefs)"}
  - {id: n32-outcomes, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/32-perf-addendum-plan.md#L478-L597", title: "Note 32 §5.1 (per-session outcomes)"}
  - {id: n32-followups, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/32-perf-addendum-plan.md#L749-L793", title: "Note 32 §5.4 (standing follow-ups)"}
  - {id: n31-e3, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/31-perf-sprint-3-plan.md#L704-L749", title: "Note 31 §E3 (re-baselined draw cost)"}
  - {id: setclscales, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/vibegraph-lib/src/coupling/cluster/setclscales.rs#L270-L325", title: "coupling/cluster/setclscales.rs per-call Vecs"}
  - {id: merge-tables, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/vibegraph-lib/src/coupling/cluster/graph.rs#L345-L373", title: "MergeTablesByOrder"}
measured:
  - {commit: 45a7d62, host: "Apple M3 Max, macOS 15.7", command: "scripts/profile.sh (release-debug, extended-validation, samply 1 kHz)"}
  - {host: "Apple M3 Max", command: "cargo test -p vibegraph-lib --profile release-debug --features extended-validation --test validate_sigma -- --ignored --nocapture --test-threads=1 probe_scale_cost"}
---

# Where integrate and sample time goes, and per-point allocations

Four samply profiles of the validation layer's integrate and sample stages, the
cost of the per-point scale-configuration draw, and the allocation work they
pointed at. Profiles were taken at `main` `45a7d62` on the M3 Max; the evaluator
has become several times faster since (see
[matrix element against MadGraph](../performance/matrix-element-vs-madgraph.md)),
so read the shares as where time sat then, and re-profile before sizing a new
session. How to take a profile is in [profiling](../tooling/profiling.md).

## Profiles

All four with `scripts/profile.sh`: `--profile release-debug` (thin LTO),
`extended-validation`, samply at 1 kHz, `--unstable-presymbolicate`. Shares are
self time within the busiest thread.[^n30-profiles][^n30-grouped]

| group | integrate partonic (`validate_sigma`) | integrate `llj_dyn` (`validate_hadronic`) | sample unweighting | sample proton (`cli_generate_proton`) |
|---|--:|--:|--:|--:|
| busiest-thread work | 40.9 s | 109.3 s | 14.8 s | 27.2 s |
| evaluator (`helas::*`) | **51.4%** | **52.2%** | **49.9%** | **62.5%** |
| PDF interpolation | 0.0% | **14.5%** | 0.0% | **19.4%** |
| phase-space map | 12.9% | 6.2% | 5.5% | 7.0% |
| allocator + libc mem | 12.0% | 6.7% | **18.4%** | 0.6% |
| kT clustering | 8.1% | 5.8% | 9.2% | 0.0% |
| `BTreeMap` (outside the map) | 4.1% | 3.2% | 5.1% | 0.0% |
| libm `log`/`exp`/`pow` | 2.1% | 3.1% | 1.2% | 3.8% |
| LHEF writing | 0.0% | 0.0% | 0.0% | 0.01% |

What the four agree on:[^n30-agree][^n30-each]
- `fill_arenas` is 28–34% of self time in every profile and `helas::*` 50–63%.
  With most kernels inlined into it under thin LTO, read `fill_arenas` as "the
  matrix element", not as dispatch overhead. The kernels that did not inline
  showed separately (`ffv_vout_bare` 6.0%, `propagate_f{in,out}_bare` 4.1% on the
  partonic integrate path).
- PDF interpolation (`LogBicubic::xfx_q2` plus `PdfMember::xfx_q2`) is 14–19%
  wherever there are protons and absent otherwise. Generation re-evaluates
  luminosities on every trial, so it peaks on the proton sample path.
- The unweighting pass is the most allocation-bound (18.4% allocator, 5.1%
  `BTreeMap`) and the most clustering-heavy (9.2%), because accept/reject
  re-derives the per-event scale on every trial.
- **LHEF writing is 0.01%**: event output is not a cost.
- The phase-space map on the partonic path was led by
  `DiagramChannel::density_at`, `subtree_momentum` and `branch_jacobian`.

Two artefacts to read past: the other ~16 threads in these profiles were rayon
workers parked in `__psynch_cvwait`, from an integrand that was single-threaded
then (integration is parallel now; see
[integrate thread scaling](../performance/integrate-thread-scaling.md)); and
on macOS, samples on dylib import stubs symbolicate to whatever text symbol
precedes the stub section.

## The scale-configuration draw

On a live-draw row (MadEvent's per-point configuration draw, `sde_strategy = 1`)
each point pays an `eval_amp2` and an `alpha_s` move before the scale is
clustered. Isolated by building two integrands from the same run-card text, one
with `sde_strategy` rewritten to 2, over the same 20 000 fixed uniform points:[^n30-draw]

| row | draw cost per point | share of per-point budget |
|---|--:|--:|
| `gu_to_epemu` / `gux_to_epemux` (one subprocess) | ≈1.0 µs | ~21% |
| `pp_to_llj_dyn` | ≈0.2 µs (good only to a factor ~1.7) | 2.5–4.3% |

The factor of five is structural: `eval_amp2` runs for the one flavour group
whose channel drew the point, while `pp_to_llj_dyn`'s matrix element sums every
group and both beam orderings. The draw is what makes σ agree, so it stays;
what changed is when it is computed:
- after the faster evaluation order, the partonic draw re-measured at ≈870 ns /
  ≈18%;[^n31-e3]
- [arena reuse](../performance/arena-reuse-cache.md) lets `eval_amp2` and
  `eval_m2` share one fill when their pools match (−83% draw cost on `pp_to_ll`);
- the fixed-beam integrand now checks the cut before drawing (`scale_u` is a
  slice of the point's own uniforms, so cutting first is a pure dead-work skip,
  bit-identical for every accepted point), where 22% of
  `gu_to_epemu`/`gux_to_epemux` points had paid ~190 ns of dead draw
  work.[^n32-outcomes]

## Per-event clustering allocations

The unweighting profile named the clustered scale path's allocations. What was
hoisted: `ScaleChoice::cluster_scales` rebuilt three `BTree` merge-table
containers per event; `MergeTablesByOrder` now builds one table set per coupling
order at setup.[^merge-tables] `probe_scale_cost` read −16.9% (`gg_to_gg`),
−21.6% (`gg_to_ttx`), −22.3% (`uux_to_uux`) ns/point; `validate_unweighting`
−16.8% and the partonic σ gate −11.1% end to end; byte-identical throughout
(2 000 events byte-for-byte on the clustering-scale card).[^n32-outcomes]

That was not the allocation the work had been scoped for. What remains
allocates per call, and the path runs 2–3 times per event under matching:
- `ScaleChoice::cluster_history` builds its momentum `Vec` per call;
- `coupling/cluster/setclscales.rs` allocates several `Vec`s per call
  (`attempts`, `traces`, `pt2`, `mt2`, `lines`).[^setclscales]

Fix: thread a scratch struct through `setclscales`/`cluster`, gated on
bit-for-bit event bytes at fixed seed and measured with `probe_scale_cost`. Open
as [scale-path-per-event-allocations](../backlog/performance/scale-path-per-event-allocations.md).
The engine itself is in [the kT clustering engine](../scales-pdf/kt-clustering-engine.md).
A per-event improvement under 5% on the profiled rows is a reasonable kill
criterion for that work.[^n32-s5-plan][^n32-followups]

`probe_scale_cost` had been unrunnable (a widened integrand dimension had
broken it) and was fixed before it measured anything; check it runs before
trusting a null.[^n32-outcomes]

## Reproduce

```
scripts/profile.sh …        # release-debug, extended-validation, samply
cargo test -p vibegraph-lib --profile release-debug --features extended-validation \
  --test validate_sigma    -- --ignored --nocapture --test-threads=1 probe_scale_draw_cost
cargo test -p vibegraph-lib --profile release-debug --features extended-validation \
  --test validate_hadronic -- --ignored --nocapture --test-threads=1 probe_scale_draw_cost
```

The per-row costs these profiles sit under are in
[validation-layer timings](../performance/validation-layer-timings.md).

[^n30-draw]: Note 30 §6.
[^n30-profiles]: Note 30 §7, profile method and paths.
[^n30-grouped]: Note 30 §7.1.
[^n30-each]: Note 30 §7.2.
[^n30-agree]: Note 30 §7.3.
[^n31-e3]: Note 31 §E3, re-baselined draw cost.
[^n32-s5-plan]: Note 32 §2, the S1 and S5 briefs.
[^n32-outcomes]: Note 32 §5.1, S1 and S5 outcomes.
[^n32-followups]: Note 32 §5.4, the E4 continuation (its `coupling/scales.rs:376` reference predates the move of clustering into `coupling/cluster/`).
[^setclscales]: Per-call `Vec`s in `coupling/cluster/setclscales.rs`.
[^merge-tables]: `MergeTablesByOrder` in `coupling/cluster/graph.rs`.
