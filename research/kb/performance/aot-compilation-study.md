---
type: Feasibility Study
title: Compiling the helicity program ahead of time
description: "Rendering the bytecode to Rust (inlined, outlined, MadGraph slot-array form): small programs gain up to 2×, the 2→6 loses to instruction footprint in every form."
status: draft
tags: [performance, evaluator, aot-compilation, code-size, interpreter]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: aot-summary, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/aot-kernels-study-results.md#L13-L51", title: "AOT study: question and answer"}
  - {id: aot-method, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/aot-kernels-study-results.md#L52-L111", title: "AOT study §1–2: host, method, correctness"}
  - {id: aot-timings, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/aot-kernels-study-results.md#L112-L193", title: "AOT study §3–4: timings, code size and codegen"}
  - {id: aot-reading, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/aot-kernels-study-results.md#L194-L289", title: "AOT study §5–7 and reproduce: reading, compile cost, caveats"}
  - {id: mg-form, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/aot-kernels-study-results.md#L290-L432", title: "MadGraph-form study M1–M4"}
  - {id: mg-form-chunked, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/aot-kernels-study-results.md#L433-L587", title: "MadGraph-form study M5–M9"}
measured:
  - {commit: 03c31e6, pr: 17, landed_in: aeb96a7, host: "Intel Xeon Emerald Rapids (family 6 model 207), 4-vCPU Firecracker VM", command: "RUSTFLAGS=\"-C target-cpu=native\" cargo bench -p vibegraph-lib --features aot-study --bench aot_kernels"}
  - {commit: b504391, pr: 17, landed_in: aeb96a7, host: "Intel Xeon Emerald Rapids (family 6 model 207), 4-vCPU Firecracker VM", command: "RUSTFLAGS=\"-C target-cpu=native\" CARGO_BUILD_JOBS=2 cargo bench -p vibegraph-lib --profile aot-study --features aot-mg-study-large --bench aot_kernels"}
---

# Compiling the helicity program ahead of time

The question: how fast does the computation `fill_arenas` interprets run when
its bytecode program is rendered to Rust as straight-line kernel calls and
compiled by rustc/LLVM? The answer depends on program size, and the dividing
line is **instruction footprint, not arithmetic**. Small programs gain 1.3–2×;
the 2→6 (`uux_to_ccx_emmm_qcd0`, 36 523 VM instructions) loses in every
rendering tried. The study code is not in the tree: it lives at commits
`03c31e6` (by-value renderings) and `b504391` (MadGraph form), removed because
its large builds take 11–52 min.[^aot-summary]

For the interpreter being compared against, see
[the helicity program layout](../performance/evaluator-program-layout.md); for
how this study's lower bound sits beside the other overhead estimates, see
[the interpreter overhead budget](../performance/interpreter-overhead-budget.md).

## Three renderings

`helas::eval::aot::render` walked a compiled, helicity-pruned evaluator's
`folded_hel()` program in production (op-blocked) order. Arena slots were
resolved symbolically, so the rendered dataflow was the interpreter's by
construction, slot recycling included.[^aot-method]

| form | what each VM instruction becomes | code per VM instr (`f64`) |
|---|---|--:|
| `aot` (inlined) | `let` binding a local to the same `kernel::*_bare` call or operator expression, kernels inlined | ~11–28 machine instrs, up to ~190 B |
| `aot_out` (outlined, "by value") | same, but each kernel behind an `#[inline(never)]` wrapper | ~6.5 machine instrs, ~41–45 B |
| MadGraph form (`aot::mg`) | one `#[inline(never)]` in-place call: result written through `&mut` into a slot array, every operand a reference into a slot array or pool | ~5 machine instrs, 26–38 B |

The MadGraph form mirrors MadGraph's Fortran matrix elements: every
wavefunction in one memory-resident array, every HELAS call an out-of-line
routine taking operands by reference and writing in place. It used the
interpreter's own `Program::arena_sizes` and `Program::dest`, so slot
assignment and arena footprint matched exactly. Constant-pool loads emitted no
call (a later read names the pool entry, as MadGraph passes a coupling from its
coupling array).[^mg-form]

## Correctness

Every rendered row reproduced the interpreter's |M|² **bit for bit** (`to_bits`)
at `f64` and `LaneField<4>`, under every binding, on the bench's 16 points. That
is what the construction predicts: the same kernels and operator expressions in
the same operand order, no fast-math, and Rust does not contract `a*b + c` into
an FMA. Moving instructions between functions or inlining them changes no
rounding. A 1e-7 move of one coupling moved the rendered |M|², so the rendered
code reads the bound pools.[^aot-method]

Blind spot: an instruction whose result is never read. `gg_to_gg` has four such
instructions (complex products under the configuration bundle), which the
rendered code drops as dead and the interpreter computes.

## Timings

Emerald Rapids VM, `rustc 1.97.0`, `-C target-cpu=native`, `bench` profile
(fat LTO) unless noted; ns/event, minimum over rounds; ratio `vm / arm` (above 1:
rendered is faster). See [benchmark hosts](../performance/benchmark-hosts.md).

By-value renderings, fat LTO (run 2 / run 3 ratios):[^aot-timings]

| row | VM instrs | `vm_f64` ns | `aot_f64` | `aot_out_f64` | `aot_lanes4` |
|---|--:|--:|--:|--:|--:|
| `ee_to_mumu` | 66 | 451 | 1.55 / 1.39 | 1.63 / 1.45 | 1.87 / 1.80 |
| `gg_to_gg` | 695 | 2 413 | 1.34 / 1.30 | 2.04 / 1.96 | 1.26 / 1.50 |
| `ee_to_mumu_tata_qcd0` | 1 756 | 8 139 | 0.79 / 0.80 | 1.30 / 1.37 | 0.97 / 0.93 |
| 2→6, one function | 36 523 | 166 189 | 0.20 / 0.21 | 0.74 / 0.74 | 0.29 / 0.30 |
| 2→6, chunks of 2 000 | | 163 395 | 0.20 / 0.20 | 0.53 / 0.52 | 0.22 / 0.22 |

MadGraph form, 2→6 in 19 functions of 2 000 instructions, thin-LTO `aot-study`
profile (all arms of that binary share the profile):[^mg-form-chunked]

| row | `mg_f64` ratio | `byv_f64` ratio | `mg_lanes4` ratio |
|---|--:|--:|--:|
| `ee_to_mumu` | 1.18 | 1.69 | 1.29 |
| `gg_to_gg` | 1.05 | 2.22 | 1.34 |
| `ee_to_mumu_tata_qcd0` | 1.12 | 1.67 | 1.33 |
| `uux_to_ccx_emmm_qcd0` | **0.66** | 0.74 (one function, fat LTO) | **0.79** |

A second run of the same binary read the 2→6 at 0.66 again (0.67 at lanes4);
the small-row MadGraph-form gains (1.05–1.27 across runs) are inside the host's
per-cell layout noise but point the same way on every row.

## Code size

| function (`f64`) | bytes | per VM instr | stack refs | calls |
|---|--:|--:|--:|--:|
| `fill_arenas` (the interpreter, any row) | 21 KiB | — | 404 | 184 |
| 2→6 inlined | 7 409 KiB | 28 machine instrs | 520 338 | 2 175 |
| 2→6 outlined (by value) | 1 663 KiB | 6.5 | 145 617 | 16 265 |
| 2→6 MadGraph form (20 functions) | 950 542 B | 5.1, 26 B | 16 000 | 36 480 |

What the generated code shows:[^aot-timings][^mg-form-chunked]
- **Straight-line code runs once per event, so its size is fetch traffic.** The
  inlined 2→6 streams 7.4 MiB of instructions per event, 3.7× the 2 MiB L2.
  Against the interpreter, a rendering wins up to 73 KiB of code and loses from
  319 KiB up. The MadGraph-form 2→6 (0.93 MiB) fits L2 but not L1i or the µop
  cache, and still runs at two thirds of the interpreter.
- **Inlining keeps the arena round trip; it moves it to the stack.** Op-blocked
  order keeps tens of thousands of values live, so the register allocator spills
  nearly every one: 14 stack-referencing instructions per VM instruction in the
  inlined 2→6, against the interpreter's 13 `f64` words of arena traffic.
- **LLVM declines to inline some kernels into a huge caller** (`propagate_fout_bare`
  864 call sites, `ffv_vout_bare` 602, …), which the interpreter inlines into its
  `match` arms.
- **Chunking adds its own round trip.** Op-blocked order puts a value's readers
  one ASAP level later, typically in a later chunk: ~31–36 k cross-chunk stores
  per event.
- **LLVM rewrites the MadGraph form's by-reference convention.** Under LTO every
  entry point is internal, and argument promotion passes scalar operands in
  registers; arrays of four complex values stay pointers. Slot base addresses
  that do not fit callee-saved registers are cached on the stack (~2 stack refs
  per call). A Fortran compiler with separately compiled HELAS cannot do this.

## Reading

**What the interpreter's own overhead costs.** The rendered program performs the
same arithmetic without dispatch, operand decoding, arena indexing or bounds
checks, so `vm − aot_out` is a lower bound on everything else the interpreter
does: **at least 23–51% of scalar `forward` time** on rows whose rendered code
stays cache-resident (`ee_to_mumu` 31%, `gg_to_gg` 49%,
`ee_to_mumu_tata_qcd0` 27%).[^aot-reading] That is larger than the mispredict
(~2%) and bounds-check (3.5–5.5%, see [bounds checks](../performance/bounds-checks.md))
shares measured elsewhere.

**Most of that overhead is the arena round trip, not dispatch and decode.** The
three arms split it. The MadGraph form removes dispatch and operand decoding but
keeps every operand and result in memory, as the interpreter does; it gains only
1.05–1.27× on the small rows. The by-value form removes the same dispatch and
decode *and* passes values between kernels in registers and the caller's frame;
it gains 1.4–2.2× with code only 15–30% larger. So the decode-and-dispatch share
of the interpreter's overhead is the small part (the MadGraph-form gain) and the
store-and-reload of every value through its arena is the larger (the by-value
form's extra gain). This is inferred from the three arms' differences, not from
counters.[^mg-form-chunked]

**Against the FP-issue ceiling.** Scaled to the `aot_out` times, scalar
`gg_to_gg` and `ee_to_mumu_tata_qcd0` run at about 75–90% of their operation
mix's issue ceiling, against 37–68% interpreted
([roofline census](../performance/roofline-census.md)). The gap that census left
to "dependency latency" is, on these rows, mostly interpreter overhead. The
ceilings carry that census's ±10%.

**Why ahead-of-time compilation does not scale.** Straight-line code costs
instruction bytes linear in program length, executed once per event; an
interpreter's code is fixed and its program is data, which the data-side
prefetchers stream. Past a few hundred KiB the front end, not the FP ports, sets
the pace. The outlined binding moves the crossover from between 695 and 1 756 VM
instructions to between 1 756 and 36 523; it does not remove it. The front-end
reading is by elimination: the host has no PMU.

**What would scale** (untried): code that does not grow with the program. Either
a loop per kernel kind over a table of operand indices (the interpreter again,
minus what a typed, batched dispatch removes), or rendering with loops over the
repeated helicity structure. A hypothesis not tested: MadGraph's Fortran 2→6
runs near our interpreter's speed in this same slot-array form, and its edge may
be code volume per event, since it re-executes straight-line code across
helicities rather than unrolling per helicity as our helicity-expanded program
does.

## Compile cost

| build | wall | peak RSS |
|---|--:|--:|
| default `eval_strategies` (reference) | 90 s | 1.0 GiB |
| three small rows, by value, fat LTO | 80 s | 1.0 GiB |
| + 2→6 by value, one function, fat LTO | 3 135 s (52 min) | 3.7 GiB |
| + 2→6 by value, chunks of 2 000 / 500 | 860 s / 641 s | 3.9 / 3.0 GiB |
| + 2→6 MadGraph form, one function | library `rustc` OOM-killed at 13.9 GiB | — |
| + 2→6 MadGraph form, 19 chunks, thin LTO, cold, 2 jobs | 408 s | lib 1.94 GiB |

The 52-minute build spent nearly all its time in LLVM's pre-RA machine
scheduler on one basic block of a million instructions; the MadGraph-form OOM is
rustc's MIR `ReferencePropagation` pass, before LLVM. Both are recorded in
[rustc limits on generated code](../performance/rustc-limits-on-generated-code.md).
Any generator emitting Rust for the large processes must chunk. Chunking cut
build time but not run time. The fat-LTO cost of the chunked MadGraph-form 2→6
was not measured: two container restarts on the host coincided with fat-LTO
builds containing it.[^aot-reading][^mg-form-chunked]

## Caveats

- One host, a shared 4-vCPU VM with another session building at times. Single
  cells move 10–20% with code layout; round spreads reach 0.5–0.8 on some cells.
  Conclusions rest on factor-level differences reproduced in both runs; the
  1.3–1.6× small-row gains are read with their direction across rows and runs.
- No PMU: the front-end reading is by elimination, not counters.
- Production order only. An order that keeps producers next to consumers would
  spill less and chunk with a small carried set; not tried.
- The outlined binding keeps `build_external_core` and the operator
  expressions inline in both arms; only the `kernel::*_bare` calls differ.
- The 2→6 MadGraph-form rows use the thin-LTO profile; only chunks of 2 000 were
  tried; the by-value 2→6 figures are the first study's (one function, fat LTO).

## Reproduce

Check out `03c31e6` (features `aot-study`, `aot-study-large`) or `b504391`
(`aot-mg-study`, `aot-mg-study-large`). Rendering is the ignored `aot_render`
lib test (`VIBEGRAPH_AOT_ROWS`, `VIBEGRAPH_AOT_CHUNK`, `VIBEGRAPH_AOT_FORMS`), the
bit-for-bit oracle is the `aot` lib tests, and timings are
`RUSTFLAGS="-C target-cpu=native" cargo bench -p vibegraph-lib --features <f> --bench aot_kernels`
(`VIBEGRAPH_AOT_ROUNDS`, `VIBEGRAPH_AOT_CELL_MS`, `VIBEGRAPH_AOT_BENCH_ROWS`). The
exact command lines are in the archived results note. Do not build the
MadGraph-form 2→6 as one function on a host with less than ~16 GB
free.[^aot-reading][^mg-form-chunked]

[^aot-summary]: AOT study question and answer, `aot-kernels-study-results.md` L13–51.
[^aot-method]: Host, rendering method and correctness, L52–111.
[^aot-timings]: Timings, code size and codegen, L112–193.
[^aot-reading]: Reading, compile cost, caveats and reproduce, L194–289.
[^mg-form]: MadGraph-form method and the one-function OOM, L290–432.
[^mg-form-chunked]: MadGraph-form chunked build, timings, reading and caveats, L433–587.
