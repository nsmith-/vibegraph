# 30 — Per-stage timing baseline: vibegraph against MadGraph on one host

**Status:** measurement record, taken 2026-08-04 on `main` @ `45a7d62` (the merge that
brought the FMA/inlining evaluator changes in), before the performance sprint's first
optimizing session. Everything below is a recorded measurement on one machine; the
host block is in §1 and every number carries the command that produced it.

Answers three questions the repository had never measured:

1. **Where does a validation run's wall time go, stage by stage, per process** — and
   how does that compare with MadGraph's own generate / output / compile / integrate /
   events on the same machine (§3, §4, §5).
2. **What does a full MadGraph reference regeneration actually cost?** The phrase in
   note 29 and `TODO.md` is "multi-hour", which was a warning, not a datum. It is
   **16 min 42 s** for all 31 processes on this host (§4.3).
3. **What did note-29 chain B's per-point configuration draw cost?** `TODO.md` recorded
   it as unmeasured. It is **~1.0 µs/point (≈21%) on a partonic live-draw row** and
   **~0.2 µs/point (≈3%) on `pp_to_llj_dyn`** (§6).

Plus samply profiles of the integrate and sample stages (§7), which put the evaluator at
**50–63% of self time** on all four — one symbol, its instruction-dispatch loop, is
28–34% of it — and PDF interpolation at **14–19%** on every hadronic path.

### The comparison this note does and does not license

Note 15 §2.4's position stands and is the reason every number here is labelled with
its machine: **cross-host comparison of absolute times is out of scope**. What is in
scope is *ratios taken on one host in one sitting*, which is what §5 reports. Nothing
in this note goes into a refdata bundle — these are measurements about a machine, not
references — and `assemble_bundle.sh` was not touched.

---

## 1. The machine, and both sides' build settings

Written by the gates themselves as `target/validation-report/host.json` (schema 1),
and separately by the MadGraph pass as `target/s3-mg-timing/timings.json`'s `host`
block. Both were captured on the same host in the same sitting.

| | |
|---|---|
| CPU | Apple M3 Max, 16 logical / 16 physical cores — **12 performance + 4 efficiency** |
| clock | **not exposed by the OS**: `sysctl -n hw.cpufrequency hw.cpufrequency_max` return empty on Apple Silicon. Recorded as `null` rather than filled with a vendor figure the machine cannot confirm |
| memory | 51 539 607 552 B (48 GiB) |
| OS | macOS 15.7.7, Darwin 24.6.0 (build 24G720) |
| core placement | **no affinity is set on either side.** Nothing pins itself to a core class, so the scheduler is free to place work on P- or E-cores. This is a real uncertainty on a hybrid CPU and is why the run-to-run spread in §3.3 is quoted |
| vibegraph toolchain | `rustc 1.94.1 (e408947bf 2026-03-25)`, `cargo 1.94.1`, host triple `aarch64-apple-darwin`, toolchain `stable-aarch64-apple-darwin` |
| vibegraph build | profile `release-debug` (thin LTO, `opt-level = 3`, `debug = 1`, `debug_assertions = false`), features `extended-validation`. `RUSTFLAGS` **unset** — default codegen, which is what the recorded reference tables assume |
| MadGraph | the **pinned submodule** `research/refs/mg5amcnlo`, VERSION 3.7.1 (2026-04-29), driven through `validation/madgraph/mg5_pinned.sh` — not the conda `mg5amcnlo 3.5.7`, which supplies only the environment |
| MadGraph toolchain | Python 3.11.15, `GNU Fortran (GCC) 14.3.0`, `Apple clang 17.0.0`; LHAPDF 6.5.6 interface |
| MadGraph Fortran flags | from each generated `Source/make_opts`: `FFLAGS= -w -fPIC`, `+= -ffixed-line-length-132`, `+= -fno-common`, `LDFLAGS=$(STDLIB) $(MACFLAG)` with `STDLIB=-lc++`. **No `-O` flag and no `-march`** beyond MadGraph's own `GLOBAL_FLAG` default |
| MadGraph parallelism | `nb_core = None` (all cores). madevent ran **up to 16 concurrent jobs** — confirmed by `grep -ho "Running: *[0-9]*" *.timed.log` reaching 16 |

The asymmetry in that last row is the single most important caveat in this note: on
the integrate and events stages **MadGraph is a 16-way parallel job farm and our
integrator is one thread**. §5 therefore compares CPU time, not wall time.

---

## 2. What was instrumented

**Our side.** Every gate that writes a report row now writes `duration_s` beside it —
the wall time of that row's own measurement, from where the row's work starts to where
`write()` is called (`vibegraph-lib/tests/common/report.rs`, `Stopwatch`). The first row
any gate process writes also writes `target/validation-report/host.json`, atomically, so
one machine block accompanies each run's rows. `validation/validate.sh` deletes it with
the row directories, so no run reads its durations against another machine's identity.
The collator renders a `## Timing` section and carries `durations_s` per cell in
`report.json`. **No cell's verdict logic changed**; §3.1 is the evidence.

`duration_s` is not a benchmark. `cargo test` runs a binary's tests in parallel threads
and the integrators fan out over rayon underneath, so concurrently measured rows each
charge themselves the contended wall time and **their durations overlap** — a category's
summed time is not the invocation's elapsed time.

**MadGraph side.** `validation/madgraph/time_stages.py` regenerates each named process
**into a scratch directory** (it refuses to write into `validation/madgraph/output`) and
stamps every line MadGraph prints with the seconds elapsed since that process started, so
stage boundaries are read off the transcript rather than guessed:

| stage | opening marker | closing marker |
|---|---|---|
| `startup` | (process start) | `Checking for minimal orders` / `Trying process` |
| `generate` | that | `N processes with M diagrams generated in X s` |
| `output` | `initialize a new directory` | `Output to directory ... done.` |
| `compile` | `compile directory` | `Running Survey` |
| `integrate` | `Running Survey` | `finish refine` |
| `events` | `Combining Events` | `End Parton` |

`generate` is cross-checked against MadGraph's own printed self-timing on that line; the
two agree to the millisecond on every process. The result is `timings.json` plus one
timestamped transcript per process under `logs/`.

---

## 3. Our side: per-row wall times

### 3.1 The run these come from

```
$ pixi run --skip-deps validate            # instrumented tree, release-debug
START 2026-08-04T16:40:06Z
...
[report] 29 rows x 4 categories: the measured cells are the declared cells
VALIDATE_EXIT=0
END 2026-08-04T16:51:37Z
```

**691 s elapsed** (11 min 31 s), including cargo's incremental rebuild of the changed
test binaries. Census from `target/validation-report/report.md`:

```
29 rows × 4 categories = 116 cells: 98 measured in this layer (96 ✅, 2 ⚠️, 4 ⏳, 14 — / uncovered).
The measured cells are exactly the cells the manifest declares, every gate cell
passed, and every measurement agrees with the manifest about what it is.
```

96 ✅ / 2 ⚠️ — unchanged from the pre-instrumentation census, which is what "instrumentation
must move nothing" means.

### 3.2 Per-row durations

Seconds, from `report.json`. `—` is a cell the manifest does not have this layer measure.

| row | diagrams | amplitudes | integrals | samples |
|---|--:|--:|--:|--:|
| `ee_to_mumu` | 0.55 | 0.02 | 0.17 | 0.63 |
| `ee_to_ee` | 0.55 | 0.02 | 0.96 | 12.08 |
| `ee_to_ttx` | 0.55 | 0.02 | 0.23 | 0.71 |
| `ee_to_wpwm` | 0.55 | 0.02 | 0.80 | 1.39 |
| `ee_to_zh` | 0.55 | 0.02 | 0.13 | 0.42 |
| `uux_to_mumu` | 0.51 | 0.09 | 0.16 | 0.60 |
| `uux_to_uux` | 0.51 | 0.00 | 1.34 | 2.81 |
| `gg_to_ttx` | 0.54 | 0.02 | 3.01 | 3.85 |
| `gg_to_gg` | 0.55 | 0.02 | 2.47 | 7.03 |
| `ee_to_mumua` | 0.55 | 0.03 | 1.28 | 3.92 |
| `ee_to_tatah` | 0.55 | 0.03 | 0.85 | 1.79 |
| `uux_to_epemg` | 0.51 | 0.00 | 3.16 | 7.95 |
| `ddx_to_epemg` | 0.55 | 0.02 | 3.40 | 7.66 |
| `gu_to_epemu` | 0.54 | 0.30 | 3.35 | 10.25 |
| `gux_to_epemux` | 0.55 | 0.02 | 3.32 | 8.78 |
| `ee_to_mumu_tata_qcd0` | 0.55 | 0.04 | 5.83 | 24.17 |
| `ud_to_epemud_qcd0` | 0.51 | 0.04 | 9.40 | 34.17 |
| `uux_to_ccx_emmm_qcd0` | 0.65 | 1.12 | — | — |
| `bbx_to_ccx_emmm_qcd0` | 0.68 | 1.12 | — | — |
| `pp_to_ll` | 0.52 | — | 15.82 | 31.76 |
| `pp_to_ll_qcd0` | 0.51 | 0.02 | — | — |
| `pp_to_bb` | 0.55 | — | 62.94 | 38.00 |
| `pp_to_bb_qcd2` | 0.51 | — | 78.45 | 46.57 |
| `pp_to_bb_fixed` | 0.55 | — | 38.59 | 29.89 |
| `pp_to_jj` | — | — | 174.81 | 125.38 |
| `pp_to_ll_scalefact2` | — | — | 41.78 | 34.26 |
| `pp_to_llj_fixed` | 0.53 | — | 87.87 | 149.22 |
| `pp_to_llj` | 0.56 | — | 186.19 | 129.08 |
| `pp_to_llj_dyn` | — | — | 116.32 | 127.78 |

Category totals (sums of overlapping spans, not elapsed time): diagrams **14.2 s** over
26 rows, amplitudes **3.0 s** over 20, integrals **842.6 s** over 27, samples **840.2 s**
over 27. **97.9% of the layer's timed work is integrals and samples**, and 6 hadronic
rows carry two thirds of it.

Two shapes worth naming. Every `diagrams` row costs ~0.51–0.68 s whatever the process,
because each trial re-loads the interned SM and re-runs enumeration; the enumeration
itself is the small part. Every `amplitudes` row costs ~0.02 s, because that gate compares
committed tables and never builds an evaluator — the evaluator construction that
MadGraph's `output` stage corresponds to lives *inside* the integrals and samples rows.
§5 states that mapping caveat again where it matters.

### 3.3 Reproducibility

The run above was preceded by an identical one at 16:28 UTC (`VALIDATE_EXIT=0`, same
96 ✅ / 2 ⚠️, 694 s elapsed). Over the **43 measurements above 1 s** the two runs agree to
a **median 0.8%, worst 3.4%**. That is the noise floor for every our-side number in this
note, and it is what makes a claimed sub-1% speedup unmeasurable at this granularity.

---

## 4. MadGraph's side: per-stage wall times

### 4.1 The pass

```
$ pixi run -e madgraph python validation/madgraph/time_stages.py \
      --out target/s3-mg-timing <31 processes>
PASS START 2026-08-04T15:52:15Z
CHECKPOINT (21 representative rows) 2026-08-04T16:08:03Z
PASS END 2026-08-04T16:17:37Z
```

31 processes, **all exit 0**, every stage boundary parsed (no nulls). Ordering was the
`mg_timings.json` perf kit first, then `dy13_default`, `pp_to_llj`, `pp_to_jj`, then the
rest of the census.

### 4.2 Per-process stages (seconds)

| process | generate | output | compile | integrate | events | total |
|---|--:|--:|--:|--:|--:|--:|
| `ee_to_mumu` | 0.002 | 1.72 | 1.74 | 4.96 | 0.41 | 9.2 |
| `ee_to_ee` | 0.002 | 1.72 | 1.80 | 4.00 | 0.38 | 8.4 |
| `ee_to_ttx` | 0.002 | 1.76 | 1.69 | 5.24 | 0.39 | 9.5 |
| `ee_to_wpwm` | 0.002 | 1.59 | 1.75 | 4.11 | 0.39 | 8.3 |
| `ee_to_zh` | 0.002 | 1.49 | 1.73 | 3.78 | 0.43 | 7.9 |
| `ee_to_mumua` | 0.004 | 1.72 | 1.74 | 4.92 | 1.12 | 9.9 |
| `ee_to_tatah` | 0.003 | 2.04 | 1.73 | 4.72 | 0.72 | 9.6 |
| `ee_to_mumu_tata_qcd0` | 0.010 | 2.13 | 1.80 | 15.52 | 1.74 | 21.7 |
| `uux_to_mumu` | 0.002 | 1.89 | 1.72 | 5.16 | 0.37 | 9.6 |
| `uux_to_uux` | 0.003 | 1.39 | 1.71 | 4.29 | 0.45 | 8.3 |
| `gg_to_ttx` | 0.003 | 1.41 | 1.69 | 4.31 | 0.48 | 8.3 |
| `gg_to_gg` | 0.002 | 2.01 | 1.75 | 4.25 | 0.54 | 9.0 |
| `uux_to_epemg` | 0.002 | 1.73 | 1.74 | 5.35 | 1.20 | 10.5 |
| `ddx_to_epemg` | 0.002 | 1.72 | 1.78 | 4.51 | 1.36 | 9.8 |
| `gu_to_epemu` | 0.002 | 1.80 | 1.81 | 4.38 | 1.48 | 9.9 |
| `gux_to_epemux` | 0.002 | 1.75 | 1.78 | 5.57 | 1.14 | 10.7 |
| `ud_to_epemud_qcd0` | 0.013 | 2.25 | 1.83 | 5.28 | 1.58 | 11.4 |
| `uux_to_ccx_emmm_qcd0` | 0.296 | 3.63 | 1.85 | 172.84 | 3.23 | 182.3 |
| `bbx_to_ccx_emmm_qcd0` | 0.340 | 4.40 | 1.80 | 216.53 | 7.40 | 231.0 |
| `pp_to_ll` | 0.023 | 2.07 | 1.78 | 10.13 | 8.57 | 23.1 |
| `pp_to_ll_qcd0` | 0.022 | 1.72 | 1.69 | 10.29 | 8.83 | 23.0 |
| `pp_to_ll_scalefact2` | 0.024 | 2.10 | 2.57 | 10.93 | 8.57 | 24.9 |
| `pp_to_bb` | 0.010 | 1.60 | 2.45 | 7.03 | 8.02 | 19.8 |
| `pp_to_bb_fixed` | 0.009 | 2.12 | 2.65 | 18.30 | 7.80 | 31.6 |
| `pp_to_bb_qcd2` | 0.009 | 2.06 | 2.57 | 23.26 | 8.03 | 36.6 |
| `pp_to_llj` | 0.110 | 2.33 | 2.71 | 14.55 | 9.10 | 50.0 |
| `pp_to_llj_fixed` | 0.108 | 2.21 | 2.61 | 15.34 | 9.29 | 30.2 |
| `pp_to_llj_dyn` | 0.106 | 2.18 | 2.71 | 14.86 | 9.32 | 29.9 |
| `pp_to_jj` | 0.114 | 2.47 | 2.90 | 25.08 | 8.04 | 39.3 |
| `dy13_default` | 0.008 | 1.83 | 2.77 | 30.84 | 22.98 | 59.6 |
| `dy13_mmll_60_120` | 0.008 | 1.91 | 2.52 | 25.44 | 17.69 | 48.5 |
| **sum** | **1.2** | **62.8** | **63.4** | **685.8** | **151.1** | **1001.6** |

Stage totals say the shape plainly: **diagram generation is 0.1% of MadGraph's cost**
(1.2 s over 31 processes; median 10 ms), writing the process directory and compiling
its Fortran are ~6% each and are nearly process-independent at 1.4–2.9 s, and
**integrate is 68%** — 685.8 s, of which the two 2→6 rows alone are 389 s.

The stages account for each process's total to within ~0.7 s except `pp_to_llj`, where
50.0 s total against 28.8 s of stages is a **one-off 20.5 s LHAPDF set install**:
madevent's LHAPDF looks in the pixi environment's `share/LHAPDF` rather than in
`validation/pdf/`, so the first `pdlabel = lhapdf` launch of the pass downloaded
NNPDF23_lo_as_0130_qed there. Only that one process paid it
(`grep -l "successfully downloaded" logs/*.timed.log` matches one file).

### 4.3 The regeneration-cost answer

**Full regeneration of all 31 MadGraph process directories, with their launches:
1001.6 s = 16 min 42 s wall on this host** — 981 s net of the one-off PDF install.
The 21-row representative subset reached at the checkpoint took **736 s (12 min 16 s)**,
so the projected remainder was ~4 min: far inside the ~3 h continue-or-stop threshold,
and the pass was run to the full census.

Scope this precisely. This is the `madgraph` stage of `validation/generate_references.sh`
— `build.sh`-equivalent generation plus each script's own `launch`. It does **not**
include the `refs` stage (f2py matrix-element modules, amplitude tables, α_s and PDF
oracles) or `bundle`, both of which write into the reference bank and were deliberately
not run. "Multi-hour" as a description of *this* stage is off by an order of magnitude
on this machine; whether the `refs` stage restores it is unmeasured and remains open.

Two things this pass has warm that a truly cold one would not: the OS page cache
(a cold first MadGraph invocation pays ~3.7 s extra of Python import — a smoke run
before the pass measured `ee_to_mumu` at 19.0 s against 9.2 s inside the pass), and
the conda environment. Read 16 min 42 s as a warm-cache figure.

The regenerated directories are timing artifacts only. They were not compared against
the pinned bank and the bank was never written to; the scratch area is
`target/s3-mg-timing/` and is labelled by that path. As a sanity check that the runs
did the work they claim rather than exiting early, each one's `SubProcesses/results.dat`
carries a cross section of the expected magnitude (`pp_to_jj` 6.7885e8 pb,
`pp_to_llj_fixed` 423.84 pb, `dy13_default` 933.23 pb).

---

## 5. The comparison, and what it does not mean

### 5.1 Stage mapping — read this before the table

The two pipelines do not cut their work at the same seams. Four asymmetries dominate.

- **`diagrams` vs `generate`.** Both enumerate. But our 0.51–0.68 s per row is almost
  all fixed per-trial setup (interned-SM load, process-card parse, topology work), while
  MadGraph's `generate` is the enumeration alone, self-timed at 2–340 ms. Our number is
  an upper bound on our enumeration cost; MadGraph's is a tight one. Comparing them
  directly compares a harness to an algorithm.
- **`amplitudes` vs `output` + `compile`.** These do *not* correspond. MadGraph's
  `output` writes Fortran and ALOHA routines and `compile` builds them — the moral
  equivalent of our evaluator construction — whereas our `amplitudes` cell only
  compares committed tables and never builds an evaluator. Our evaluator construction
  is inside the integrals and samples rows and is not separately timed here. **The
  output+compile column has no our-side counterpart in this run.**
- **`integrals` vs `integrate`.** Different budgets *and* different stopping rules: we
  spend a fixed `seeds × neval × niter`, MadGraph refines until a requested accuracy is
  met. Our integrals row also carries evaluator construction, the multichannel α survey
  and the grid adaptation. And MadGraph runs 16 jobs in parallel while our hadronic path
  is single-threaded by construction (`hadronic.rs`'s `RefCell` scratch — the
  DY-parallelism backlog item).
- **`samples` vs `events`.** Ours generates events *and* runs the weighted-ECDF KS and
  χ² comparisons against the banked MadGraph sample; MadGraph's `events` stage combines
  and unweights only. Our samples column is therefore an over-estimate of the generation
  work by an unmeasured amount.

Given all that, the wall-time table below is a *shape* comparison. The throughput table
in §5.3 is the one with a defensible denominator.

### 5.2 Wall time, side by side (seconds)

| row | ours: diagrams | MG generate | ours: integrals | MG integrate | ours: samples | MG events |
|---|--:|--:|--:|--:|--:|--:|
| `ee_to_mumu` | 0.55 | 0.002 | 0.17 | 4.96 | 0.63 | 0.41 |
| `ee_to_ee` | 0.55 | 0.002 | 0.96 | 4.00 | 12.08 | 0.38 |
| `ee_to_ttx` | 0.55 | 0.002 | 0.23 | 5.24 | 0.71 | 0.39 |
| `ee_to_wpwm` | 0.55 | 0.002 | 0.80 | 4.11 | 1.39 | 0.39 |
| `ee_to_zh` | 0.55 | 0.002 | 0.13 | 3.78 | 0.42 | 0.43 |
| `uux_to_mumu` | 0.51 | 0.002 | 0.16 | 5.16 | 0.60 | 0.37 |
| `uux_to_uux` | 0.51 | 0.003 | 1.34 | 4.29 | 2.81 | 0.45 |
| `gg_to_ttx` | 0.54 | 0.003 | 3.01 | 4.31 | 3.85 | 0.48 |
| `gg_to_gg` | 0.55 | 0.002 | 2.47 | 4.25 | 7.03 | 0.54 |
| `ee_to_mumua` | 0.55 | 0.004 | 1.28 | 4.92 | 3.92 | 1.12 |
| `ee_to_tatah` | 0.55 | 0.003 | 0.85 | 4.72 | 1.79 | 0.72 |
| `uux_to_epemg` | 0.51 | 0.002 | 3.16 | 5.35 | 7.95 | 1.20 |
| `ddx_to_epemg` | 0.55 | 0.002 | 3.40 | 4.51 | 7.66 | 1.36 |
| `gu_to_epemu` | 0.54 | 0.002 | 3.35 | 4.38 | 10.25 | 1.48 |
| `gux_to_epemux` | 0.55 | 0.002 | 3.32 | 5.57 | 8.78 | 1.14 |
| `ee_to_mumu_tata_qcd0` | 0.55 | 0.010 | 5.83 | 15.52 | 24.17 | 1.74 |
| `ud_to_epemud_qcd0` | 0.51 | 0.013 | 9.40 | 5.28 | 34.17 | 1.58 |
| `uux_to_ccx_emmm_qcd0` | 0.65 | 0.296 | — | 172.84 | — | 3.23 |
| `bbx_to_ccx_emmm_qcd0` | 0.68 | 0.340 | — | 216.53 | — | 7.40 |
| `pp_to_ll` † | 0.52 | 0.008 | 15.82 | 30.84 | 31.76 | 22.98 |
| `pp_to_ll_qcd0` | 0.51 | 0.022 | — | 10.29 | — | 8.83 |
| `pp_to_bb` | 0.55 | 0.010 | 62.94 | 7.03 | 38.00 | 8.02 |
| `pp_to_bb_qcd2` | 0.51 | 0.009 | 78.45 | 23.26 | 46.57 | 8.03 |
| `pp_to_bb_fixed` | 0.55 | 0.009 | 38.59 | 18.30 | 29.89 | 7.80 |
| `pp_to_jj` | — | 0.114 | 174.81 | 25.08 | 125.38 | 8.04 |
| `pp_to_ll_scalefact2` | — | 0.024 | 41.78 | 10.93 | 34.26 | 8.57 |
| `pp_to_llj_fixed` | 0.53 | 0.108 | 87.87 | 15.34 | 149.22 | 9.29 |
| `pp_to_llj` | 0.56 | 0.110 | 186.19 | 14.55 | 129.08 | 9.10 |
| `pp_to_llj_dyn` | — | 0.106 | 116.32 | 14.86 | 127.78 | 9.32 |

† the `pp_to_ll` manifest row's integrals and samples cells compare against the
`dy13_default` MadGraph run, so that is the MG column here; the `pp_to_ll` MadGraph
*directory* is a different, much cheaper run and appears as `pp_to_ll_qcd0`'s twin in
§4.2.

### 5.3 Throughput, on a denominator that means something

Our points are `seeds × neval × niter` VEGAS evaluations, from each row's own report
detail line. MadGraph's are field 4 of `SubProcesses/results.dat` — MadEvent's own count
of phase-space points behind the banked result — divided by that file's
`<cumulated_time>`, which is the **summed CPU seconds of its Fortran jobs** and so removes
the 16-way parallelism from the comparison. Our column is wall time on one thread.

| row | our points | our s | our kpts/s | MG points | MG CPU s | MG kpts/CPU-s | ours/MG |
|---|--:|--:|--:|--:|--:|--:|--:|
| `ee_to_mumu` | 180 000 | 0.17 | 1082 | 236 641 | 6.9 | 34 | 31.7× |
| `ee_to_ee` | 800 000 | 0.96 | 836 | 233 174 | 5.5 | 42 | 19.9× |
| `ee_to_ttx` | 180 000 | 0.23 | 795 | 217 112 | 5.4 | 41 | 19.6× |
| `ee_to_wpwm` | 320 000 | 0.80 | 402 | 228 879 | 5.8 | 39 | 10.2× |
| `ee_to_zh` | 180 000 | 0.13 | 1381 | 219 576 | 4.7 | 46 | 29.7× |
| `uux_to_mumu` | 180 000 | 0.16 | 1106 | 215 607 | 5.4 | 40 | 27.5× |
| `uux_to_uux` | 240 000 | 1.34 | 180 | 262 909 | 5.4 | 49 | 3.7× |
| `gg_to_ttx` | 480 000 | 3.01 | 159 | 278 197 | 5.7 | 48 | 3.3× |
| `gg_to_gg` | 240 000 | 2.47 | 97 | 434 257 | 7.0 | 62 | 1.6× |
| `ee_to_mumua` | 640 000 | 1.28 | 499 | 546 061 | 10.2 | 54 | 9.3× |
| `ee_to_tatah` | 480 000 | 0.85 | 567 | 230 642 | 6.4 | 36 | 15.8× |
| `uux_to_epemg` | 480 000 | 3.16 | 152 | 633 977 | 9.6 | 66 | 2.3× |
| `ddx_to_epemg` | 480 000 | 3.40 | 141 | 557 306 | 8.9 | 63 | 2.3× |
| `gu_to_epemu` | 480 000 | 3.35 | 143 | 196 468 | 6.4 | 31 | 4.6× |
| `gux_to_epemux` | 480 000 | 3.32 | 145 | 181 778 | 6.2 | 29 | 4.9× |
| `ee_to_mumu_tata_qcd0` | 800 000 | 5.83 | 137 | 686 608 | 53.5 | 13 | 10.7× |
| `ud_to_epemud_qcd0` | 960 000 | 9.40 | 102 | 283 965 | 12.9 | 22 | 4.6× |
| `pp_to_ll` † | 8 640 000 | 15.82 | 546 | 3 066 042 | 93.2 | 33 | 16.6× |
| `pp_to_bb` | 9 000 000 | 62.94 | 143 | 387 950 | 8.1 | 48 | 3.0× |
| `pp_to_bb_qcd2` | 9 000 000 | 78.45 | 115 | 401 968 | 8.8 | 45 | 2.5× |
| `pp_to_bb_fixed` | 9 000 000 | 38.59 | 233 | 191 164 | 6.9 | 28 | 8.4× |
| `pp_to_jj` | 9 000 000 | 174.81 | 51 | 229 869 | 10.1 | 23 | 2.3× |
| `pp_to_ll_scalefact2` | 9 000 000 | 41.78 | 215 | 193 558 | 7.2 | 27 | 8.0× |
| `pp_to_llj_fixed` | 9 000 000 | 87.87 | 102 | 176 036 | 9.9 | 18 | 5.8× |
| `pp_to_llj` | 18 000 000 | 186.19 | 97 | 176 272 | 11.0 | 16 | 6.0× |
| `pp_to_llj_dyn` | 9 000 000 | 116.32 | 77 | 176 036 | 10.8 | 16 | 4.8× |

**Geometric mean 6.8× over 26 rows, range 1.6×–31.7×.**

Two caveats sized honestly. First, MadEvent's `results.dat` point count is its own
bookkeeping field; whether it includes the survey pass as well as the refine passes was
not established here, so a systematic factor of order unity sits on every MG column.
Second, our per-point work is not MadGraph's per-point work — ours carries the
multichannel map, the VEGAS grid, the cuts and (on live-draw rows) the configuration
draw; MadGraph's carries its own analogues. Read the column as integrand throughput,
not as a matrix-element ratio: the matrix-element ratio is what
`scripts/mg_perf_compare.sh` measures, and this is not a substitute for it.

**The shape is the finding.** Our advantage is 20–30× on the cheapest 2→2 leptonic rows
and collapses to 1.6–6× on the ones a sprint cares about — `gg_to_gg`, `pp_to_jj`,
`pp_to_llj*`. Whatever the residual systematic on the denominators, it cannot be a
function of process complexity, so the *trend* survives it. The hardest processes are
where we are closest to MadGraph, and they are also where the wall time is.

---

## 6. Chain B's live-draw cost

Note-29 chain B put MadEvent's per-point scale-configuration draw into production: on a
live-draw row each point pays one `eval_amp2` and one `set_alpha_s` before the scale is
clustered. `TODO.md` recorded the cost as unmeasured.

**Isolating it.** A dynamical-against-fixed comparison would answer a different question,
because it also carries the kT clustering and the per-point coupling move. The draw is
gated by `EventScaleSource::draws_configuration()`, which is `sde_strategy == 1 &&
tmin_for_channel == -1.0` — and `SDE_strategy` is read at exactly one place in the crate
(`hadronic.rs:309`). So each probe builds **two integrands from the same run-card text**,
one with `sde_strategy` rewritten to `2`, and asserts `scale_draw_ndim()` is `(1, 0)`.
Everything else — process, cuts, clustering, running coupling, the 20 000 fixed uniform
points — is identical, so the gap is the draw and nothing else. Points the cuts reject
return before the draw, so the cost falls on the surviving fraction while the percentage
is against the same points' total, which is what a budget is spent on.

```
cargo test -p vibegraph-lib --profile release-debug --features extended-validation \
  --test validate_sigma    -- --ignored --nocapture --test-threads=1 probe_scale_draw_cost
cargo test -p vibegraph-lib --profile release-debug --features extended-validation \
  --test validate_hadronic -- --ignored --nocapture --test-threads=1 probe_scale_draw_cost
```

Four repeats each, machine otherwise idle:

| row | without the draw | with it | the draw, per repeat | share of the per-point budget |
|---|--:|--:|--:|--:|
| `gu_to_epemu` | 3761–3895 ns | 4787–4916 ns | **+972, +1021, +1029, +1060 ns** | 20.3 – 22.0% |
| `gux_to_epemux` | 3786–3886 ns | 4822–4886 ns | **+1000, +1007, +1032, +1033 ns** | 20.5 – 21.4% |
| `pp_to_llj_dyn` | 5803–5915 ns | 6021–6068 ns | **+153, +189, +218, +260 ns** | 2.5 – 4.3% |

**≈1.0 µs/point and ~21% on the partonic live-draw rows; ≈0.2 µs/point and ~3% on
`pp_to_llj_dyn`.** The partonic figure is tight to ±5%; the `llj_dyn` one is a small
difference between two ~6 µs numbers and is only good to a factor of ~1.7 across
repeats, so read it as "a few per cent", not as 190 ns. The factor of five between them is structural, not noise: `eval_amp2`
runs for one flavour group, the group whose channel drew the point, while the point's
matrix-element cost on `pp_to_llj_dyn` is a sum over every group and both beam orderings.
The partonic row has one subprocess, so the draw is a second full amplitude pass against
one matrix element.

Sizing it against the sprint: the two partonic rows are ~3.4 s each of the 843 s
integrals total, so removing the draw entirely would save well under 1% of a validation
run — but on a partonic live-draw row it is a fifth of the per-point budget, which makes
it the second-largest single item in that row's per-point cost after the matrix element
itself, ahead of the ~100 ns per-event scale cost already in the backlog. It is not
removable (the draw is what makes the σ agree; note 29 chain B), so the question a
sprint session can ask is whether `eval_amp2` can share work with the `eval_m2` that
follows it on the same momenta.

---

## 7. Profiles

All four recorded with `scripts/profile.sh`, i.e. **`--profile release-debug`, thin LTO,
`extended-validation`**, at samply's default 1000 Hz, with `--unstable-presymbolicate`
so the saved profile carries its own symbols. Durable copies (browse with
`samply load <path>`):

| stage | path |
|---|---|
| integrate, partonic | `target/s3-profiles/integrate-validate_sigma-all-partonic-rows.json.gz` |
| integrate, hadronic | `target/s3-profiles/integrate-validate_hadronic-llj_dyn.json.gz` |
| sample, partonic | `target/s3-profiles/sample-validate_unweighting.json.gz` |
| sample, proton | `target/s3-profiles/sample-cli_generate_proton.json.gz` |

Each has a `.syms.json` sidecar beside it. Percentages below are self time within the
busiest thread; the other ~16 threads are rayon workers parked in `__psynch_cvwait`
(~94% of all samples), which is what a single-threaded integrand under a thread pool
looks like and is itself worth noting.

### 7.1 Where the time sits, grouped

Self time within the busiest thread, grouped by what the symbol belongs to. "evaluator"
is `vibegraph::helas::*`, "phase-space map" is `phasespace::*` (chiefly
`diagram_channel`), "allocator+libc mem" is `malloc`/`free`/`memmove`/`memset` and their
zone internals.

| group | integrate partonic | integrate llj_dyn | sample unweighting | sample proton |
|---|--:|--:|--:|--:|
| busiest-thread work | 40.9 s | 109.3 s | 14.8 s | 27.2 s |
| evaluator (`helas::*`) | **51.4%** | **52.2%** | **49.9%** | **62.5%** |
| PDF interpolation | 0.0% | **14.5%** | 0.0% | **19.4%** |
| phase-space map | 12.9% | 6.2% | 5.5% | 7.0% |
| allocator + libc mem | 12.0% | 6.7% | **18.4%** | 0.6% |
| kT clustering | 8.1% | 5.8% | 9.2% | 0.0% |
| `BTreeMap` (outside the map) | 4.1% | 3.2% | 5.1% | 0.0% |
| libm `log`/`exp`/`pow` | 2.1% | 3.1% | 1.2% | 3.8% |
| LHEF writing | 0.0% | 0.0% | 0.0% | **0.01%** |
| accounted | 90.5% | 91.7% | 89.3% | 93.3% |

### 7.2 One paragraph each

**integrate / `validate_sigma::sigma_gate_matches_madgraph`** — 40.9 s of work; the whole
partonic σ gate, weighted by how hard each row is to integrate. (There is no per-row
filter on this gate: it is one test looping over every banked directory, so the profile
is the σ stage as a whole rather than `gg_to_ttx` alone.) Half the time is the evaluator,
and one symbol carries most of it: `vibegraph::helas::eval::run::fill_arenas` at **34.2%**
of self time — the typed instruction-dispatch loop, with most kernels inlined into it
under thin LTO, so read it as "the matrix element", not as dispatch overhead. Next
largest are the resonance-aware multichannel map (`DiagramChannel::density_at` 4.8%,
`subtree_momentum` 4.1%, `branch_jacobian` 2.4%) at 12.9% together, then **12.0% in the
allocator** plus 4.1% in `BTreeMap` lookups, and the kT clustering at 8.1%
(`merge_tables` 2.9%, `kt::cluster` 2.5%, `setclscales` 1.8%). The named HELAS kernels
that did *not* inline are `ffv_vout_bare` 6.0% and `propagate_f{in,out}_bare` 4.1%.

**integrate / `validate_hadronic::sigma_llj_dynamical_scale_vs_mg`** — 109.3 s, the single
most expensive gate in the layer. Same evaluator share (52.2%, `fill_arenas` 27.8%), but
the second group is now the parton densities: `LogBicubic::xfx_q2` **10.4%** plus
`PdfMember::xfx_q2` **4.0%** = **14.5%** that the partonic profile does not have at all.
The phase-space map drops to 6.2% and the allocator to 6.7% — both diluted rather than
cheaper, because each point now costs a sum over flavour groups and both beam orderings.
The kT clustering is 5.8%. On the hadronic path the integrand is roughly half amplitude,
one seventh PDF, and the rest map + clustering + allocator.

**sample / `validate_unweighting`** — 14.8 s. The evaluator is again half (49.9%,
`fill_arenas` 33.3%), but this is the **most allocation-bound of the four**: 18.4% in the
allocator and libc memory routines plus 5.1% in `BTreeMap`, against 12.0%/4.1% on the
partonic integrate path. The kT clustering is also at its highest share here, 9.2%,
because accept/reject re-derives the per-event scale on every trial. If any profile
argues for the "`ScaleChoice::clustered` heap-allocates its beam–leg candidate list per
event" backlog item, it is this one.

**sample / `cli_generate_proton`** — 27.2 s in the busiest thread of a 51 s test. The
most evaluator-dominated of the four (62.5%, `fill_arenas` 33.7%) and the most
PDF-heavy (**19.4%**: `LogBicubic::xfx_q2` 13.8% + `PdfMember::xfx_q2` 5.6%), because
generation re-evaluates luminosities on every trial. Clustering is **0.0%** — this test
runs the *fixed*-scale `pp_to_llj_fixed` cards, so there is no per-event scale to
cluster. `ProtonIntegrand::shape` shows 1.2% self time. **LHEF writing is 0.01%**:
`lhef::build::SubprocessRecord::event`, `emit::Buffer::emit` and the `EventSource`
adapter together do not reach a tenth of a percent, so event output is not a cost on
this path and needs no sprint attention.

### 7.3 What the four agree on

`fill_arenas` is 28–34% of self time in every one of them, and `helas::*` is 50–63%. PDF
interpolation is 14–19% wherever there are protons and absent otherwise. Allocator
traffic runs 6–18% and peaks on the unweighting pass. Nothing else exceeds 13%. A sprint
that wants a broad win has exactly two targets and they do not overlap: **the evaluator
loop, and `LogBicubic::xfx_q2`**. A third, narrower one — the allocator traffic on the
accept/reject path — is already named in the backlog.

---

## 8. What this note leaves open

- The `refs` stage of `generate_references.sh` (f2py builds, amplitude tables, α_s and
  PDF oracles) is untimed, because timing it means writing into the reference bank.
  The regeneration figure in §4.3 is the `madgraph` stage only.
- Whether MadEvent's `results.dat` point count includes the survey pass. Settling it
  would tighten §5.3's denominators; it does not move the trend.
- Our evaluator *construction* time is inside the integrals and samples rows and is not
  separately timed, so the row that would face MadGraph's `output` + `compile` is
  missing. A finer `duration_s` — one per phase inside a row rather than one per row —
  is the way to get it.
- Every duration here is wall time under whatever the macOS scheduler chose between P-
  and E-cores. §3.3's 0.8% median / 3.4% worst run-to-run spread is the resulting noise
  floor, and it sets the smallest speedup this instrumentation can see.
