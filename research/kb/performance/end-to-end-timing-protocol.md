---
type: Procedure
title: Timing validation rows and integrate runs
description: "Pin RUST_TEST_THREADS and RAYON_NUM_THREADS for per-row times; duration_s is not a benchmark; isolated is not in-pass; same-host ratios only; per-point cost rises with acceptance."
status: draft
tags: [performance, measurement-method, validation, timing, madgraph]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-09}]
sources:
  - {id: n30-scope, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/30-perf-baseline-timings.md#L34-L43", title: "Note 30: the comparison this note does and does not license"}
  - {id: n30-repro, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/30-perf-baseline-timings.md#L179-L187", title: "Note 30 §3.3 (reproducibility)"}
  - {id: n30-stages, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/30-perf-baseline-timings.md#L284-L312", title: "Note 30 §5.1 (stage mapping)"}
  - {id: n31-gates, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/31-perf-sprint-3-plan.md#L768-L797", title: "Note 31 §5 (gates and measurement honesty)"}
  - {id: n31-protocol, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/31-perf-sprint-3-plan.md#L805-L874", title: "Note 31 §6.1 (correction to the close-out protocol)"}
  - {id: n31-comparable, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/31-perf-sprint-3-plan.md#L875-L926", title: "Note 31 §6.2 (what is and is not comparable)"}
  - {id: n31-mg-control, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/31-perf-sprint-3-plan.md#L1140-L1194", title: "Note 31 §6.6 (MadGraph's side as the host-drift control)"}
  - {id: n31-disagree, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/31-perf-sprint-3-plan.md#L1304-L1359", title: "Note 31 §6.9 (where this record disagrees)"}
  - {id: n32-tta, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/32-perf-addendum-plan.md#L865-L955", title: "Note 32 §7 (time to a target accuracy; cost vs acceptance)"}
  - {id: n31-load, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/31-perf-sprint-3-plan.md#L610-L613", title: "Note 31: row timings under sibling load"}
  - {id: n34-cutfirst, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/34-draw-followup-plan.md#L29-L55", title: "Note 34 §1.1 (cut-first density draw)"}
---

# Timing validation rows and integrate runs

How to take a timing of the validation layer, of one of its rows, or of a whole
`vibegraph integrate` run, so that it can be compared with anything. Kernel-level
timing of the evaluator has its own procedure:
[benchmarking the evaluator](../performance/microbenchmark-protocol.md). The
recorded numbers are in
[validation-layer timings](../performance/validation-layer-timings.md); the
layer's structure is in [validation layers](../validation/validation-layers.md).

## 1. Same host, same sitting, ratios only

Cross-host comparison of absolute times is out of scope. What is in scope is a
**ratio taken on one host in one sitting**, with every number labelled with its
machine. Timings are measurements about a machine, not references: nothing of
this kind goes into a refdata bundle.[^n30-scope] The hosts and their noise
floors are in [benchmark hosts](../performance/benchmark-hosts.md); on the M3 Max
two identical passes agree to a median 0.8%, worst 3.4%, so a sub-1% claim is not
measurable per row.[^n30-repro]

Every table carries its command. Layer-level claims use the report's per-row
`duration_s`; kernel-level claims use criterion (`eval_strategies`). A
`duration_s` is a row's wall time inside a validation run, not a benchmark of
the code under it.[^n31-gates]

A change to the evaluator is gated on `tests/amplitude_oracle.rs` (byte equality
where the change claims bit-for-bit) and reports `eval_strategies` medians with
the host fingerprint plus a `scripts/mg_perf_compare.sh` before/after, the
per-point `MATRIX1` comparison of
[matrix element against MadGraph](../performance/matrix-element-vs-madgraph.md).
A change that touches σ adds a ≥5-seed sweep at two budgets.[^n31-gates]

## 2. Per-row times: pin the harness *and* the pool

Integration fans out on the global rayon pool, and `validate.sh` runs the gates
under the harness's default test parallelism. Pinning only `RAYON_NUM_THREADS=1`
funnels every concurrently running σ row through one worker, and each row's
`Stopwatch` charges itself the whole cohort's wall time. The signature: every
hadronic integrals row lands at ~305–320 s whether it spends 4.3M, 4.5M or 9M
points (an ~8× phantom regression). The correct protocol pins both:[^n31-protocol]

```
RUST_TEST_THREADS=1 RAYON_NUM_THREADS=1 cargo test -p vibegraph-lib \
    --profile release-debug --features extended-validation --test <binary> \
    -- --nocapture --test-threads=1
```

**A row measured alone is not the same measurement as that row inside a pass.**
`pp_to_bb_fixed` reads 37.1 s alone and 26.0 s inside a pass, same binary, same
9M points. The difference is one-time process setup (the interned SM, and on a
hadronic row the PDF grid), which an isolated run charges to the only row present
and a pass charges to whichever row runs first. A session isolating a row must
subtract that setup or compare only like with like.[^n31-protocol][^n31-disagree]

The per-target invocation (`-p vibegraph-lib --features extended-validation`)
rebuilds relative to `validate.sh`'s workspace invocation even though the
resolved features are identical; it is a build difference to state, far below
the moves usually reported.[^n31-disagree]

## 3. What is comparable across a change

- **Per-row `duration_s`**: comparable under the pinned protocol, subject to the
  setup caveat above.
- **Elapsed wall of the default command** (`pixi run --skip-deps validate`):
  comparable only like-for-like, and then as a **lower bound** on a gain when the
  suite has grown between the two runs (new tests mean the later run buys more
  work). The pinned per-row protocol serialises the harness, so its wall is
  comparable to nothing and is reported per binary only.[^n31-comparable]
- **Wall versus CPU.** `validate` runs rows concurrently, so a CPU saving shows in
  wall only to the extent the run is CPU-bound; quote both.
- **The `diagrams` category is a protocol artefact.** `sm_model()` is a
  process-wide interned model. Under default test parallelism the 26 rows race
  its lazy initialisation and each `Stopwatch` spans the contention, which reads
  as a uniform ~0.55 s per row (14.2 s total). One row at a time the category is
  1.29 s, concentrated in the two rows with real enumeration work.
- **The `amplitudes` category is honest work.** `amplitude_oracle::measure` runs
  enumeration and `AmplitudeEvaluator::compile` per row, which is why its two 2→6
  rows carry ~1.0–1.1 s and every other row ≤ 0.02 s; it reproduces under both
  protocols.[^n31-comparable][^n31-disagree]
- **The `samples` category has no uncontended protocol.** Rows that drive
  `vibegraph generate` as a subprocess misbehave under the pinned protocol
  (`pp_to_llj_dyn` samples read 93.2 s and 210.4 s under the two protocols); the
  in-process partonic samples rows behave. Take layer-level samples numbers from
  the like-for-like default run only.[^n31-disagree]
- A row's `samples` cell carries event generation *and* the weighted-ECDF KS and
  χ² comparisons against MadGraph's banked sample, so a large change in event
  read-out shows as a small change in the cell.

## 4. MadGraph's side, and what maps to what

Re-run MadGraph's stage timings in the same sitting as the control that says the
host has not drifted:[^n31-mg-control]

```
pixi run -e madgraph python validation/madgraph/time_stages.py \
    --out <dir> <the processes, same order>
```

- Quote `time_stages.py`'s stage-accounted sum, not the wrapper's wall: the
  wrapper also counts environment activation and LHAPDF fetches (2 995 s wall
  against 1 028 s of stages in one sitting).
- The first MadGraph invocation of a pass in a fresh environment pays a cold
  Python import (~19–20 s on `ee_to_mumu` against 9.2 s warm); net it out.
- A one-off LHAPDF set install recurs on the one process that needs it.
- A host that reproduces the previous MadGraph pass to ~2–3% licenses reading
  our side's before/after against it.

The two pipelines do not cut their work at the same seams, so a stage-by-stage
wall table is a shape comparison only:[^n30-stages][^n31-comparable]

| ours | MadGraph | why they differ |
|---|---|---|
| `diagrams` | `generate` | ours was dominated by harness setup (see §3); MadGraph's is the enumeration alone |
| `amplitudes` | `output` + `compile` | ours compares tables and compiles an evaluator per row; MadGraph writes and builds Fortran and ALOHA routines |
| `integrals` | `integrate` | different budgets and stopping rules; ours also carries evaluator construction, the multichannel α survey and grid adaptation |
| `samples` | `events` | ours also runs the KS and χ² comparisons; MadGraph combines and unweights only |

Both integrators run in parallel (MadGraph as a job farm, ours on rayon), so a
cost comparison against MadGraph uses CPU seconds on its side
(`<cumulated_time>`) and single-thread wall on ours; see
[integration against MadGraph](../performance/integration-vs-madgraph.md) for
that construction and its host dependence.

## 5. Time to a target accuracy

The quantity a user waits for is CPU-seconds to a given relative uncertainty on
σ. Ours is measured, wall-clock over the whole process, so model load,
enumeration and evaluator compilation are inside it:[^n32-tta]

```
vibegraph integrate <proc card> --run-card <run card> \
    --target-rel 0.001 --seed {20260719,20260720,20260721} -j 1
```

MadGraph's side is its banked run's `<cumulated_time>` with that run's σ ± err
from `SubProcesses/results.dat`, scaled by the 1/δ² law to the δ *our* run
reached, which puts the whole extrapolation on MadGraph's side. δ is the
χ²-scaled error (quoted error × √max(1, χ²/dof)), the same quantity
`--target-rel` stops on. Report seed spread; it follows the iteration count.

## 6. Per-point cost rises as the grid learns

A point the cuts reject short-circuits before the matrix element, the scale
clustering and the multichannel density sum, so the average point gets more
expensive as the grids learn the fiducial region. The rise was first seen on the
579-channel 2→6 (70.1 → 91.5 µs/point over six iterations at constant points per
iteration). It was explained on `pp_to_llj`: acceptance goes from 23.8% on an
untrained grid to 48.5% trained, and a two-component fit (`c_pre ≈ 1.0 µs`,
`c_post ≈ 12.9 µs`) reproduces that row's per-iteration averages; `ee_to_mumu` is
flat because its acceptance barely moves. **An ns/point quoted
from early iterations understates a converged run's cost.**[^n32-tta]

The density sum is priced only after the cut and the matrix element (the
cut-first draw, see [the phase-space channel contract](../phase-space/channel-contract.md)),
so a rejected point costs only the draw. That change was order-preserving
(byte-identical artifacts at fixed seed) and moved `probe_2to6_eval_cost` from
62.9/68.5 to 4.6/6.8 µs on the 579/615-channel rows; real fresh-grid multichannel
acceptance there is ≈3%.[^n34-cutfirst]

## Traps seen

- A σ printed by a probe in GeV⁻² is not pb: multiply by `GEV2_TO_PB`
  (3.893793721e8) before comparing with a bank.[^n32-tta]
- A row timing from a sitting under heavy sibling load is not bankable even when
  the row contents are byte-identical (one such spread reached −57%); re-measure
  on a quiet host.[^n31-load]
- On the M3 Max a loaded host inflates wall by tens of per cent while CPU moves
  little; see [benchmark hosts](../performance/benchmark-hosts.md).

[^n30-scope]: Note 30, "The comparison this note does and does not license".
[^n30-repro]: Note 30 §3.3.
[^n30-stages]: Note 30 §5.1, the stage mapping (its `amplitudes` explanation and its single-threaded-integrator premise are superseded; see note 31 §6.2 and §6.7).
[^n31-gates]: Note 31 §5, per-session gates and measurement honesty.
[^n31-protocol]: Note 31 §6.1.
[^n31-comparable]: Note 31 §6.2.
[^n31-mg-control]: Note 31 §6.6.
[^n31-disagree]: Note 31 §6.9.
[^n32-tta]: Note 32 §7, protocol, the cost-versus-acceptance explanation, and the probe's unit correction.
[^n34-cutfirst]: Note 34 §1.1.
[^n31-load]: Note 31, an evaluator session's row timings taken under sibling load (`31-perf-sprint-3-plan.md` L610–613).
