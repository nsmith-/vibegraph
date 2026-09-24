# Tail-call-threaded dispatch and the execution order — results

**Status: CLOSED (2026-09-24). Threaded dispatch through `become` works, is
bit-identical to the `match` loop, and ties it at the production order: +1.6%
`forward`, +1.5% lanes4, +5.2% lanes8 (geomean over the 8 bench rows, 3
rounds). It stays behind the nightly-only `threaded-dispatch` feature, and
nothing ships on it. The execution-order question it raised gets a sharper
answer than note 31 E1 could give. On this host, op-blocking's scalar win over
arena order (24–26%) comes from its ASAP-level structure, not its run lengths:
a level-mixed control with mean runs of 2–3 matches it under both dispatchers.
So the production order is not a dispatch workaround, and it stays.**

Host: Intel Xeon, family 6 model 85 stepping 7 (Cascade Lake), 2.8 GHz, 4-vCPU
Firecracker VM, 15 GiB, 32 KiB L1d / 1 MiB L2 per core / 33 MiB L3. AVX-512
F/DQ/CD/BW/VL + VNNI. This is not the Emerald Rapids VM of
`x86-avx2-perf-study-results.md`'s AVX-512 section.
Toolchain `nightly-2026-09-24` (`rustc 1.100.0-nightly 6eeff9a52`) for every arm,
`RUSTFLAGS="-C target-cpu=native"`, bench profile (`release`, fat LTO). Tree:
PR #8's head `6bd7325` plus this branch.

Figure of merit: `eval_strategies`, `forward` (scalar `f64`) and `lanes4` /
`lanes8` (`LaneField<N>`, per-chunk transpose included) over the 8 `BENCH_ROWS`.
Every cell is criterion's median; cells run round-robin, A B A B …; a table
entry is the geometric mean over the 8 rows of (median over rounds) / reference
cell, with the per-row range beside it. Run-to-run drift on this host is 1–4%
per cell, so a geomean inside ±2% is a tie.

## 1. The dispatcher

`fill_arenas`'s per-instruction work is one `#[inline(always)] fn step(&Instr,
loc, &mut Arenas, &EvalEnv)`. The `match` loop is `for … { step(…) }`. Under
`threaded-dispatch`, `threaded::run` starts a chain of handlers instead:

```rust
fn handler<F: Real, const OP: u8>(vm: &mut Vm<'_, '_, F>, rest: &[Threaded]) {
    let [cur, tail @ ..] = rest else { unreachable!() };
    if cur.instr.kind() == OP {
        step(&cur.instr, cur.dest as usize, &mut vm.arenas, vm.env);
    } else {
        unreachable!()
    }
    become (Table::<F>::HANDLERS[cur.next as usize])(vm, tail)
}
```

There is one monomorphisation per opcode, in a 256-entry `const` table, so a
`u8` indexes it unchecked. Inside the `kind() == OP` branch the compiler knows
the variant, and `step`'s `match` folds to the one arm. So the instruction
bodies are textually shared, and the two arms differ in dispatch alone. Each
`Threaded` record carries the instruction, its destination slot and the *next*
instruction's opcode. The handler therefore finds its successor in the cache
line it already loaded. All 212 handler instances (53 kinds × the bench's four
fields) end in exactly one `jmp *table(,%rax,8)` and hold no switch.
`MulScalarC` at `f64`, whole hot path:

```
push %rax; test %rdx,%rdx; je …            ; rest non-empty
cmpb $0xe,(%rsi); jne …                     ; kind check (folds step's match)
3 × (mov operand; cmp len; jbe panic)       ; bounds checks
vmovupd; vmulpd{1to2}; vshufpd; vmulpd{1to2}; vaddsubpd; vmovupd
movzbl 0x18(%rsi),%eax; dec %rdx; add $0x1c,%rsi; lea table; pop; jmp *(%rcx,%rax,8)
```

**Correctness.** `threaded_dispatch_matches_match_loop_bit_for_bit` runs both
dispatchers in one build (a test-only thread-local routes a pass through the
loop). It compares `AMP2`, `JAMP2` and per-helicity |M|² `to_bits` over every
MG-validated process, at `f64` and `LaneField<4>`, covering the base and the
helicity-expanded arenas. A planted fault fails it: `MulScalarC`'s handler
skipping `step` fails at `e+ e- > mu+ mu-`. `cargo test --lib` passes in full
under the feature.

## 2. Three things measured on the way, each a trap

**(a) The first handler layout lost 13.5%.** v1 read `instrs[pc]`, `dest[pc]`
and `opcodes[pc + 1]` from three streams through a `&mut Vm` that held
`&mut Arenas`. That meant three bounds checks on `pc`, one extra pointer hop to
every arena, and 44 hot-path instructions for `MulScalarC`. Against the `match`
loop (two rounds): `forward` 1.135, lanes2 1.076, lanes4 1.067, lanes8 1.022
(median per-row ratio). Fusing the three streams into the `Threaded` record,
passing the remaining stream as the slice argument and holding the arenas by
value took it to parity (§3).

**(b) `step(*instr, …)` cost the `match` loop 7% on lanes.** Passing the
instruction by value makes LLVM load all 20 bytes of it in the dispatch block,
before the jump: six loads. The register pressure then spills the loop counter
to the stack. PR #8's head beat the refactored loop by lanes4 0.926×, lanes8
0.906× (two rounds; `forward` 0.990). `step(&Instr)` with `match *instr`
restores the parent's shape: parent/refactor 0.999 / 0.967 / 0.997 in the next
sweep.

**(c) A permuted `kind()` evicted the kernels from the handlers.** Keying the
handlers on the existing `Instr::kind` looked free, but kinds 38–52 were a
permutation of the declaration order, so `kind()` compiled to a table lookup.
Its cost inside each handler tipped the soft-`#[inline]` kernels over LLVM's
threshold: 116 out-of-line `*_bare` calls across the handler instances, against
0. The threaded arm fell to 1.150 / 1.118 / 1.322. `Instr` is now declared in
`kind` order, so `kind()` is the identity on the tag and compiles to the tag
load; 0 calls. The op-blocked sort key is unchanged, so the production order is
too. Lesson for any future dispatcher work: diff the out-of-line call census
between builds, not only the timings.

## 3. Dispatcher × execution order

Both arms built once with `eval-schedule-study`, swept over
`VIBEGRAPH_EVAL_SCHEDULE` (the order is fixed at program build, so one binary
serves every order). Reference cell: `match@opblocked`, the production
configuration.

**Sweep A**: five orders, 2 rounds. The threaded binary is the one before
§2(c)'s regression (strum-numbered opcodes, 14 out-of-line kernel calls). PR
#8's head at the production order is a third arm.

| cell | `forward` | lanes4 | lanes8 |
|---|--:|--:|--:|
| `match@opblocked` | 1.000 | 1.000 | 1.000 |
| `threaded@opblocked` | 1.005 [0.97..1.04] | 0.981 [0.92..1.02] | 1.035 [1.02..1.04] |
| PR #8 head `@opblocked` | 0.999 [0.98..1.04] | 0.967 [0.87..1.00] | 0.997 [0.98..1.02] |
| `match@opwin32` | 1.055 [1.00..1.15] | 1.014 [0.90..1.12] | 1.019 [0.99..1.07] |
| `threaded@opwin32` | 1.028 [0.98..1.13] | 1.018 [0.92..1.10] | 1.048 [1.02..1.07] |
| `match@minlive` | 1.205 [1.09..1.25] | 1.009 [0.90..1.12] | 1.015 [0.82..1.08] |
| `threaded@minlive` | 1.198 [1.08..1.27] | 1.013 [0.92..1.09] | 1.061 [0.83..1.15] |
| `match@arena` | 1.238 [1.13..1.33] | 0.993 [0.88..1.05] | 1.029 [0.84..1.11] |
| `threaded@arena` | 1.262 [1.19..1.30] | 0.986 [0.90..1.04] | 1.051 [0.86..1.12] |
| `match@dfs` | 1.258 [1.18..1.31] | 1.014 [0.93..1.10] | 1.043 [0.84..1.09] |
| `threaded@dfs` | 1.267 [1.15..1.34] | 1.018 [0.97..1.07] | 1.048 [0.85..1.11] |

**Sweep B**: the level-mixed control (below), 3 rounds, both arms at `9df9e5a`.

| cell | `forward` | lanes4 | lanes8 |
|---|--:|--:|--:|
| `match@opblocked` | 1.000 | 1.000 | 1.000 |
| `threaded@opblocked` | 1.016 [0.98..1.05] | 1.015 [1.00..1.04] | 1.052 [1.02..1.08] |
| `match@levelmix` | 1.007 [0.98..1.03] | 0.985 [0.96..1.02] | 1.003 [0.96..1.03] |
| `threaded@levelmix` | 0.997 [0.97..1.04] | 1.007 [0.98..1.04] | 1.027 [1.01..1.05] |
| `match@arena` | 1.238 [1.15..1.32] | 1.017 [0.96..1.05] | 1.017 [0.84..1.08] |
| `threaded@arena` | 1.264 [1.18..1.32] | 1.025 [1.00..1.04] | 1.059 [0.87..1.14] |

The lanes8 2→6 cell: arena 0.837 / 0.867, `levelmix` 1.025 / 1.019 (match /
threaded). `levelmix` has op-blocked's peak bytes, and it loses the arena-order
gain along with them.

### Reading

- **The dispatcher does not change which order wins.** Every order ranks the
  same under both arms, and within an order the two arms agree to 1–3% on
  `forward`. On lanes the threaded arm is 1–5% behind. A plausible cause, not
  isolated here: every handler reloads the arena pointers from `Vm`, where the
  loop keeps them in registers. Intel's indirect predictor has been
  history-based since Haswell, so a single dispatch site already predicts about
  as well as one per handler; Rohou, Swamy and Seznec (CGO 2015, "Branch
  prediction and the performance of interpreters — don't trust folklore")
  measured the same. Threading comes out ahead only on short-run orders,
  `opwin32` (runs of 12–14, `forward` 1.055 → 1.028) and `levelmix` (runs of
  2–3, 1.007 → 0.997), and both differences are inside the noise.
- **Op-blocking's win is the level structure, not the runs.** Note 31 E1
  attributed its −17.9% (M3 Max) to run length, from the `opwin` controls. The
  expectation that threaded dispatch would make op-blocking redundant rests on
  that attribution. `levelmix` is the sharper control. It is op-blocked's ASAP
  levels in the same sequence, with the variants dealt round-robin inside each
  level (mean run 2.1–2.9 against 6.8–397): same levels, same operand distances
  and, on the large programs, the same peak bytes. It matches op-blocked to
  within 1% on `forward` under both dispatchers (1.007 `match`, 0.997 threaded),
  while arena order (mean run 1.0) costs 24–26%. What op-blocking buys is
  dependency-level grouping: consecutive instructions are independent, so the
  out-of-order core overlaps them across the dispatch jump (inferred from the
  controls; this VM exposes no PMU to count it). Arena and `dfs`
  order put each producer next to its consumer, which serialises the latency
  chain. `minlive` sits between them. On the M3 Max, E1's `opwin` controls did
  show the win growing with run length at fixed level grouping, so run length
  matters on that core. How much of E1's −17.9% is level grouping was never
  measured; a `levelmix` run there would say.
- **The size of the win reproduces across hosts; its mechanism may not.**
  Op-blocked against arena order is −19.2% geomean on `forward` here (sweep B,
  `match`; per row −12.9% to −24.3%). E1 measured −17.9% (−8.9% to −22.6%) on
  the M3 Max, and E1b −17.34% in production. The fuller −11% to −30% per row of
  note 31 §6.8 is the whole sprint (E1b + E2 + E2b), not the order alone. What
  differs between the hosts is the attribution. E1's `opwin` controls grew with
  run length at fixed level grouping, while here `levelmix` (runs of 2–3) loses
  nothing. The two cores' indirect predictors and out-of-order windows (the M3's
  is several times larger) are candidate causes, unmeasured.
- **Lanes care much less about order**: 1–6% on lanes4/lanes8 against 20–27%
  on `forward`. A lane instruction is 2–8× the arithmetic for the same
  dispatch and the same dependency chain.
- **One cell runs the other way: lanes8 on the 2→6.** `uux_to_ccx_emmm_qcd0`
  at lanes8 is 15–18% *faster* in every non-op-blocked order, under both
  dispatchers (sweep A: arena 0.835 / 0.857, `minlive` 0.820 / 0.834, `dfs`
  0.835 / 0.854). That is the working set. Its live-arena peak is 302 KB
  op-blocked against 237 KB arena and 206 KB `minlive` at `f64`
  (`execution_order_metrics`), so 2.4 / 1.9 / 1.6 MB at `LaneField<8>`,
  against a 1 MiB L2. The mean producer→consumer distance grows as well:
  11 091 against 8 196 instructions. `SCHEDULE_BYTE_LIMIT`'s rationale was
  that the order trades against the dispatch, not the working set. That holds
  at `f64` and at lanes4, and not at lanes8 on a program this size. The limit
  is evaluated at `f64` bytes and is lane-blind; this branch rewrites its
  comment to say so.

## 4. What this leaves

- **Production order: unchanged.** Op-blocked is the best or tied order for
  both dispatchers on every width but one cell. The threaded dispatcher gives
  no reason to revisit it.
- **Threaded dispatch: not adopted, kept buildable.** It ties at best, needs
  nightly, and would put an incomplete language feature (`explicit_tail_calls`
  still warns as incomplete) into the evaluator. The feature, its bit-identity
  test and the advisory `threaded-dispatch` CI job keep it from rotting until
  `become` stabilises. Revisit on a core with a weaker indirect predictor, or
  once handlers can keep arena pointers in registers across the chain (today
  every handler reloads them from `Vm`).
- **A lane-aware order fallback** (open, measure-first): pick `minlive` or
  arena order when `arena bytes × lane width` exceeds L2, which is exactly the
  lanes8 2→6 cell. `Program` is built once per evaluator and shared by every
  `F`, so this needs a per-width program or a per-width order. Size the
  payoff at the width a production lane path would use: lanes4 is where lanes
  would ship on a v3 target, and there the effect is absent.

## Reproduce

```
scripts/bench_dispatch.sh 3 opblocked levelmix arena     # sweep B's design
scripts/bench_dispatch.sh 2 opblocked arena dfs minlive opwin32
cargo +nightly-2026-09-24 test -p vibegraph-lib --lib \
    --features threaded-dispatch,eval-schedule-study -- threaded schedule
# order metrics (mean run, live bytes, distances):
cargo +nightly-2026-09-24 test -p vibegraph-lib --lib \
    --features threaded-dispatch,eval-schedule-study -- \
    --ignored --nocapture execution_order_metrics
```
