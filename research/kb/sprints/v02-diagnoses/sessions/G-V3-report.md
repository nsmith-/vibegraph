---
type: Session Report
title: "G-V3 report (interim): attributing the two macOS heap-corruption aborts"
description: "INCOMPLETE. The unsafe audit and the zstd call census of the integrate child are done; zstd never runs during integration, mimalloc is not linked, the workspace has no unsafe outside unchecked-study. glibc's MALLOC_CHECK_ is inert without libc_malloc_debug.so. The ASan build, the gdb census of validate_hadronic, the Miri reruns and the Linux soak are still running."
status: draft
agent: validation-dev (Opus)
tags: [flaky, heap-corruption, asan, miri, soak, macos, report]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-04}
sources:
  - {id: brief, resource: "G-V3.md", title: "G-V3 brief"}
---
**Status: interim.** This version is committed so a restart cannot lose it. The
runs still in flight are listed under each step. Every number here is from a run
made in this session. The scratch is the session scratchpad, every file
prefixed `G-V3-`.

## Corrections to the brief

1. **`MALLOC_CHECK_=3` alone is a no-op on glibc 2.39.** Since glibc 2.34 the
   checking malloc lives in `libc_malloc_debug.so`, which has to be preloaded.
   A planted 25-byte write into a 24-byte block (`G-V3-mcheck/ctl.c`):

   | environment | outcome |
   |---|---|
   | none | `overflow survived` |
   | `MALLOC_CHECK_=3` | `overflow survived` |
   | `LD_PRELOAD=libc_malloc_debug.so.0 MALLOC_CHECK_=3` | `free(): invalid pointer` (abort) |

   `MALLOC_PERTURB_` works without the preload: a freed 64-byte block read back
   0x5a with `MALLOC_PERTURB_=165`, and 0x00 without it. The soak below preloads
   the library, which `/proc/<pid>/maps` confirms.
2. **Miri cannot run a test that builds a process.** The interned SM blob is
   zstd-compressed and decoded through zstd-sys FFI:
   `unsupported operation: can't call foreign function ZSTD_DStreamInSize`. The
   Miri runs on the evaluator use an uncommitted copy of the workspace whose
   `deserialize_parsed` reads, under `cfg(miri)`, the raw bincode that
   `zstd -d` produced natively (62 240 bytes).
3. **Miri needs `-Zmiri-deterministic-floats`.** Without it, Miri's deliberate
   error on imprecise float operations breaks the bit-exact asserts:
   `setclscales::an_exact_tie_goes_to_the_pair_visited_first` gives
   `left: 249.99999999999991, right: 250.0`, and 4 of the 17 `rewgt` tests fail
   the same way. Neither is UB.

## 1. `unsafe` audit

**The workspace.** `grep -rn '\bunsafe\b'` over `vibegraph-lib`, `vibegraph-cli`
and `validation-report` finds only `helas/eval/run.rs:1053` and `:1069`, the
`get_unchecked` pair behind `#[cfg(feature = "unchecked-study")]`.
`cargo tree -e features -i vibegraph-lib` shows only `default` and
`extended-validation` enabled. There is no `#[global_allocator]` in the
workspace.

**cargo geiger 0.14** (`cargo geiger --output-format Ascii --all-dependencies`
from `vibegraph-cli`, real 7m6s) totals 632/1320 functions and 49496/87531
expressions, used/found. What it cannot see:
- It is not cfg-aware inside a file: it reports `vibegraph-lib` as `2/2` used
  for the gated pair.
- C and asm are invisible to it: `zstd-sys` shows `0/53`, `libmimalloc-sys`
  shows `?`.
- It includes build-time crates.
- "Used" means compiled, not executed on the crash paths.

| crate | unsafe exprs (geiger, used/found) | on the crash paths |
|---|---|---|
| rayon 1.12.0 / rayon-core 1.13.0 | 667/667, 563/566 | work stealing, scope; both stacks |
| crossbeam-deque 0.8.6 / -epoch 0.9.18 / -utils 0.8.21 | 457/457, 426/438, 75/75 | rayon's deques and epoch GC |
| thread_local 1.1.9 | 109/109 | `ThreadLocal<ProtonScratch>`, `FixedBeamScratch`, channel memo: the per-thread arenas whose `Vec`s the stacks grow and free |
| rand 0.9.4 / rand_chacha 0.9.0 / ppv-lite86 0.2.21 | 37/37, 0/0, 632/680 | RNG |
| wide 1.7.1 / safe_arch 1.2.0 / bytemuck 1.25.2 | 0/0, 835/2861, 356/434 | SIMD lane field |
| hashbrown 0.14–0.17, indexmap 2.14.0, smallvec 1.15.2 | 1244–1403 each, 78/83, 555/557 | containers |
| zstd 0.13.3 / zstd-safe 7.2.4 / zstd-sys 2.0.16 (C) | 6/6, 407/542, C | SM blob decode at start; artifact encode at the end of `integrate` |
| libyaml-rs 0.3.0 (via yaml_serde 0.10.7) | 14491/14969 | transpiled libyaml: the PDF `.info` and member headers, once at PDF load |
| egglog-core-relations 2.0.0, dashmap 6.2.1, bumpalo 3.20.3 | 267/267, 348/370, 520/1318 | program folding, before integration |
| nalgebra 0.34.2 | 2241/2549 | reweighting LU/QR |
| tracing-subscriber 0.3.23 / sharded-slab 0.1.7 | 142/142, 147/147 | the CLI child only (`vibegraph-cli/src/logging.rs`); no subscriber in the test binary |
| ring 0.17.14 (C/asm) | 432/444 | ureq TLS, reached only from `fetch.rs` |
| mimalloc 0.1.52 / libmimalloc-sys 0.1.49 (C) | 14/36, C | pulled in by egglog's `bin` feature, **not linked** |

About mimalloc: `nm` counts 0 `mi_` symbols in both the validate_hadronic test
binary and `target/release-debug/vibegraph`. egglog's `#[global_allocator]` is
in its `src/main.rs` only. The PDF reader is in-tree safe Rust with no mmap.
feyngraph has no `unsafe`.

## 2. Which C code runs in the crashing processes

**The validate_hadronic binary links only zstd decompression** (`nm`:
`ZSTD_createDCtx`, `ZSTD_decompressStream`, `ZSTD_freeDCtx`, the DDict
functions). It links no compressor, no mimalloc and no ring.

**The `pp_to_llj_dyn` integrate child, under gdb.** The command line was the
one `validate_samples_proton` spawns: `vibegraph integrate <proc_card>
--run-card validation/madgraph/output/pp_to_llj_dyn/Cards/run_card.dat --neval
300000 --niter 8 --seed 20260731`, with `LHAPDF_DATA_PATH=validation/pdf`.
gdb broke on the six ZSTD create/stream/free entry points. The process exited
normally after 4m27s, with 16 hits, all on thread 1:

- At start-up: `createDCtx` ×1, `decompressStream` ×11, `freeDCtx` ×1. All
  come from `ufo::sm::deserialize_parsed` (`sm.rs:147`) under `sm_model`. That
  is before `ufo_load`, before the PDF load, and before any rayon worker exists.
- At the end: `createCCtx`, `compressStream` and `freeCCtx` ×1 each, from
  `IntegrateArtifact::write_to_path` (`artifact.rs:441`) in `integrate::run`
  (`integrate.rs:877`).

No zstd code runs during VEGAS warm-up, which is where the 2026-09-07 abort
happened. The only zstd that runs earlier is one decode of a fixed
`include_bytes!` blob, which is deterministic.

*Pending:* the same census over a full validate_hadronic run.

## 3. Sanitizers

**Planted control, caught.** The environment was `RUSTFLAGS="-Zsanitizer=address
-Cforce-frame-pointers=yes" CC=clang CFLAGS="-fsanitize=address
-fno-omit-frame-pointer -g"`, with nightly 1.101 (2026-10-27) `-Zbuild-std` and
clang 18.1.3. The control is C compiled through the `cc` crate, as zstd-sys's
is.

| control | ASan report |
|---|---|
| `overflow` | `heap-buffer-overflow ... in c_overflow overflow.c:6:46` |
| `uaf` | `heap-use-after-free ... in c_use_after_free overflow.c:15:10` |
| `rust-overflow` | `heap-buffer-overflow ... main.rs:14:17` |
| `clean` | `result 3`, no report |

*Pending:* the ASan build of the CLI and of validate_hadronic, its runs, the
`nm` check of zstd-sys's objects, and TSan.

## 4. Miri

**Harness for the scratch pattern** (`G-V3-mirih`, the workspace lockfile's
rayon 1.12.0, thread_local 1.1.9 and crossbeam-epoch 0.9.18). It drives a
`ThreadLocal<Scratch{RefCell<Vec<f64>>, RefCell<Vec<[f64;4]>>}>`, grown on
demand under `into_par_iter`. Each point builds and drops a `Vec<f64>`.
Short-lived scoped threads recycle `thread_local` ids, both on their own and
alongside a rayon pass.

- **Stacked Borrows** reports UB in **crossbeam-epoch 0.9.18**,
  `internal.rs:549`, `Local::element_of`:
  `&*(entry as *const Entry).cast::<Local>()` retags a pointer to the first
  field over the whole struct. It is reached from `Global::try_advance` on a
  rayon worker, at failing seed 5. This is an aliasing-model violation. It is
  not a write out of bounds or after free, so it cannot corrupt a free list.
- **Tree Borrows**, seeds 0..32: no UB. Seed 30 tripped only the leak check,
  because rayon's global pool is detached.
- **Tree Borrows with `-Zmiri-ignore-leaks`**, seeds 0..64: no UB in the seeds
  that finished before the run was stopped at its time limit.

**kt clustering:** `kt::tests::a_forced_line_is_tagged_whatever_its_width` ok.
The `rewgt` and `setclscales` tests showed no UB. Their failures are the float
noise of correction 3.

*Pending:* `coupling::cluster::` in full, `arena_reuse::a_stale_fill_is_never_reused`
and `arena_reuse::fills_do_not_cross_threads`, all under Tree Borrows with
deterministic floats.

## 5. Linux soak

*Running.* The loop runs a copy of the acec723 validate_hadronic test binary
with `--nocapture` under
`LD_PRELOAD=libc_malloc_debug.so.0 MALLOC_CHECK_=3 MALLOC_PERTURB_=1+(37i mod 254)`.
One row per run goes to `G-V3-soak/results.tsv`, and the full log is kept for
every run that fails.

How to read a clean count N: the 95 % upper limit on the per-run abort rate is
3/N. If the prior rate of 1 in 70 holds, the chance of N clean runs is e^(−N/70):
37 % at N = 70, 5 % at N = 210.

## 6. The M3 soak request (posted on #89)

1. **Binary.** If the exact crashing binary from `47ac21a` is still on the M3,
   use it: it is the binary whose abort rate is being measured. Otherwise build
   from the sprintg-diagnoses tip carrying this report, which is code-identical
   to `acec723`:
   - Fetch the inputs:
     `pixi run fetch-pdf && pixi run fetch-refdata && pixi run init-sm-submodule`
   - Build, and take the printed path as `BIN`:
     `cargo test -p vibegraph-lib --profile release-debug --features extended-validation --test validate_hadronic --no-run 2>&1 | sed -n 's/.*Executable .*(\(.*\))/\1/p'`
   - Record `git rev-parse HEAD`, `shasum -a 256 $BIN`, `sw_vers`,
     `sysctl -n machdep.cpu.brand_string`, and
     `nm $BIN | grep -c ' _mi_'` (expect 0).
2. **Two arms, interleaved A, B, A, B, …:**
   - A: the default environment (nano allocator).
   - B: `MallocNanoZone=0`.

   Run nothing heavy alongside. Use the default test and rayon thread counts.
3. **Runs per arm:** at least 100 in a first tranche, ideally 300. At 1/70,
   300 runs give an expected 4.3 aborts in arm A, and the chance of none is
   about 1.4 %.
4. **Each run:**
   - Before the run, from `vibegraph-lib`:
     `ls ~/Library/Logs/DiagnosticReports/ > before.txt`
   - Run it: `env [MallocNanoZone=0] "$BIN" --nocapture > run-$i-$arm.log 2>&1; rc=$?`
   - Append `i arm start_utc wall_s rc` to `results.tsv`. An `rc` of 134 is
     SIGABRT.
   - If `rc ≥ 128`: keep the log, list the new reports with
     `ls ~/Library/Logs/DiagnosticReports/ | comm -13 before.txt -`, and copy
     every new `validate_hadronic-*.ips`.
5. **Post on #89:**
   - `results.tsv`;
   - each `.ips`, or at least its crashing-thread backtrace, its
     `exception`/`termination` fields and its `vmRegionInfo`;
   - the last 100 lines of the matching run log;
   - the identity block from step 1.

## Found

- **Soaks that set `MALLOC_CHECK_` on glibc ≥ 2.34 without preloading
  `libc_malloc_debug.so` ran without heap checking.** Evidence: correction 1.
  To close: preload the library wherever `MALLOC_CHECK_` is used, and re-read
  earlier Linux evidence that relied on it.
- **crossbeam-epoch 0.9.18 violates Stacked Borrows at `internal.rs:549`.**
  This code runs in every rayon program, and Tree Borrows accepts it. It is a
  provenance issue, not a corrupting write. To close: find or file the issue
  upstream.
- **Miri cannot run any library test that builds a process,** because the
  interned SM model is decoded through zstd-sys FFI. To close: a `cfg(miri)`
  path to an uncompressed blob, or a pure-Rust decoder for tests.
- **libyaml-rs 0.3.0, transpiled C with about 14.5k unsafe expressions, runs in
  both crashing processes.** Its input is deterministic, so one clean ASan run
  covers it. To close: name it in the item's "ruled out" list once that run is
  recorded.
