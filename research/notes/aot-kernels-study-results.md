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
