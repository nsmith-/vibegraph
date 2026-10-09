---
type: Measurement
title: "Validation-layer and MadGraph stage timings on the M3 Max"
description: "Per-row wall times of the validation layer and MadGraph's per-stage times at 45a7d62, re-measured at 62d78e4: integrals −53.7% one row at a time, validate 691 → 391 s."
status: draft
tags: [performance, timing, validation, madgraph, baseline]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-09}]
measured:
  - {commit: 45a7d62, host: "Apple M3 Max (12P + 4E), 48 GiB, macOS 15.7.7, no core affinity", command: "pixi run --skip-deps validate (release-debug, extended-validation, RUSTFLAGS unset); pixi run -e madgraph python validation/madgraph/time_stages.py --out target/s3-mg-timing <31 processes>"}
  - {commit: 62d78e4, host: "Apple M3 Max (12P + 4E), macOS 15.7.7", command: "RUST_TEST_THREADS=1 RAYON_NUM_THREADS=1 cargo test -p vibegraph-lib --profile release-debug --features extended-validation --test <target> -- --nocapture --test-threads=1; pixi run --skip-deps validate; time_stages.py control pass"}
sources:
  - {id: n30, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/30-perf-baseline-timings.md#L11-L69", title: "Note 30 §0–§1, the question and both sides' build settings"}
  - {id: n30-3, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/30-perf-baseline-timings.md#L108-L187", title: "Note 30 §3, our per-row wall times"}
  - {id: n30-4, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/30-perf-baseline-timings.md#L190-L281", title: "Note 30 §4, MadGraph per-stage times and regeneration cost"}
  - {id: n30-5, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/30-perf-baseline-timings.md#L284-L351", title: "Note 30 §5.1–§5.2, stage mapping and side-by-side wall time"}
  - {id: n31-61, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/31-perf-sprint-3-plan.md#L840-L926", title: "Note 31 §6.1–§6.2, the corrected protocol and what is comparable"}
  - {id: n31-63, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/31-perf-sprint-3-plan.md#L927-L1035", title: "Note 31 §6.3, per-row integrals and samples"}
  - {id: n31-65, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/31-perf-sprint-3-plan.md#L1104-L1194", title: "Note 31 §6.5–§6.6, the layer as a user runs it; MadGraph control"}
  - {id: n31-610, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/31-perf-sprint-3-plan.md#L1360-L1378", title: "Note 31 §6.10, what moved"}
---

# Validation-layer and MadGraph stage timings on the M3 Max

Two measurements on one host: a baseline at `45a7d62` and a re-measure at `62d78e4`, kept
side by side because the change between them is the point. Later trees are not measured.
Every number is a wall time on that machine; only ratios taken on one host in one sitting
are compared.[^n30] How to take such timings is in
[timing validation rows](end-to-end-timing-protocol.md).

## Host and builds

Apple M3 Max, 16 cores (12 performance + 4 efficiency), 48 GiB, macOS 15.7.7. The clock is
not exposed by the OS. **No affinity is set on either side**, so work can land on E-cores,
an unquantified uncertainty on a hybrid CPU.

- **Ours**: `rustc 1.94.1`, profile `release-debug` (thin LTO, `opt-level = 3`, `debug = 1`),
  feature `extended-validation`, `RUSTFLAGS` unset. Each gate writes `duration_s` beside its
  report row (`vibegraph-lib/tests/common/report.rs`, `Stopwatch`) and the first row writes
  `target/validation-report/host.json`.
- **MadGraph**: the pinned submodule (3.7.1) through `validation/madgraph/mg5_pinned.sh`,
  Python 3.11, GNU Fortran 14.3, LHAPDF 6.5.6. Its generated `make_opts` carry no `-O` and no
  `-march` beyond MadGraph's own default. `nb_core = None`: madevent ran up to 16 concurrent
  jobs.
- `validation/madgraph/time_stages.py` regenerates each process into a scratch directory (it
  refuses `validation/madgraph/output`) and timestamps every transcript line, so stage
  boundaries (`generate`, `output`, `compile`, `integrate` from `Running Survey` to
  `finish refine`, `events`) are read, not guessed. `generate` matches MadGraph's own
  self-timing to the millisecond.

## The layer as a user runs it

`pixi run --skip-deps validate`, nothing pinned:

| | `45a7d62` | `62d78e4` |
|---|--:|--:|
| elapsed | 691 s | **391 s (−43.4%)** |
| census | 98 measured: 96 ✅ / 2 ⚠️ / 4 ⏳ | identical |
| `#[test]` / `#[ignore]` | 861 / 32 (counted at `e951045`, the pre-sprint tip) | 905 / 34 |

The second run buys more work (42 more running tests, some deliberately expensive), so −43.4%
understates the change. A repeat of the baseline agreed per row to a median 0.8%, worst 3.4%,
over the 43 measurements above 1 s: that is the our-side noise floor, and why a sub-1% claim
is unmeasurable at this granularity.[^n30-3]

**This command's per-row durations are not a benchmark.** `cargo test` runs a binary's tests
in parallel threads and the integrators fan out underneath, so concurrently measured rows each
charge themselves contended wall time and overlap. At `62d78e4` eight hadronic integrals rows
landed within 31–37 s of each other while spending 4.5M to 9M points, and `pp_to_ll` read 64 s
against 11 s alone.

## Integrals, one row at a time

The comparable per-row protocol pins the harness and the pool:
`RUST_TEST_THREADS=1 RAYON_NUM_THREADS=1 … --test-threads=1`, for the six targets that write
rows, then the collator.[^n31-63] Seconds:

| row | points `45a7d62` → `62d78e4` | `45a7d62` | `62d78e4` | Δ |
|---|---|--:|--:|--:|
| `ee_to_mumu` / `ee_to_zh` / `uux_to_mumu` | 180k | 0.17 / 0.13 / 0.16 | 0.14 / 0.11 / 0.15 | −17 / −14 / −6% |
| `ee_to_ee` / `ee_to_ttx` / `ee_to_wpwm` | 800k / 180k / 320k | 0.96 / 0.23 / 0.80 | 0.78 / 0.18 / 0.59 | −19 / −23 / −27% |
| `uux_to_uux` / `gg_to_gg` / `gg_to_ttx` | 240k / 240k / 480k | 1.34 / 2.47 / 3.01 | 1.25 / 2.07 / 2.67 | −7 / −16 / −11% |
| `ee_to_mumua` / `ee_to_tatah` | 640k / 480k | 1.28 / 0.85 | 1.11 / 0.67 | −13 / −21% |
| `uux_to_epemg` / `ddx_to_epemg` | 480k | 3.16 / 3.40 | 2.93 / 2.89 | −7 / −15% |
| `gu_to_epemu` / `gux_to_epemux` | 480k | 3.35 / 3.32 | 3.08 / 3.07 | −8 / −8% |
| `ee_to_mumu_tata_qcd0` / `ud_to_epemud_qcd0` | 800k / 960k | 5.83 / 9.40 | 4.54 / 7.57 | −22 / −20% |
| `pp_to_ll` | 8.64M | 15.82 | 11.25 | −28.9% |
| `pp_to_bb` / `pp_to_bb_qcd2` / `pp_to_bb_fixed` | 9M | 62.94 / 78.45 / 38.59 | 46.52 / 56.99 / 26.04 | −26 / −27 / −33% |
| `pp_to_jj` | 9M | 174.81 | 86.17 | −50.7% |
| `pp_to_ll_scalefact2` | 9M | 41.78 | 26.01 | −37.8% |
| `pp_to_llj_fixed` | 9M → 4.5M | 87.87 | 27.75 | −68.4% |
| `pp_to_llj` | 18M → 4.5M | 186.19 | 34.40 | −81.5% |
| `pp_to_llj_dyn` | 9M → 4.5M | 116.32 | 40.85 | −64.9% |
| **total** | | **842.6** | **389.8** | **−53.7%** |

- **Partonic block** (17 rows, no PDF, unchanged budgets): 39.9 → 33.8 s (−15.2%), every row
  improved. This is the cleanest read on the evaluator alone, diluted by the phase-space map,
  cuts and clustering (about half the partonic integrand).
- **Hadronic block**: 802.8 → 356.0 s (−55.7%). At unchanged budget −26% to −51%, well above
  the partonic −15%, which is the all-flavour PDF kernel where the profiles put PDF
  interpolation (14.5–19.4% of self time on proton paths; see
  [integrate profiles](integrate-profiles.md)). The three llj rows also spend a quarter to a
  half of the baseline's points, so roughly half their drop is budget.
- The `45a7d62` column sums to note 30's own integrals total, which checks the transcription.

**Two traps in this protocol.**[^n31-61] A thread pool pinned to one thread while other rows
ran made one row read 305 s instead of 37 s: cross-row contention, not work. And a row measured
alone is not the same measurement as that row inside a pass: alone `pp_to_bb_fixed` read
37.1 s, inside the pass 26.0 s, because one-time setup (the interned SM, the PDF grid) lands on
whichever row runs first.

**Samples are not comparable under this protocol.** `pp_to_llj_dyn` samples read 127.8 s
(baseline, default command), 93.2 s (`RAYON_NUM_THREADS=1`) and 210.4 s (fully serialised).
Serialising should never make a row 2.3× slower; these rows drive `vibegraph generate` as a
subprocess, and a lone thread can land on an E-core, neither established. In-process partonic
samples behave: 128.2 → 106.7 s (−16.8%), all 17 improved. Collator totals under the default
command (overlapping spans, loose): integrals 842.6 → 336.7 s, samples 840.1 → 585.7 s.

## `diagrams` and `amplitudes`

Together about 17 s of ~1 700, so nothing rests on them. **`diagrams`** costs ~0.5–0.7 s per
row under the default command and 1.29 s in total run one at a time: `sm_model()` is a
process-wide interned model, and under default parallelism the rows race its lazy
initialisation, each `Stopwatch` spanning the contention. Run sequentially, only the 2→6 rows
(~0.6 s each) do real enumeration work; every other row is ≤ 0.05 s. **`amplitudes`** is
honest work either way (2.6 s and 2.2 s): `amplitude_oracle::measure` runs enumeration and
`AmplitudeEvaluator::compile` per row, which is why the 2→6 rows cost about 1.1 s each and a
2→2 0.02 s.[^n31-61]

## MadGraph's side

31 processes, all exit 0, every stage boundary parsed. Stage totals (seconds):[^n30-4]

| stage | `45a7d62` pass | control pass (2026-08-05) |
|---|--:|--:|
| `generate` | 1.2 | 1.2 |
| `output` | 62.8 | 62.3 |
| `compile` | 63.4 | 66.6 |
| `integrate` | 685.8 | 703.3 |
| `events` | 151.1 | 149.1 |
| **sum of per-process totals** | **1001.8** | **1028.3** (1017.6 net of a cold start) |

- Diagram generation is 0.1% of MadGraph's cost (median 10 ms). `output` and `compile` are
  ~6% each and nearly process-independent at 1.4–2.9 s. `integrate` is 68%, and the two 2→6
  rows alone are 389 s of it (173 s and 217 s).
- Typical `e⁺e⁻`/`qq̄` 2→2 and 2→3 rows take 8–11 s in total; the 4-lepton row 22 s;
  hadronic rows 20–60 s (`dy13_default` 59.6 s).
- **The control** reran the same 31 processes after the sprint, which touched neither
  MadGraph nor the bank: 27 of 31 within ±8%, the host reproducing to ~2–3%. That is what
  licenses reading the our-side change against the baseline at all.
- Recurring effects: the first MadGraph invocation pays a cold Python import (`ee_to_mumu`
  19–20 s against 9.2 s warm); `pp_to_llj` paid a one-off 20.5 s LHAPDF set download, because
  madevent's LHAPDF looks in the pixi environment's `share/LHAPDF`, not `validation/pdf/`.
  Quote the stage-accounted sum: the wrapper's own wall (2 995 s for the control) includes
  environment activation and the fetch.

**Regeneration cost**: all 31 MadGraph process directories with their launches took
**1001.6 s = 16 min 42 s** on this host, 981 s net of the PDF install, with warm page cache and
conda environment. It covers the `madgraph` stage of `validation/generate_references.sh` only;
the `refs` stage (f2py modules, amplitude tables, α_s and PDF oracles) and `bundle` write into
the reference bank and are untimed
([backlog](../backlog/performance/timing-baseline-unmeasured-stages.md)). See
[reference generation](../tooling/madgraph-reference-generation.md).

## Why the two sides' columns do not line up

- **`diagrams` vs `generate`**: ours is mostly per-trial harness setup, MadGraph's is the
  enumeration alone; comparing them compares a harness to an algorithm.
- **Evaluator construction vs `output` + `compile`**: our construction is inside the
  integrals and samples rows and is not timed separately, so `output` + `compile` has no
  our-side counterpart.
- **`integrals` vs `integrate`**: we spend a fixed `seeds × neval × niter`; MadGraph refines to
  a requested accuracy. Ours also carries construction, the multichannel α survey and grid
  adaptation. MadGraph runs up to 16 jobs in parallel; our integrators use the machine
  internally too (the hadronic path is multi-threaded), so wall times on both sides
  reflect parallel work. A per-point comparison needs CPU time or
  [integration against MadGraph](integration-vs-madgraph.md)'s throughput denominators; thread
  scaling is in [integrate thread scaling](integrate-thread-scaling.md).
- **`samples` vs `events`**: ours also runs the KS and χ² comparisons against the banked
  sample; MadGraph's stage only combines and unweights.
- At the baseline, side by side, our integrals rows were faster than MadGraph's `integrate`
  on every 2→2 and 2→3 partonic row, and slower on every hadronic row but `pp_to_ll`
  (15.8 s against `dy13_default`'s 30.8 s): `pp_to_jj` 174.8 s against 25.1 s, `pp_to_llj`
  186.2 s against 14.6 s.[^n30-5] Read this as shape, not as a
  ratio.

[^n30]: Note 30 §0–§1, at `45a7d62`.
[^n30-3]: Note 30 §3.
[^n30-4]: Note 30 §4; control in note 31 §6.6.
[^n30-5]: Note 30 §5.1–§5.2.
[^n31-61]: Note 31 §6.1–§6.2 (corrects note 30 §3.2's explanation of the diagrams and amplitudes columns).
[^n31-63]: Note 31 §6.3, at `62d78e4`; summary in §6.10.
