# The helicity program compiled ahead of time — results

**Status: measurement record, 2026-10-04. The study code is not in the tree: it
lives at commit `03c31e6` (`git checkout 03c31e6` to reproduce), and was removed
because its large builds take 11–52 min.** The question: how fast does the *same* computation run
when the bytecode program `fill_arenas` interprets is instead rendered to Rust as
straight-line kernel calls and compiled by rustc/LLVM?

**Answer.** It depends on the program's size, and the dividing line is instruction
footprint, not arithmetic.
- **Small programs gain.** With every kernel inlined (`aot`) the rendered program is
  1.3–1.6× faster than the interpreter on `ee_to_mumu` and `gg_to_gg` at scalar
  `f64`, and 1.3–1.9× at `LaneField<4>`.
- **Large programs lose, and badly.** Inlined, the rendered code is about 190 B of
  machine code per VM instruction, executed once per event. `ee_to_mumu_tata_qcd0`
  (319 KiB) is 0.8× the interpreter; the 2 → 6 (7.4 MiB, past the 2 MiB L2) is 0.2×,
  five times slower.
- **Calling the kernels out of line wins wherever the code is not too large.** The
  rendered order with each kernel behind an `#[inline(never)]` wrapper (`aot_out`) is
  45 B per VM instruction. It is the fastest arm at scalar width on every row but the
  2 → 6: 1.4–1.6× on `ee_to_mumu`, 2.0× on `gg_to_gg`, 1.3–1.4× on
  `ee_to_mumu_tata_qcd0`. On the 2 → 6 (1.6 MiB of code) it is 0.74×.
- **So what LLVM does across kernel boundaries is worth less than the instruction
  cache it costs.** Inlining every kernel into one function removes no stack traffic
  (§4): the values do not fit in registers and are spilled roughly as often as the
  interpreter stores to its arenas.
- **What the interpreter's own overhead costs, read as a lower bound** from the
  out-of-line arm: at least 23–51% of scalar `forward` time on the rows whose rendered
  code stays cache-resident. That is larger than the mispredict (~2%) and bounds-check
  (3.5–5.5%) shares measured before, so the per-instruction decode, operand loads and
  call glue that a `match` loop executes are most of it (§5, inferred).
- **The 2 → 6 compiles, but slowly.** One 36 523-statement function took 52 min and
  3.7 GiB peak RSS to build under fat LTO, nearly all of it in LLVM's machine
  scheduler on one basic block. Chunked into functions of 2 000 or 500 instructions it
  builds in 14 or 11 min (4.0 / 3.0 GiB), but runs no faster.

Every rendered row reproduces the interpreter's |M|² bit for bit at `f64` and at
`LaneField<4>`, under both kernel bindings, on the bench's 16 points (§2).

## 1. Host and method

Host: the Emerald Rapids Firecracker VM of `roofline-census-results.md` (Intel Xeon
family 6 model 207, 4 vCPU, 48 KiB L1d / 2 MiB L2 per core, measured 3.2 GHz, no PMU).
`rustc 1.97.0`, `RUSTFLAGS="-C target-cpu=native"`, `bench` profile (fat LTO). Tree:
`4f40a12` plus this study. Another session was building on the host at times.

**Rendering.** `helas::eval::aot::render` walks a compiled, helicity-pruned
evaluator's `folded_hel()` program in production (op-blocked) order. Each `Instr`
becomes one `let` binding a fresh local to the expression its `fill_arenas` arm
evaluates: the same `kernel::*_bare` call, or the same operator expression (`a * b`,
the left fold of an `Add*`, `PMomOut`'s `0 ± p ± …` then negation), with the same
operands in the same order. Arena slots are resolved symbolically: a read of
`(class, slot)` names the local the latest write to that slot produced, so the
rendered dataflow is the interpreter's by construction, slot recycling included.
External legs call `build_external_core` with the leg table's spin, charge, direction
and baked helicity as literals. Constant pools, external momenta and the per-point
momentum pool (resolved by the same `MomTable::resolve` loop as `resolve_moms`) are
arguments, converted to fixed-size array references once at entry, so pool indexing
carries no bounds check. The roots come back in `RootKind::Hels` order and
`AotAmplitude::eval_m2` applies `eval_m2`'s read-out: helicity sum times `CF(1,1)`,
or the per-combination CF contraction.

The rendered sources live in `helas/eval/aot/generated/` and are compiled through
`include!`, twice: once with `k::` bound to `kernel` (inlined) and once to
`#[inline(never)]` wrappers of the same kernels (outlined). The three small rows are
checked in (3–70 KB of source); the 2 → 6 (1.6–4.2 MB) is git-ignored, written by
the `aot_render` ignored test, and compiled only under `aot-study-large`.
Generation is a test, not a build script, because it needs the crate's own compiler
pipeline. `AotAmplitude::new` re-renders the live program and refuses a source that
differs, so a stale file cannot be timed. Rendering is deterministic: the four rows
rendered in separate processes are byte-identical.

**Timing protocol** (`benches/aot_kernels.rs`, own harness). Per row, the arms
`vm_f64` (`BoundAmplitude::eval_m2`), `aot_f64`, `aot_out_f64`, `vm_lanes4`
(`eval_m2_lanes_packed` on prepacked momenta) and `aot_lanes4` run round-robin, the
starting arm rotating each round. A cell is one pass of the 16 bench points, repeated
to about 200 ms. Rounds: 9 (run 2) and 11 (run 3). The reported figure is the minimum
ns/event over rounds; spread is `max/min − 1`. Before timing, every arm's 16 values
are asserted equal to the scalar interpreter's, bit for bit.

Bench points: the `eval_strategies` points — `StdRng::seed_from_u64(0xBE7C4)`,
√s = 500, beams along ±z, `rambo_massless`, 16 per row, drawn in bench-row order
(rows before the requested one are drawn and discarded).

## 2. Correctness

| check | result |
|---|---|
| `aot_matches_interpreter_bit_for_bit` (dev profile, opt 2 + debug assertions): 3 small rows × 16 points, `to_bits` against `BoundAmplitude::<f64>::eval_m2`, inlined and outlined at `f64`, inlined at `LaneField<4>` | pass |
| `aot_reads_the_bound_pools`: a 1e-7 move of one coupling moves the rendered |M|² | pass |
| bench pre-check (`bench` profile, native): all 4 rows, every arm, 16 points, `to_bits` | pass in all four binaries (small rows; monolithic, 2 000- and 500-chunk 2 → 6) |

Bit equality is what the construction predicts: the same kernels and operator
expressions in the same operand order, no fast-math, and Rust does not contract `a *
b + c` into an FMA. Moving instructions between functions or inlining them does not
change any rounding. What the oracle cannot see: an instruction whose result is never
read. `gg_to_gg` has four (complex products under the configuration bundle), which the
rendered code drops as dead and the interpreter computes; 4 of 695 instructions.

## 3. Timings

ns/event, minimum over rounds, with the round spread in parentheses. Ratio is
`vm / aot` (above 1: the rendered program is faster). Run 3 is the 11-round run;
run 2's minima (9 rounds, another process) agree within 3–10% except where marked.

| row | VM instrs | `vm_f64` | `aot_f64` | `aot_out_f64` | `vm_lanes4` | `aot_lanes4` |
|---|--:|--:|--:|--:|--:|--:|
| `ee_to_mumu` | 66 | 451 (0.42) | 325 (0.12) | 312 (0.21) | 136 (0.80) | 76 (0.23) |
| `gg_to_gg` | 695 | 2 413 (0.20) | 1 854 (0.12) | 1 232 (0.46) | 952 (0.27) ¹ | 634 (0.06) |
| `ee_to_mumu_tata_qcd0` | 1 756 | 8 139 (0.47) | 10 127 (0.20) | 5 936 (0.54) | 2 691 (0.55) | 2 900 (0.15) |
| `uux_to_ccx_emmm_qcd0`, one function | 36 523 | 166 189 (0.41) | 774 922 (0.23) | 225 867 (0.15) | 64 030 (0.20) | 213 372 (0.24) |
| — chunks of 2 000 | | 163 395 (0.24) | 801 534 (0.16) | 317 018 (0.30) | 64 457 (0.17) | 287 449 (0.09) |
| — chunks of 500 | | 165 421 (0.56) | 863 764 (0.14) | 333 658 (0.40) | 63 621 (0.51) | 303 051 (0.26) |

¹ Run 2 read 755; the run-3 cell is the outlier of the two.

| row | `aot_f64` ratio, run 2 / 3 | `aot_out_f64` ratio, run 2 / 3 | `aot_lanes4` ratio, run 2 / 3 |
|---|--:|--:|--:|
| `ee_to_mumu` | 1.55 / 1.39 | 1.63 / 1.45 | 1.87 / 1.80 |
| `gg_to_gg` | 1.34 / 1.30 | 2.04 / 1.96 | 1.26 / 1.50 |
| `ee_to_mumu_tata_qcd0` | 0.79 / 0.80 | 1.30 / 1.37 | 0.97 / 0.93 |
| 2 → 6, one function | 0.20 / 0.21 | 0.74 / 0.74 | 0.29 / 0.30 |
| 2 → 6, chunks of 2 000 | 0.20 / 0.20 | 0.53 / 0.52 | 0.22 / 0.22 |
| 2 → 6, chunks of 500 | — / 0.19 | — / 0.50 | — / 0.21 |

A first 7-round run of the small-row binary read 1.42 / 1.32 / 0.82 (`aot_f64`) and
1.92 / 1.28 / 0.93 (`aot_lanes4`) on the three small rows. The VM cells here match the
`eval_strategies` medians of `roofline-census-results.md` (385 / 2 482 / 7 562 /
173 844) to within the host's 10–20% layout noise.

## 4. Code size and codegen

`f64` instances in the monolithic bench binary (`objdump -d`, counted over each
function's symbol range). "Stack refs" counts instructions with an `(%rsp)` operand:
spills, reloads and by-memory kernel arguments and results.

| function | bytes | machine instrs | per VM instr | FP instrs | stack refs | calls |
|---|--:|--:|--:|--:|--:|--:|
| `fill_arenas::<f64>` (the interpreter, any row) | 21 KiB | 4 360 | — | 974 | 404 | 184 ² |
| `aot_ee_to_mumu`, inlined | 7 KiB | 1 134 | 17 | 410 | 524 | 19 |
| `aot_ee_to_mumu`, outlined | 2.6 KiB | 463 | 7.0 | 36 | 208 | 33 |
| `aot_gg_to_gg`, inlined | 50 KiB | 7 811 | 11 | 4 084 | 2 961 | 24 |
| `aot_gg_to_gg`, outlined | 29 KiB | 4 802 | 6.9 | 2 273 | 1 600 | 159 |
| `aot_ee_to_mumu_tata_qcd0`, inlined | 319 KiB | 46 715 | 27 | 27 355 | 20 436 | 131 |
| `aot_ee_to_mumu_tata_qcd0`, outlined | 73 KiB | 11 004 | 6.3 | 3 917 | 6 292 | 769 |
| `aot_uux_to_ccx_emmm_qcd0`, inlined | 7 409 KiB | 1 023 916 | 28 | 600 211 | 520 338 | 2 175 |
| `aot_uux_to_ccx_emmm_qcd0`, outlined | 1 663 KiB | 237 234 | 6.5 | 99 020 | 145 617 | 16 265 |

² 157 of them `panic_bounds_check`, on cold paths.

At `LaneField<4>` the inlined functions are 12 / 84 / 393 / 8 094 KiB. Chunked, the
2 → 6's inlined `f64` code is 7 828 KiB over 20 functions and the outlined 2 528 KiB.

- **Straight-line code is executed once per event, so its size is fetch traffic.**
  The inlined 2 → 6 streams 7.4 MiB of instructions per event, 3.7× the L2, at about
  3 B/cycle. The interpreter's whole hot path is 21 KiB plus its kernels. Between the
  two renderings, the smaller one wins on every row but `ee_to_mumu`, where both fit
  in L1i and tie. Against the interpreter, a rendering wins up to 73 KiB of code and
  loses from 319 KiB up. No counter confirms front-end stalls on this host (no PMU);
  this reading rests on that ordering and on the outlined arm doing the same
  arithmetic in a quarter of the bytes.
- **Inlining keeps the arena round trip; it moves it to the stack.** The inlined
  2 → 6 has 520 k stack-referencing instructions per event (218 k `mov` loads, 145 k
  `mov` stores, the rest FP ops with a memory operand), 14 per VM instruction. The
  interpreter moves 3.9 MB of arena values per event, 13 `f64` words per VM
  instruction (`roofline-census-results.md`). Op-blocked order keeps tens of
  thousands of values live at once, so the register allocator spills nearly every
  one; production order gives LLVM's scheduler no locality to exploit.
- **LLVM declines to inline some kernels into a large caller.** The inlined 2 → 6
  still calls `propagate_fout_bare` (864 sites), `ffv_vout_bare` (602),
  `propagate_fin_bare` (480), `propagate_vector_bare` (208). The interpreter inlines
  them into its `match` arms.
- **Chunking adds its own round trip.** Op-blocked order puts an instruction's
  readers one ASAP level later, typically in a later chunk, so almost every value
  crosses a boundary: 31 085 stores and 33 577 loads of cross-chunk values per event
  at 2 000 instructions, 35 768 and 41 782 at 500. Slots are recycled once their last
  reading chunk has run, which caps the carried set at 12 k values (~310 KB), close
  to the interpreter's 451 KiB of arenas. The carried arrays are allocated and zeroed
  per event, ~310 KB of stores: an estimated 1–3% of a chunked event's time, not
  measured.

## 5. Reading

- **What the interpreter costs.** The rendered program performs the same arithmetic
  without dispatch, operand decoding, arena indexing or bounds checks. Its time is
  therefore an upper bound on the time of that arithmetic, and `vm − aot_out` a lower
  bound on what the interpreter spends on everything else: at least 23–51% of scalar
  `forward` (`ee_to_mumu` 31%, `gg_to_gg` 49%, `ee_to_mumu_tata_qcd0` 27%, run 3).
  The previous attributions summed to under a tenth (mispredicts ~2%, bounds checks
  3.5–5.5%), so most of the interpreter's overhead is the instructions a dispatch
  step executes even when predicted: the indirect jump, the instruction-record and
  operand-index loads and the call glue. This is inferred from the difference of two
  timings, not split by counters.
- **Against the FP-issue ceiling.** `roofline-census-results.md` §4a puts scalar
  `gg_to_gg` at 37–42% of the ceiling of its operation mix and
  `ee_to_mumu_tata_qcd0` at 57–68%. Scaled to the `aot_out` times, the same arithmetic
  runs at about 75–90% of the ceiling on both. So the gap that note left to
  "dependency latency" is, on these rows, mostly interpreter overhead: the same
  chains run within 10–25% of the issue bound once the dispatch is gone. (The
  ceilings carry that note's ±10%, and its counts include a dozen debug-build
  operations per event.)
- **Why ahead-of-time compilation does not scale.** A straight-line rendering costs
  instruction bytes linear in program length, executed once per event; an
  interpreter's code is fixed and its program is data. Past a few hundred KiB the
  front end, not the FP ports, sets the pace. The out-of-line binding moves the
  crossover, from between 695 and 1 756 VM instructions to between 1 756 and 36 523,
  but does not remove it.
- **What would.** Code that does not grow with the program: a loop per kernel kind
  over a table of operand indices, which is the interpreter again, minus whatever of
  its overhead a typed, batched dispatch removes; or rendering with loops over the
  repeated helicity structure. Both are outside this study. For the lanes the
  picture is the same, shifted: the per-event code is amortised over four events, and
  `aot_lanes4` still loses on the two large rows.

## 6. Compile cost

`cargo bench --no-run` of the `aot_kernels` bench, `bench` profile (fat LTO), native,
incremental after touching `lib.rs` unless noted; wall time and peak RSS of any
descendant process. Each build compiles every rendered row at three instances
(inlined `f64`, inlined `LaneField<4>`, outlined `f64`). The generic rendered
functions are only checked in the library crate (about a minute); they are
monomorphised and code-generated in the bench crate, in one fat-LTO codegen unit.

| build | wall | peak RSS |
|---|--:|--:|
| default features, `eval_strategies` (reference) | 90 s | 1.0 GiB |
| `aot-study`, three small rows | 80 s | 1.0 GiB |
| `aot-study-large`, 2 → 6 as one function | 3 135 s (52 min) | 3.7 GiB |
| `aot-study-large`, 2 → 6 in chunks of 2 000 (20 functions) | 860 s (14 min) | 3.9 GiB |
| `aot-study-large`, 2 → 6 in chunks of 500 (75 functions) | 641 s (11 min) | 3.0 GiB |

The three small rows cost nothing measurable. The monolithic 2 → 6 build overlapped
the first, cold chunked build (951 s, 4.0 GiB) for about 16 of its 52 minutes. Three
`gdb` backtraces of it, 38 minutes in, were all in `MachineScheduler` → `ScheduleDAGMI::moveInstruction` →
`SlotIndexes::insertMachineInstrInMaps`: the pre-RA scheduler moving instructions
within one basic block of a million, which renumbers slot indices per move. Chunking
cuts the block size, and the time follows, but the chunked code runs no faster
(§3).

## 7. Caveats

- One host, a shared 4-vCPU VM with another session building at times. Single
  cells move 10–20% with code layout here, and round spreads reach 0.5–0.8 on
  some cells. Every conclusion above rests on factor-level differences, which both
  runs reproduce; the 1.3–1.6× small-row gains are smaller than that and are read
  together with their direction across rows and runs.
- No PMU: the front-end reading of §4 is by elimination, not counters.
- Production order only. A rendering in an order that keeps producers next to their
  consumers would spill less and chunk with a small carried set; it was not tried.
- The `outlined` binding keeps `build_external_core` and the operator expressions
  inline in both arms; only the `kernel::*_bare` calls differ.

## Reproduce

At commit `03c31e6`:

```
# render the three committed rows (default) or the 2 → 6, optionally chunked
cargo test -p vibegraph-lib --features aot-study --lib aot_render -- --ignored --nocapture
VIBEGRAPH_AOT_ROWS=uux_to_ccx_emmm_qcd0 VIBEGRAPH_AOT_CHUNK=2000 \
    cargo test -p vibegraph-lib --features aot-study --lib aot_render -- --ignored --nocapture

# the bit-for-bit oracle
cargo test -p vibegraph-lib --features aot-study --lib aot

# timings (add aot-study-large after rendering the 2 → 6; expect 11–52 min to build)
RUSTFLAGS="-C target-cpu=native" cargo bench -p vibegraph-lib --features aot-study \
    --bench aot_kernels
#   VIBEGRAPH_AOT_ROUNDS (7), VIBEGRAPH_AOT_CELL_MS (200), VIBEGRAPH_AOT_BENCH_ROWS
```

Code-size counts: `nm -C -S` for each function's range, then `objdump -d
--start-address --stop-address`, counting instructions, `call`s, `(%rsp)` operands
and `v{add,sub,mul,fmadd,…}` mnemonics.

---

# MadGraph's form: values in slot arrays, one in-place call per instruction

**Status: measurement record, 2026-10-04. Answer: no — MadGraph's form is the
smallest rendering and, split into functions, builds in four minutes, but the 2 → 6
runs at 0.66× the interpreter (0.67–0.79× at four lanes), and in one function it does
not compile on a 16 GB host (rustc's MIR optimisation passes 13.9 GiB).** The code is
not in the tree: it lives at commit `b504391` (`aot-mg-study` feature). The question: does the form MadGraph's Fortran matrix elements take — every wavefunction in
one memory-resident array, every HELAS call an out-of-line routine taking operands by
reference and writing its result in place, the matrix routine nothing but a sequence of
such calls — keep the rendered program's code small and its build cheap on the 2 → 6,
where the by-value rendering above was 0.74× the interpreter and took 52 min to build?

## M1. Method

Host and timing protocol as §1 (Emerald Rapids VM, `rustc 1.97.0`,
`-C target-cpu=native`, round-robin arms with rotating start, min over rounds, spread
`max/min − 1`), same 16 bench points.

**The rendered form** (`helas::eval::aot::mg`). Each row's program becomes one generic
function over an `MgArenas<F, S, V, M, I, O>`: one boxed fixed-size array per result
class (complex scalars, vectors, multivectors, kets, bras), sized by the interpreter's
`Program::arena_sizes` and indexed by its `Program::dest`, so the slot assignment and
the arena footprint are the interpreter's. Every instruction is one call to an
`#[inline(never)]` entry point (`aot::kernels::out_param`) of the form
`k::ffv_fin(&mut a.i[7], &a.v[3], &a.i[2], &cc[4], &a.s[9])`: the result written through
`&mut` into its slot, every operand a reference into a slot array or a pool, every index
a literal into an array of literal length. Flags (`reversed`, `Chirality`, helicity) are
passed by value. The entry points are thin wrappers that call the existing `*_bare`
kernels (which inline into them) and store the result: `*out = kernel::ffv_fin_bare(eps,
fi, *gl, *gr)`. `Add*` is one generic `add(out, &[&t0, &t1, …])` folding left from the
first term, `Mul*`/`Scale*` one generic `mul(out, a, b)` computing `*a * *b`, external
legs `ext_*(out, &mo[leg], hel, spin, charge, incoming, &cr[mass])` over
`build_external_core`, `PMomOut` a loop over a literal `(momentum id, sign)` slice. No
value is held in a local in the rendered function.

Two details make this safe Rust with no bounds checks. The interpreter's slot allocator
never gives an instruction a destination equal to one of its operands, so an operand of
the destination's own class is borrowed from the halves of that array on either side of
the destination (`let (l, d, h) = sp(&mut a.i, 7);`, an `#[inline(always)]`
`split_at_mut` with literal arguments, which folds away). Constant-pool loads
(`ComplexConst`, `RealConst`) emit no call: a later read of the slot they filled names
the pool entry (`&cc[4]`), as MadGraph passes a coupling straight from its coupling
array. 66 → 52 calls on `ee_to_mumu`, 36 523 → 36 476 on the 2 → 6.

**Arms.** `vm_f64` / `vm_lanes4` (the interpreter, `eval_m2` / `eval_m2_lanes_packed`),
`mg_f64` / `mg_lanes4` (this form), `byv_f64` (the by-value out-of-line rendering of the
first study, `aot_out` above, re-rendered by the same code: its three small sources are
byte-identical to `03c31e6`'s). The inlined arm of the first study is not kept.

## M2. Correctness

| check | result |
|---|---|
| `aot_matches_interpreter_bit_for_bit` (dev profile): 3 small rows × 16 points, `to_bits` against `BoundAmplitude::<f64>::eval_m2`; `mg` and `byv` at `f64`, `mg` at `LaneField<4>` | pass |
| `aot_reads_the_bound_pools`: a 1e-7 move of one coupling moves both forms' |M|² | pass |
| `mg_form_is_calls_only`: every body line of every compiled-in MG source is one `k::` call, optionally after its `sp` split | pass |
| bench pre-check (`bench` profile, native): small rows, every arm, 16 points, `to_bits` | pass |

## M3. Small rows, fat LTO (`bench` profile)

ns/event, min over 11 rounds (run 1), spread in parentheses. Ratio `vm / arm`.

| row | `vm_f64` | `mg_f64` | `byv_f64` | `vm_lanes4` | `mg_lanes4` |
|---|--:|--:|--:|--:|--:|
| `ee_to_mumu` | 330 (0.44) | 262 (0.47) | 246 (0.13) | 111 (0.31) | 82 (0.47) |
| `gg_to_gg` | 1 777 (0.41) | 1 378 (0.34) | 842 (0.57) | 591 (0.45) | 514 (0.50) |
| `ee_to_mumu_tata_qcd0` | 5 732 (0.46) | 5 196 (0.33) | 4 115 (0.29) | 1 941 (0.50) | 1 585 (0.43) |

| row | `mg_f64` | `byv_f64` | `mg_lanes4` |
|---|--:|--:|--:|
| `ee_to_mumu` | 1.26 | 1.35 | 1.35 |
| `gg_to_gg` | 1.29 | 2.11 | 1.15 |
| `ee_to_mumu_tata_qcd0` | 1.10 | 1.39 | 1.22 |

The `vm` and `byv` ratios reproduce the first study's (1.45 / 1.96 / 1.37 for
`aot_out`). The MG form beats the interpreter on every small row, but by less than the
by-value form: its operands come from and its results go to memory, as the
interpreter's do, so it removes the dispatch and decode but not the arena traffic.

**Code size** (`objdump` over each function's symbol range, fat-LTO bench binary).

| function | bytes | machine instrs | per VM instr | calls | stack refs | bounds checks |
|---|--:|--:|--:|--:|--:|--:|
| `mg_ee_to_mumu::<f64>` | 2 084 | 413 | 32 B | 56 | 109 | 0 |
| `mg_gg_to_gg::<f64>` | 25 891 | 4 077 | 37 B | 677 | 1 098 | 0 |
| `mg_ee_to_mumu_tata_qcd0::<f64>` | 66 079 | 10 496 | 38 B | 1 720 | 3 784 | 0 |
| `aot_ee_to_mumu::<f64>` (by value) | 2 673 | 463 | 41 B | 33 | 208 | 0 |
| `aot_gg_to_gg::<f64>` (by value) | 29 713 | 4 802 | 43 B | 159 | 1 600 | 0 |
| `aot_ee_to_mumu_tata_qcd0::<f64>` (by value) | 74 862 | 11 004 | 43 B | 769 | 6 292 | 0 |

At `LaneField<4>` the MG functions are within 1–8% of their `f64` size (only addresses
change). The MG form is 32–38 B per VM instruction against the by-value form's 41–43 B:
smaller, but not by the factor the "argument setup only" picture suggests. Two things
the generated code does that the source does not say:
- **LLVM rewrites the by-reference calling convention.** Under LTO every entry point is
  internal, and argument promotion turns small by-reference operands into by-value
  registers: `mul::<C<f64>, C<f64>>` is called with its two complex operands loaded
  into `xmm0–3` by the caller. Arrays of four complex values (vectors, spinors) stay
  pointers. So the call sites do load scalar operands, and the `&` in the source is not
  the ABI.
- **Slot addresses are cached on the stack.** The 4–9 base pointers (five arenas,
  four pools) do not fit the callee-saved registers, and LLVM also keeps some
  `base + offset` addresses it has already formed in stack slots rather than
  re-forming them, so roughly two stack references per call remain.

The entry points themselves are 18–640 B each (`f64`); `fill_arenas::<f64>` is
18–22 KiB with 157 bounds-check call sites.

**Build cost, small rows** (`cargo bench --no-run`, fat LTO, incremental after touching
`lib.rs`, per-`rustc` wall and peak RSS from a `RUSTC_WRAPPER`): 78 s wall; the lib crate
22.5 s / 1.24 GiB, the bench crate (all monomorphisation and codegen) 54.5 s / 0.95 GiB.
That wrapper ran `rustc` detached from cargo's jobserver; later builds attach it.

## M4. The 2 → 6 in one function does not compile on this host

The monolithic MG rendering of the 2 → 6 (36 476 calls, 2.6 MB of source) never reaches
code generation: the **library** crate's `rustc` is OOM-killed at 13.9 GiB RSS (the
host has 15 GiB), twice, in about two minutes. `-Z time-passes` (via
`RUSTC_BOOTSTRAP=1`) and `gdb` backtraces taken as RSS crossed 3.5, 5, 7, 9 and
10.5 GiB locate it:

| phase | time | RSS after |
|---|--:|--:|
| `type_check_crate` | 22–30 s | 1.7 GiB |
| `MIR_borrow_checking` (`rustc_borrowck::get_flow_results`) | 30–48 s | 2.8–3.4 GiB |
| `optimized_mir` → `ReferencePropagation` → `MaybeStorageDead::iterate_to_fixpoint` | — | > 10.5 GiB, killed |

`optimized_mir` runs for a generic function whenever its crate's metadata is encoded, so
this is the library build, before any LLVM work, and fat versus thin LTO does not enter.
Why this pass, inferred rather than measured: `ReferencePropagation`'s storage-liveness
dataflow keeps a bitset of locals per basic block; every call ends a basic block and
every `&a.v[3]` operand is a MIR temporary, so both counts grow with the program and the
memory with their product.
`cargo check` of the same crate (no optimised MIR) completes in 53 s at 3.4 GiB peak,
nearly all of it type checking and borrow checking. Why the by-value rendering of the first
study got through the same pipeline is not established; it makes 16 k kernel calls
against 36 k here, and its operands are named locals rather than fresh references.

This is a property of rustc's MIR pipeline on huge functions, not of LLVM: the first
study's 52-minute build was LLVM's machine scheduler; this one never gets that far.
Splitting the program into functions is therefore the fix to try, and in this form it is
free at run time: no value crosses a function boundary except through the slot arrays.

## M5. The 2 → 6 in functions of 2 000 instructions (thin LTO)

Rendered with `VIBEGRAPH_AOT_CHUNK=2000`: `mg_uux_to_ccx_emmm_qcd0` calls 19 functions of
up to 2 000 instructions in order, each taking the slot arrays and pools; nothing else
crosses. Built under the `aot-study` profile (release with `lto = "thin"`,
`codegen-units = 16`), `CARGO_BUILD_JOBS=2`, because fat-LTO builds containing the 2 → 6
coincided with two restarts of this host's container. Every arm of that binary,
interpreter included, uses that profile; the small rows were re-timed in it too.

**Build** (cold, deps included, 2 jobs): 408 s wall. The library crate 53.6 s at
1.94 GiB peak RSS (against > 13.9 GiB and an OOM kill in one function); the bench crate,
which monomorphises and generates code for every row at `f64` and `LaneField<4>` and
runs thin LTO, 190 s at 1.15 GiB. No `rustc` passed 2 GiB.

**Timings** (11 rounds, run 1, `aot-study` profile). All four rows bit-identical to the
interpreter at `f64` and `LaneField<4>` on the 16 points (bench pre-check).

| row | `vm_f64` | `mg_f64` | `byv_f64` | `vm_lanes4` | `mg_lanes4` |
|---|--:|--:|--:|--:|--:|
| `ee_to_mumu` | 334 (0.41) | 283 (0.55) | 197 (0.43) | 104 (1.18) | 81 (0.43) |
| `gg_to_gg` | 1 894 (0.67) | 1 812 (0.64) | 855 (0.47) | 656 (0.48) | 488 (0.46) |
| `ee_to_mumu_tata_qcd0` | 6 930 (0.37) | 6 208 (0.28) | 4 154 (0.35) | 2 550 (0.30) | 1 914 (0.37) |
| `uux_to_ccx_emmm_qcd0` | 124 838 (0.45) | 188 194 (0.69) | — | 47 291 (0.76) | 59 944 (0.41) |

| row | `mg_f64` ratio | `byv_f64` ratio | `mg_lanes4` ratio |
|---|--:|--:|--:|
| `ee_to_mumu` | 1.18 | 1.69 | 1.29 |
| `gg_to_gg` | 1.05 | 2.22 | 1.34 |
| `ee_to_mumu_tata_qcd0` | 1.12 | 1.67 | 1.33 |
| `uux_to_ccx_emmm_qcd0` | **0.66** | (0.74, first study, fat LTO, one function) | **0.79** |

Run 2 (9 rounds, same binary) reads `vm_f64` 326 / 1 842 / 6 049 / 124 354 and ratios
`mg_f64` 1.15 / 1.27 / 1.19 / **0.66**, `byv_f64` 1.57 / 2.15 / 1.53, `mg_lanes4`
1.38 / 1.25 / 1.17 / **0.67**; round spreads 0.23–0.86. The 2 → 6 ratio reproduces to
the percent; the small-row MG gains (1.05–1.27 across the two runs) are inside the
host's layout noise taken one cell at a time, but point the same way on every row.

The by-value 2 → 6 is not rebuilt here: its one-function form is the 52-minute build of
§6, and its ratio is quoted from §3.

**Code size** (`aot-study` binary, summed over the 20 functions of the 2 → 6).

| function | bytes | machine instrs | per VM instr | calls | stack refs | bounds checks |
|---|--:|--:|--:|--:|--:|--:|
| `mg_ee_to_mumu::<f64>` | 1 801 | 397 | 27 B | 56 | 70 | 0 |
| `mg_gg_to_gg::<f64>` | 20 981 | 4 027 | 30 B | 677 | 1 331 | 0 |
| `mg_ee_to_mumu_tata_qcd0::<f64>` | 54 671 | 10 093 | 31 B | 1 720 | 2 969 | 0 |
| `mg_uux_to_ccx_emmm_qcd0*::<f64>` (20 functions) | 950 542 | 187 058 | 26 B | 36 480 | 16 000 | 0 |
| — the same at `LaneField<4>` | 951 115 | 187 058 | 26 B | 36 480 | 16 000 | 0 |
| `fill_arenas::<f64>` (interpreter) | 17–21 KiB | 3 513–4 200 | — | 193 | 412–491 | — ³ |

³ Not counted in this binary: no instruction in it calls `panic_bounds_check` by
that name (the interpreter's checks branch to shared blocks whose callee `objdump`
does not resolve), so the grep that counts 157 in the fat-LTO binary reads zero here.
The rendered functions' zero is from the fat-LTO binary, where the grep works.

The MG 2 → 6 is 0.93 MiB of code, 5.1 machine instructions and 26 B per VM instruction:
an address computation per operand and the call. That is 57% of the by-value form's
1.6 MiB and an eighth of the inlined form's 7.4 MiB, and it fits the 2 MiB L2. It still
runs at two thirds of the interpreter.

## M6. Compile cost, side by side

`cargo bench --no-run` of `aot_kernels`, per-`rustc` wall and peak RSS from a
`RUSTC_WRAPPER`. The library crate type-checks, borrow-checks and MIR-optimises the
generic rendered functions; the bench crate monomorphises them (each row at `f64` and
`LaneField<4>`, the by-value small rows at both too) and generates all code.

| build | profile | lib crate | bench crate | wall |
|---|---|--:|--:|--:|
| small rows | `bench` (fat LTO), incremental | 22.5 s / 1.24 GiB | 54.5 s / 0.95 GiB | 78 s |
| small rows | `aot-study` (thin LTO, 16 CGUs), incremental, 2 jobs | 25.5 s / 1.08 GiB | 50.5 s / 0.36 GiB | 77 s |
| + 2 → 6, one function | `bench` (fat LTO) | OOM-killed at 13.9 GiB after ~2 min (§M4) | — | — |
| + 2 → 6, 19 functions of 2 000 | `aot-study`, cold (deps included), 2 jobs | 53.6 s / 1.94 GiB | 190.0 s / 1.15 GiB | 408 s |
| first study: + 2 → 6 by value, one function (§6) | `bench` (fat LTO) | ~1 min | — | 3 135 s / 3.7 GiB |

The chunked MG 2 → 6 adds about 28 s and 0.9 GiB to the library crate and 140 s to the
bench crate over the small rows alone. The fat-LTO cost of the chunked 2 → 6 was not
measured: both container restarts on this host happened during fat-LTO builds that
contained it (one in this form's chunked bench crate, after its library crate had built
in 51 s at 2.0 GiB), and those builds were then dropped for the thin profile.

## M7. Reading

- **MadGraph's form fixes the code size and, chunked, the build — not the run time.**
  It is the most compact rendering measured: 26–31 B and about five machine
  instructions per VM instruction (an address per operand and the call), 0.93 MiB for
  the 2 → 6 against 1.6 MiB by value and 7.4 MiB inlined. Split into 19 functions it
  builds in about four minutes at 2 GiB. Yet it is 0.66× the interpreter on the 2 → 6
  in both runs (0.67–0.79× at four lanes), below the by-value form's 0.74×.
- **The arena round trip is the interpreter's real cost, and this form keeps it.** On
  the small rows the MG form gains 1.05–1.27× over the interpreter where the by-value
  form gains 1.4–2.2×, with code 15–30% smaller. Both remove dispatch and operand
  decoding; only the by-value form also removes the memory round trip of every
  operand and result, because its values pass between out-of-line kernels in
  registers and in the caller's stack frame. So of the interpreter's overhead bounded
  in §5, the decode and dispatch share is the small part (the MG gain), and the store
  and reload of every value through its arena is the larger (the by-value form's
  extra gain). Inferred from the three arms' differences, not from counters.
- **Why the 2 → 6 still loses, by elimination.** Its 0.93 MiB of straight-line code
  fits the 2 MiB L2 but not the L1i or the µop cache, and each instruction of it is
  fetched once per event. The interpreter executes 21 KiB of code from L1i and streams
  its program as data, which the data-side prefetchers handle. Same arithmetic, same
  arena traffic, same slot layout: what differs is instruction fetch versus data
  fetch, so a front-end-bound reading is the one that fits; no PMU here to confirm it.
- **What MadGraph's own speed suggests.** Its Fortran 2 → 6 runs at about the
  interpreter's speed in this same form: routines reading `W(:, k)` from memory and
  writing back. A hypothesis not tested here is that its advantage over this rendering
  is code volume per event — how much of its straight-line code is re-executed across
  helicities rather than unrolled per helicity as our helicity-expanded program is.
  Rendering with loops over the repeated helicity structure, which §5 named, is the
  lever this study leaves untried.
- **A compiler finding worth keeping.** One generic Rust function of 36 k calls with
  reference operands cannot get through rustc's MIR optimisation on a 16 GB host,
  while the same calls split 19 ways cost 2 GiB. Any code generator emitting Rust for
  the large processes must chunk, whatever LLVM would then do.

## M8. Caveats

- Same host caveats as §7: shared 4-vCPU VM, 10–20% layout noise per cell, no PMU.
  The conclusions rest on the 2 → 6's 0.66× (reproduced to the percent in two runs)
  and on the ordering of the three arms on the small rows, which both runs share.
- The 2 → 6 rows use the thin-LTO `aot-study` profile, not the fat-LTO `bench`
  profile of §3. Within that binary all arms share the profile. The small rows were
  timed under both; the interpreter's small-row times agree within the noise.
- The MG form's "by reference" is the source-level form; under LTO, LLVM promotes
  scalar and real operands to registers (§M3), as a Fortran compiler with
  interprocedural optimisation could not for MadGraph's separately compiled HELAS.
- Only chunks of 2 000 were tried for the MG 2 → 6. The by-value 2 → 6 was not
  rebuilt; its figures are the first study's (§3, one function, fat LTO).

## M9. Reproduce

At commit `b504391`:

```
# render (small rows committed; the 2 -> 6 is git-ignored), both forms
cargo test -p vibegraph-lib --features aot-mg-study --lib aot_render -- --ignored --nocapture
VIBEGRAPH_AOT_ROWS=uux_to_ccx_emmm_qcd0 VIBEGRAPH_AOT_FORMS=mg VIBEGRAPH_AOT_CHUNK=2000 \
    cargo test -p vibegraph-lib --features aot-mg-study --lib aot_render -- --ignored --nocapture

# the bit-for-bit oracle (small rows) and the calls-only check
cargo test -p vibegraph-lib --features aot-mg-study --lib aot

# timings: small rows under fat LTO; all four rows under the thin study profile
RUSTFLAGS="-C target-cpu=native" cargo bench -p vibegraph-lib --features aot-mg-study \
    --bench aot_kernels
RUSTFLAGS="-C target-cpu=native" CARGO_BUILD_JOBS=2 cargo bench -p vibegraph-lib \
    --profile aot-study --features aot-mg-study-large --bench aot_kernels
#   VIBEGRAPH_AOT_ROUNDS (7), VIBEGRAPH_AOT_CELL_MS (200), VIBEGRAPH_AOT_BENCH_ROWS
```

Do not build the 2 → 6 rendered as one function (`VIBEGRAPH_AOT_CHUNK=0`) on a host
with less than ~16 GB free: the library crate alone passes 13.9 GiB.
