---
type: Session Report
title: "G-V3 report: neither macOS heap-corruption abort is attributable to code that Linux tools can see"
description: "No corrupting write was found. Every C and asm path in the two crashing processes runs outside the crash window or deterministically, and ASan, valgrind, Miri and TSan are clean on them; mimalloc is not linked; the workspace has no unsafe outside unchecked-study. What remains is the macOS side (the nano allocator or the host), so the M3 soak with and without the nano zone stays the closing measurement. glibc's MALLOC_CHECK_ needs libc_malloc_debug.so preloaded."
status: draft
agent: validation-dev (Opus)
tags: [flaky, heap-corruption, asan, tsan, miri, valgrind, soak, macos, report]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-04}
sources:
  - {id: brief, resource: "G-V3.md", title: "G-V3 brief"}
---
This was a diagnosis session, and it commits no source change. Every number
below comes from a run made in this session on the 4-core Linux container.
Two sibling sessions shared the host, so load averages ran at 12–16. The
source tree is `acec723`. The scratch is in the session scratchpad, and every
file there is prefixed `G-V3-`.

**Status at hand-back.** Every run is final except the Linux soak, which is still running detached (driver PID 18175). To stop it: `touch <scratchpad>/G-V3-soak/STOP`.

## Verdict

**No corrupting write was found.** The table shows what the Linux tools rule
out. Three explanations remain:
- the macOS nano allocator;
- the M3 host;
- a write that depends on timing and is too rare for these runs to catch.

The validate_hadronic item closes on its second condition, an M3 soak with and
without the nano zone (§6). The proton-sample item has nothing reproducible
left to attack on Linux.

| suspect | evidence | rules out | cannot rule out |
|---|---|---|---|
| workspace `unsafe` | the only `unsafe` sits behind `unchecked-study`, which no build enables (§1) | the crate's own `unsafe` | — |
| a non-System allocator | `nm`: 0 `mi_` symbols in either binary; egglog's `#[global_allocator]` is in its `main.rs` only (§1) | mimalloc, or any custom allocator | — |
| zstd-sys C | ASan instruments 31 of the 37 `libzstd.a` objects, with 182 checks in `ZSTD_decompressStream`; both processes ran clean (§3) | a deterministic overflow in the decoder | a write inside zstd's own buffers |
| zstd x86-64 Huffman asm, which ASan does not instrument | the asm loop runs once in the blob decode (gdb); valgrind memcheck is clean on that decode and catches a planted overflow (§3) | the asm decode of this blob | the macOS arm64 build, which has no such asm |
| zstd running during the crash | gdb census: one decode at start-up, before any worker exists, and one encode after integration in `integrate` (§2) | zstd running inside the crash window | — |
| rayon, crossbeam, thread_local, Vec growth | Miri: the scratch-pattern harness, all 36 `coupling::cluster` tests, `a_stale_fill_is_never_reused`, `fills_do_not_cross_threads`. TSan: `sigma_llj_dynamical_scale_vs_mg` (§3, §4) | UB on the interleavings explored; races in that test | other interleavings; crossbeam-epoch's int-to-pointer casts also weaken Miri there |
| everything in the process | ASan over the `pp_to_llj_dyn` integrate and over validate_hadronic; Linux soak under glibc's checking malloc (§3, §5) | a corruption that happens on most runs | a rare one |

## Corrections to the brief

1. **`MALLOC_CHECK_=3` alone is a no-op on glibc 2.39.** Since glibc 2.34 the
   checking malloc lives in `libc_malloc_debug.so`, which has to be preloaded.
   The test was a planted 25-byte write into a 24-byte block
   (`G-V3-mcheck/ctl.c`):

   | environment | outcome |
   |---|---|
   | none | `overflow survived` |
   | `MALLOC_CHECK_=3` | `overflow survived` |
   | `LD_PRELOAD=libc_malloc_debug.so.0 MALLOC_CHECK_=3` | `free(): invalid pointer` (abort) |

   `MALLOC_PERTURB_` does work without the preload: a freed 64-byte block read
   back as 0x5a with `MALLOC_PERTURB_=165`, and as 0x00 without it. The soak
   preloads the library, and `/proc/<pid>/maps` and `/proc/<pid>/environ`
   confirm it is loaded.
2. **Miri cannot run a test that builds a process.** The interned SM blob is
   decoded through zstd-sys over FFI, so Miri stops with
   `unsupported operation: can't call foreign function ZSTD_DStreamInSize`.
   The evaluator's Miri runs therefore use an uncommitted copy of the
   workspace (`G-V3-miriws`). In that copy, `deserialize_parsed` under
   `cfg(miri)` reads the raw bincode that `zstd -d` produced natively. That
   file is 62 240 bytes, and its FNV-1a hash equals that of the crate's own
   decode.
3. **Miri needs `-Zmiri-deterministic-floats`.** Without it, Miri's deliberate
   error on imprecise float operations breaks the bit-exact asserts. For
   example, `setclscales::an_exact_tie_goes_to_the_pair_visited_first` gives
   `left: 249.99999999999991, right: 250.0`, and 4 of the 17 `rewgt` tests
   fail the same way. None of these failures is UB.
4. **Instrumenting zstd-sys's C with clang ASan still leaves its asm
   unchecked.** `huf_decompress_amd64.S` is hand-written asm, and on x86-64 it
   is the decoder's hot Huffman loop. Valgrind covers it (§3).

## 1. `unsafe` audit

**The workspace.** `grep -rn '\bunsafe\b'` over `vibegraph-lib`,
`vibegraph-cli` and `validation-report` finds only two lines:
`helas/eval/run.rs:1053` and `:1069`. They are the `get_unchecked` pair behind
`#[cfg(feature = "unchecked-study")]`. `cargo tree -e features -i
vibegraph-lib` shows only `default` and `extended-validation` enabled. The
workspace has no `#[global_allocator]`.

**cargo geiger 0.14.** The run was `cargo geiger --output-format Ascii
--all-dependencies` from `vibegraph-cli`, real 7m6s. Its totals, used/found
for functions, expressions, impls, traits and methods, are
`632/1320 49496/87531 787/1134 77/80 1221/3965`. What it cannot see:
- It is not cfg-aware inside a file, so it reports `vibegraph-lib` as `2/2`
  used for the gated pair.
- C and asm are invisible to it: `zstd-sys` shows `0/53` and
  `libmimalloc-sys` shows `?`.
- It counts build-time crates.
- "Used" means compiled, not executed on the crash paths.

The crates on the crash paths:

| crate | unsafe exprs (geiger, used/found) | role on the crash paths |
|---|---|---|
| rayon 1.12.0 / rayon-core 1.13.0 | 667/667, 563/566 | work stealing and scope; both stacks |
| crossbeam-deque 0.8.6 / -epoch 0.9.18 / -utils 0.8.21 | 457/457, 426/438, 75/75 | rayon's deques and epoch GC |
| thread_local 1.1.9 | 109/109 | `ThreadLocal<ProtonScratch>`, `FixedBeamScratch` and the channel memo: the per-thread arenas whose `Vec`s the stacks grow and free |
| rand 0.9.4 / rand_chacha 0.9.0 / ppv-lite86 0.2.21 | 37/37, 0/0, 632/680 | random number generation |
| wide 1.7.1 / safe_arch 1.2.0 / bytemuck 1.25.2 | 0/0, 835/2861, 356/434 | the SIMD lane field |
| hashbrown 0.14–0.17, indexmap 2.14.0, smallvec 1.15.2 | 1244–1403 each, 78/83, 555/557 | containers |
| zstd 0.13.3 / zstd-safe 7.2.4 / zstd-sys 2.0.16 (C and asm) | 6/6, 407/542, C | the SM blob decode at start-up; the artifact encode at the end of `integrate` |
| libyaml-rs 0.3.0 (via yaml_serde 0.10.7) | 14491/14969 | transpiled libyaml; parses the PDF `.info` and member headers once, at PDF load |
| egglog-core-relations 2.0.0, dashmap 6.2.1, bumpalo 3.20.3 | 267/267, 348/370, 520/1318 | program folding, before integration |
| nalgebra 0.34.2 | 2241/2549 | the reweighting LU and QR |
| tracing-subscriber 0.3.23 / sharded-slab 0.1.7 | 142/142, 147/147 | the CLI child only (`vibegraph-cli/src/logging.rs`); the test binary installs no subscriber |
| ring 0.17.14 (C and asm) | 432/444 | ureq's TLS, reached only from `fetch.rs` and never from `integrate` |
| mimalloc 0.1.52 / libmimalloc-sys 0.1.49 (C) | 14/36, C | pulled in by egglog's `bin` feature, but **not linked** |

Three further checks:
- **mimalloc.** `nm` counts 0 `mi_` symbols in both the validate_hadronic test
  binary and `target/release-debug/vibegraph`. egglog's `#[global_allocator]`
  is in its `src/main.rs` only.
- **The PDF reader** is in-tree safe Rust and does not use mmap.
- **feyngraph** contains no `unsafe`.

## 2. Which C code runs in the crashing processes, and when

**What each binary links.** `nm` shows the following:
- The validate_hadronic test binary links only zstd's decompressor:
  `ZSTD_createDCtx`, `ZSTD_decompressStream`, `ZSTD_freeDCtx` and the DDict
  functions. It links no compressor, no mimalloc and no ring.
- The CLI links both zstd directions, and ring.

**The census.** gdb set breakpoints on the ZSTD create, stream and free entry
points.

**validate_hadronic** (`G-V3-gdb-zstd-hadronic.log`):
- Outcome: `test result: ok. 20 passed; 0 failed; 26 ignored`, real 43m33s.
- There were 13 hits, all on one libtest worker thread: `createDCtx` ×1,
  `decompressStream` ×11 and `freeDCtx` ×1.
- All 13 are in `ufo::sm::deserialize_parsed` (`sm.rs:147`), under the
  `OnceLock` in `sm_parsed`. That is the first `sm_model` call, before any
  amplitude exists.
- Nothing else calls zstd in this process.

**The `pp_to_llj_dyn` integrate child** (`G-V3-gdb-zstd-llj.log`). The command
line is the one `validate_samples_proton` spawns:

```
vibegraph integrate <proc_card> \
  --run-card validation/madgraph/output/pp_to_llj_dyn/Cards/run_card.dat \
  --neval 300000 --niter 8 --seed 20260731
```

It ran with `LHAPDF_DATA_PATH=validation/pdf`.
- Outcome: `exited normally`, real 4m27s.
- There were 16 hits, all on the main thread.
- At start-up there are `createDCtx` ×1, `decompressStream` ×11 and `freeDCtx`
  ×1, all in `deserialize_parsed`. They come before `ufo_load`, before the PDF
  load and before any rayon worker exists.
- After integration there are `createCCtx`, `compressStream` and `freeCCtx`,
  one each. They are in `IntegrateArtifact::write_to_path`
  (`artifact.rs:441`), called from `integrate::run` (`integrate.rs:877`).

**Conclusion.** No zstd code runs inside either crash window: not in
`eval_m2` in validate_hadronic, and not in VEGAS warm-up in the integrate
child. The only zstd work before either crash is a single decode of a fixed
`include_bytes!` blob.

## 3. Sanitizers and valgrind

### Builds

The toolchain was nightly cargo 1.101.0 (2026-10-27) with `-Zbuild-std
--target x86_64-unknown-linux-gnu`, profile `release-debug`, and clang 18.1.3.
- **ASan** used `RUSTFLAGS="-Zsanitizer=address -Cforce-frame-pointers=yes"
  CC=clang CFLAGS="-fsanitize=address -fno-omit-frame-pointer -g"`. The build
  took 33m34s.
- **TSan** used the same flags with `thread` in place of `address`. The build
  took 19m45s.

### How much of zstd-sys ASan instruments

- 31 of the 37 objects in `libzstd.a` reference `__asan_report*`, 264
  references in all.
- In both final binaries, the CLI and the test, `ZSTD_decompressStream` alone
  carries 182 `__asan_report` calls.
- Six objects are not instrumented:
  - five that contain no instrumentable heap accesses: `debug`,
    `error_private`, `pool`, `threading` and `zstd_common`;
  - the asm in `huf_decompress_amd64` (correction 4).

### Planted controls

The controls are C compiled through the `cc` crate, the same way zstd-sys's C
is built, plus one Rust case. They ran with the same option strings as the
real runs, including `log_path`.

| control | what the sanitizer reported |
|---|---|
| ASan `overflow` (C) | `heap-buffer-overflow ... in c_overflow overflow.c:6:46`, written to `asan.<pid>`; rc 134 |
| ASan `uaf` (C) | `heap-use-after-free ... in c_use_after_free overflow.c:15:10` |
| ASan `rust-overflow` | `heap-buffer-overflow ... main.rs:14:17` |
| TSan `race` (C: two pthreads incrementing an unsynchronised `long`) | `WARNING: ThreadSanitizer: data race ... bump overflow.c:20:79`, written to `tsan.<pid>` |
| `clean` | `result 3`, and no report |

### Runs

**ASan, the `pp_to_llj_dyn` integrate** (same command line as in §2):
- exit 0, real 44m46s;
- no `G-V3-asan-report.*` file was written;
- it gave `σ = 416.012730 ± 0.404316 pb`, identical in every printed digit to
  the uninstrumented run, so the instrumented process followed the same path.

**ASan, validate_hadronic:** `test result: ok. 20 passed; 0 failed; 26 ignored`, exit 0, real 182m3s, no `G-V3-asan-report.*` file. (The panic line in the log is the `#[should_panic]` test `lepton_part_refuses_an_unknown_directory_name`.)

**TSan, `sigma_llj_dynamical_scale_vs_mg`.** This test runs the `pp_to_llj_dyn`
card, whose dynamical scale goes through the kT clustering under rayon.
- `test result: ok. 1 passed`, real 86m16s;
- no `G-V3-tsan-report.*` file was written;
- the full suite was not run under TSan, which slows it about 15×.

**Valgrind memcheck on the zstd decode, which covers the asm.** `G-V3-zdec`
decodes the interned blob exactly as `deserialize_parsed` does.
- gdb confirms that `HUF_decompress4X1_usingDTable_internal_fast_asm_loop`
  runs once during this decode.
- The output is 62 240 bytes with FNV-1a `938c60696706b712`, the same as
  `zstd -d`.
- `valgrind --error-exitcode=99` returns 0 on the decode.
- With a planted one-byte heap overflow added, it returns 99 and reports
  `Invalid write of size 1 ... main.rs:9`.

### What this does not cover

The M3's arm64 build has no x86-64 asm, and macOS builds zstd's C with its own
compiler, so none of the above runs that code. Even so, the decode is a fixed
input through a deterministic decoder, and on Linux it is clean at both the C
level (ASan) and the machine-code level (valgrind). A defect in it that showed
up once in about 70 runs would be hard to square with that.

## 4. Miri

Nightly miri 0.1.0 (2026-11-03).

### The scratch-pattern harness

`G-V3-mirih` uses the workspace lockfile's rayon 1.12.0, rayon-core 1.13.0,
thread_local 1.1.9 and crossbeam-epoch 0.9.18. It drives three things:
- a `ThreadLocal<Scratch{RefCell<Vec<f64>>, RefCell<Vec<[f64;4]>>}>` grown on
  demand under `into_par_iter`;
- a per-point `Vec<f64>` that is built, has `remove(0)` called on it, and is
  dropped;
- short-lived scoped threads that recycle `thread_local` ids, both on their own
  and alongside a rayon pass.

| Miri flags | result |
|---|---|
| Stacked Borrows, `-Zmiri-many-seeds=0..8` | UB in **crossbeam-epoch 0.9.18**, `internal.rs:549`, `Local::element_of` (failing seed 5) |
| Tree Borrows, `-Zmiri-many-seeds=0..32` | both tests ok on every seed; seed 30 tripped only the leak check (rayon's global pool is detached) |
| Tree Borrows with `-Zmiri-ignore-leaks`, seeds `0..64` | no UB in any seed that finished; the sweep was stopped at the 30-minute tool limit |

The Stacked Borrows report comes from
`&*(entry as *const Entry).cast::<Local>()`. It retags a pointer to the first
field over the whole struct, and is reached from `Global::try_advance` on a
rayon worker. This breaks the aliasing model only: there is no write out of
bounds or after a free, so it cannot corrupt a free list.

### Library tests

These ran in `G-V3-miriws` with `-Zmiri-disable-isolation -Zmiri-tree-borrows
-Zmiri-deterministic-floats -Zmiri-ignore-leaks`.

| test | what it exercises | result | wall |
|---|---|---|---|
| `coupling::cluster::` | all 36 tests: configs, graph, kt, rewgt, setclscales | `ok. 36 passed; 0 failed` | 59m19s |
| `helas::eval::run::arena_reuse::a_stale_fill_is_never_reused` | one workspace refilled through `ensure_sizes` and `fill_arenas`: new points, a rewritten pool, a second instance | `ok. 1 passed` | 25m49s |
| `helas::eval::run::arena_reuse::fills_do_not_cross_threads` | 4 threads, each with forked amplitudes and its own scratch | `ok. 1 passed` | 24m36s |

No UB was reported. Miri printed 26 `integer-to-pointer cast` warnings, from
crossbeam-epoch's `atomic.rs:204`. For that code, Miri says it "might miss
pointer bugs".

**What Miri covers:** the `unsafe` that these paths actually execute,
including thread_local's, rayon's, crossbeam's and `Vec`'s, on the
interleavings Miri explores.

**What Miri does not cover:**
- C code and the real allocator;
- full-size workloads;
- interleavings it did not explore.

## 5. Linux soak

`G-V3-soak/soak.sh` loops a copy of the `acec723` validate_hadronic
release-debug test binary, run with `--nocapture`. Each run gets:

```
LD_PRELOAD=libc_malloc_debug.so.0 MALLOC_CHECK_=3 MALLOC_PERTURB_=1+(37i mod 254)
```

It writes one row per run to `results.tsv` and keeps the full log of every run
that fails.

**At hand-back, still running:** 5 runs finished, 5 of them `ok` (`test result: ok. 20 passed; 0 failed; 26 ignored`), at 2406–3389 s a run under load 6–16. The target of 70 runs was not reached: at this rate it needs about 2 more days.

**How to read a count of N clean runs:**
- The 95 % upper limit on the per-run abort rate is 3/N.
- If the prior of one abort in about 70 runs held on this host, N clean runs
  would happen with probability e^(−N/70): 37 % at N = 70, 5 % at N = 210.
- glibc's checking malloc is not the macOS nano zone. A clean Linux soak says
  nothing about a defect in the macOS allocator.

## 6. The M3 soak request (posted on #89)

**1. The binary.** If the exact crashing binary from `47ac21a` is still on the
M3, use it: it is the binary whose abort rate is being measured. Otherwise,
build from the sprintg-diagnoses tip carrying this report, which is
code-identical to `acec723`:

1. Fetch the inputs:
   `pixi run fetch-pdf && pixi run fetch-refdata && pixi run init-sm-submodule`
2. Build, and take the printed path as `BIN`:
   `cargo test -p vibegraph-lib --profile release-debug --features extended-validation --test validate_hadronic --no-run 2>&1 | sed -n 's/.*Executable .*(\(.*\))/\1/p'`
3. Record, as the identity block:
   - `git rev-parse HEAD`
   - `shasum -a 256 $BIN`
   - `sw_vers`
   - `sysctl -n machdep.cpu.brand_string`
   - `nm $BIN | grep -c ' _mi_'`, which should print 0.

**2. Two arms, interleaved A, B, A, B, …**
- Arm A: the default environment, which uses the nano allocator.
- Arm B: `MallocNanoZone=0`.

Run nothing heavy alongside, and keep the default test and rayon thread
counts.

**3. Runs per arm.** At least 100 in a first tranche, ideally 300. At a rate of
1/70, 300 runs give an expected 4.3 aborts in arm A, and the chance of seeing
none is about 1.4 %.

**4. Each run:**
1. In `vibegraph-lib`, record the existing crash reports:
   `ls ~/Library/Logs/DiagnosticReports/ > before.txt`
2. Run the test:
   `env [MallocNanoZone=0] "$BIN" --nocapture > run-$i-$arm.log 2>&1; rc=$?`
3. Append `i arm start_utc wall_s rc` to `results.tsv`. An `rc` of 134 means
   SIGABRT.
4. If `rc ≥ 128`: keep the log, list the new reports with
   `ls ~/Library/Logs/DiagnosticReports/ | comm -13 before.txt -`, and copy
   every new `validate_hadronic-*.ips`.

**5. What to post on #89:**
- `results.tsv`;
- each `.ips`, or at least its crashing-thread backtrace, its
  `exception`/`termination` fields and its `vmRegionInfo`;
- the last 100 lines of the matching log;
- the identity block from step 1.

## Commits

- `0a80500`: the interim version of this report.
- This version.

## Found

- **`MALLOC_CHECK_` soaks without the preload.** On glibc ≥ 2.34, soaks that
  set `MALLOC_CHECK_` without preloading `libc_malloc_debug.so` ran with no
  heap checking. Evidence: correction 1. To close: preload the library
  wherever `MALLOC_CHECK_` is used, and re-read any earlier Linux evidence
  that relied on it.
- **crossbeam-epoch 0.9.18 under Stacked Borrows.** It violates Stacked
  Borrows at `internal.rs:549`, in code that runs in every rayon program. Tree
  Borrows accepts it. It is a provenance issue, not a corrupting write. To
  close: find or file the issue upstream.
- **Miri and the interned SM model.** Miri cannot run any library test that
  builds a process, because the model is decoded through zstd-sys FFI. To
  close: a `cfg(miri)` path to an uncompressed blob, or a pure-Rust decoder in
  tests, so the evaluator's unit tests run under Miri without a patched copy.
- **libyaml-rs 0.3.0.** It is transpiled C with about 14.5k unsafe
  expressions, and it runs in both crashing processes. Its input is
  deterministic, and ASan instrumented it both in this session's runs and in
  the 2026-10-27 run. To close: name it in the item's "ruled out" list.
- **Long runs must be detached from the start.** The tools' 30-minute
  background limit and 2-hour waiter limit kill sanitizer and soak runs.
  Briefs for this kind of work should require detaching them (`os.setsid`
  double fork) from the start.
