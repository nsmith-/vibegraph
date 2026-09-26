# Tail-call-threaded dispatch and the execution order — results

**Status: CLOSED (2026-09-25). The dispatcher code is archived under the git tag
`study/threaded-dispatch` and is not in the tree; what stayed is the order study's
control orders, its tooling and this record (Reproduce). Threaded dispatch through `become` works, is
bit-identical to the `match` loop, and does not beat it. At the production
order it is +1.6% `forward`, +1.5% lanes4 and +5.2% lanes8 on Cascade Lake,
and +8.6% / +2.1% / +1.0% on the M3 Max (geomean over the 8 bench rows). It
is not adopted, and its code was removed from the tree. Under LLVM's `preserve_none` convention, with the arenas carried as
register arguments (AArch64 only), it recovers 4–6 points against the Rust ABI.
It then ties at lanes8 and trails by 1.4% at lanes4 and 6% at scalar (§6, M3
Max, layout-randomised). The execution order has two jobs, and op-blocking does
both. On every
program, ASAP-level grouping keeps independent instructions adjacent; arena
order loses 19% to it on both hosts. On a program too long for the branch
predictor to memorise (the 2→6, 36 523 instructions), the order must also be
predictable. A random order within the levels costs 2.2× there on the M3 Max,
under either dispatcher, and Instruments' counters show 40–47% of its cycles
discarded. Op-blocked runs and a periodic interleave cost nothing (§4). The production order is not a `match`-dispatch workaround, and it
stays.**

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
- **Op-blocking's win is the level structure, not the runs — on programs the
  predictor can memorise.** §4 qualifies this bullet: `levelmix`'s interleave
  is periodic, and a random within-level order costs 2.2× on the 2→6 on the
  M3 Max. Note 31 E1
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
  nothing. §4 re-measures the M3 Max and resolves this: `levelmix` loses nothing
  there either.
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

## 4. Apple M3 Max, and the shuffle control

Host: Apple M3 Max, 12 P-cores (128 KiB L1d, 16 MiB shared L2 per P-cluster) +
4 E-cores, macOS 15.7, `nightly-2026-09-24` (aarch64), `-C target-cpu=native`
(apple-m3), bench profile. Tree `2008fbf`. The host was a working desktop (load
average 3.7–5). Measured with `scripts/bench_dispatch.sh` and the same
binaries driven round by round.

**The estimator is min over rounds.** macOS cannot pin a thread to a P-core,
and on this host whole stretches of a cell ran about 2.1× slow, the E-core
ratio: in sweep A, 83 of 192 cell × bench × row entries had rounds differing by
more than 1.5×, all near 2.1×. Min over rounds rejects those runs, and this is
the protocol note 31 used on the same machine. After a 4th round of the three
cells where it mattered, 181 of 192 entries have at least two rounds within 10%
of their minimum, and no min-over-rounds ratio is near 2×. The median is not
usable here: it puts `threaded@opblocked` at 1.275 on `forward`.

**Sweep A**: 3 rounds (4 for `match@opblocked`, `threaded@opblocked` and
`match@levelmix`), min over rounds, relative to `match@opblocked`:

| cell | `forward` | lanes4 | lanes8 |
|---|--:|--:|--:|
| `threaded@opblocked` | 1.087 [1.05..1.19] | 1.021 [0.98..1.10] | 1.010 [0.97..1.07] |
| `match@levelmix` | 1.005 [0.99..1.05] | 0.998 [0.98..1.01] | 0.999 [0.98..1.01] |
| `threaded@levelmix` | 1.107 [1.04..1.19] | 1.064 [1.03..1.11] | 1.043 [1.03..1.07] |
| `match@opwin32` | 1.051 [0.99..1.25] | 1.018 [0.98..1.11] | 1.008 [0.99..1.07] |
| `threaded@opwin32` | 1.125 [1.04..1.36] | 1.075 [1.03..1.19] | 1.049 [1.03..1.12] |
| `match@arena` | 1.239 [1.14..1.30] | 1.105 [1.07..1.13] | 1.038 [0.98..1.07] |
| `threaded@arena` | 1.392 [1.28..1.46] | 1.142 [1.10..1.16] | 1.087 [1.04..1.12] |

- **Threaded dispatch loses on the M3 Max**: +8.7% `forward`, 1–7% on lanes.
  The M3's predictor gives threading even less to win than Cascade Lake's.
- **Op-blocked against arena order is −19.3% on `forward`**, the same as
  Cascade Lake's −19.2% and E1's −17.9%. Lanes are 4–10%.
- **`levelmix` matches op-blocked on the M3 Max too** (1.005 / 0.998 / 0.999).
  E1's run-length attribution does not reproduce: the `opwin` penalty E1 saw is
  here too (`opwin32` 1.051), but mean runs of 2–3 in a periodic pattern cost
  nothing.
- **The lanes8 working-set flip is absent.** The 2→6 at lanes8 is 0.980 in arena
  order, against Cascade Lake's 0.837. Its 2.4 MB of op-blocked arenas fit the
  M3's 16 MiB L2.

**The confound, and the shuffle.** `levelmix` deals variants round-robin, so
inside a level the dispatch sequence is periodic, and a history-based indirect
predictor learns periodic sequences. It separates run length from level
structure, but not predictability from level structure.
`Schedule::LevelShuffle` keeps the same levels and orders each one with a
fixed-seed ChaCha8 shuffle. That gives the same operand distances, peak bytes
and mean runs (2.2–3.2) as `levelmix`, with nothing periodic to learn.

**Sweep B**: rebuilt binaries at `2008fbf`, min over rounds. There are 3 rounds
for `opblocked` and `levelmix`, and 2 for `levelshuffle`: round 3 stalled
overnight at 100% CPU and was discarded. The 2→6's rounds agree to 0.03% even
so.

| cell | `forward` | lanes4 | lanes8 |
|---|--:|--:|--:|
| `threaded@opblocked` | 1.114 [1.07..1.20] | 1.063 [1.03..1.10] | 1.042 [1.02..1.07] |
| `match@levelmix` | 1.003 [0.96..1.05] | 0.994 [0.96..1.02] | 0.999 [0.97..1.01] |
| `threaded@levelmix` | 1.108 [1.04..1.19] | 1.063 [1.02..1.11] | 1.044 [1.02..1.07] |
| `match@levelshuffle` | 1.089 [0.95..2.21] | 1.052 [0.97..1.56] | 1.042 [0.99..1.31] |
| `threaded@levelshuffle` | 1.193 [1.04..2.16] | 1.108 [1.02..1.57] | 1.076 [1.03..1.33] |

The rebuild moved `threaded@opblocked` by 2–4% against sweep A, from the same
source apart from a study-only function. That is the scale of
code-layout noise between two builds.

The 2→6 row, µs/event, per round:

| cell | `forward` | lanes4 | lanes8 |
|---|--:|--:|--:|
| `match@opblocked` | 87.60 / 87.66 / 87.58 | 50.65 / 50.88 / 50.60 | 54.62 / 54.71 / 54.67 |
| `match@levelmix` | 88.17 / 88.31 / 88.42 | 50.47 / 50.43 / 50.50 | 54.67 / 54.72 / 54.58 |
| `match@levelshuffle` | 193.21 / 193.25 | 78.90 / 78.77 | 71.44 / 71.47 |
| `threaded@levelshuffle` | 188.83 / 189.44 | 79.37 / 79.51 | 72.62 / 72.56 |

### Reading

- **On the seven small programs a random order within the levels costs
  nothing** (`match` scalar 0.95–1.02). The stream repeats every event, and a
  history-based predictor learns a fixed sequence of a few hundred dispatches
  whatever its order. What arena order loses on them (14–29%) is therefore not
  dispatch: a random order is fine and arena order is not. It is the level
  structure, dependent instructions back to back, as §3 found on Cascade Lake.
- **On the 2→6, predictability matters, and badly.** Shuffling costs 2.2× on
  `forward` (+105 µs/event) under both dispatchers. The periodic interleave and
  op-blocked's runs cost nothing. At about one mispredict per instruction, 36.5k
  instructions × ~12 cycles at ~4 GHz is ≈ 110 µs, which matches. The penalty
  shrinks with lane width, 2.21× → 1.56× → 1.31×, as a fixed per-dispatch cost
  diluted by 4 and 8 lanes of arithmetic would. A locality effect would grow
  with the bytes moved. The counters below confirm it.
- **Threaded dispatch does not rescue an unpredictable stream** (2.16× against
  2.21×). A per-handler branch site still has to predict a random successor.
- **So op-blocking does two jobs.** Level grouping serves every program;
  predictable order serves the long ones, where a stream stops fitting the
  predictor's history. Its runs are one way to be predictable, and a periodic
  interleave is another. A future order that trades against op-blocking, for
  locality or live width, must keep both.
- **The counters agree** (Instruments' CPU Counters template in its default
  bottleneck mode, via `scripts/xctrace_bottlenecks.sh`, no `sudo`). Shares of
  cycles over the last 9 s of a 10 s `--profile-time` loop, one run per cell;
  a repeat of the `match` `gg_to_gg` op-blocked / shuffled pair reproduced it
  to within 1.5 points.

  Each cell is useful / delivery / processing / discarded, in percent:

  | run | `match` | threaded |
  |---|--:|--:|
  | 2→6, op-blocked | 73.9 / 0.7 / 24.1 / 1.3 | 74.5 / 0.6 / 23.8 / 1.0 |
  | 2→6, `levelmix` | 73.6 / 2.0 / 23.2 / 1.1 | 74.9 / 0.7 / 23.3 / 1.1 |
  | 2→6, `levelshuffle` | 33.3 / 6.1 / 14.1 / 46.5 | 41.7 / 5.3 / 13.3 / 39.7 |
  | 2→6, arena | 57.2 / 3.0 / 35.9 / 3.9 | 61.1 / 0.9 / 35.3 / 2.7 |
  | `gg_to_gg`, op-blocked | 79.2 / 9.2 / 3.2 / 8.3 | 86.3 / 5.2 / 4.8 / 3.7 |
  | `gg_to_gg`, `levelmix` | 79.5 / 11.1 / 3.1 / 6.2 | 89.4 / 5.5 / 3.6 / 1.5 |
  | `gg_to_gg`, `levelshuffle` | 82.7 / 9.6 / 3.5 / 4.2 | 90.0 / 5.4 / 3.4 / 1.2 |
  | `gg_to_gg`, arena | 64.8 / 14.9 / 15.8 / 4.6 | 67.9 / 4.7 / 25.7 / 1.7 |

  - **Shuffling the 2→6 flushes nearly half the cycles** (discarded 46.5%,
    against 1.3% op-blocked and 1.1% for the periodic interleave). Useful work
    falls by 2.22×, where the timing measured 2.21×. Shuffled `gg_to_gg`
    discards *less* than op-blocked, under both dispatchers: the predictor has
    memorised the short stream.
  - **Arena order loses to processing stalls, not discards.** Processing goes
    24.1 → 35.9% on the 2→6 and 3.2 → 15.8% on `gg_to_gg`, while discards stay
    under 5%. The useful-share ratios, 1.29 and 1.22, match the timings of
    1.285 and 1.23. That is the dependency-chain mechanism, measured.
  - **On the 2→6 the dispatchers are indistinguishable** in every order but
    the shuffle, where threading discards a little less (39.7% against 46.5%)
    and still loses.
  - **On `gg_to_gg`, threading does what it promises.** Discards drop from
    4–8% to 1–4%, and instruction-delivery stalls roughly halve (9–15% → 5%).
    One branch site per handler predicts and fetches better on a small, hot
    program. Threaded is nonetheless 4–10% *slower* there, with a higher useful
    share, because "useful" counts cycles spent retiring instructions, not
    physics done. Each handler retires 7 more instructions than the `match`
    arm (§5): a frame record, four `Vm` loads, and the stream and kind checks.
    Threading wins its stalls back and spends the difference on that overhead.
    A handler that kept its arena pointers in registers and needed no frame
    record would bank the prediction gain; that is the version worth building
    before threading is written off.

  "Discarded" is Apple's bucket for work flushed after any misprediction,
  mostly branches, not a pure indirect-branch count. An exact count needs a
  counter-selecting template saved from the Instruments GUI.
- **Cascade Lake is untested against the shuffle.** §3's claim that
  op-blocking's win "is not dispatch predictability" holds only for streams the
  predictor memorises, and the 2→6 was not shuffled there. The E1 run-length
  story on the M3 Max was half right. Run length is not what matters, but
  predictability is, once the program is long.

## 5. Profile: what a threaded handler costs beyond the `match` arm

samply 0.13 at 4 kHz, 15 s per profile, on both arms of §4's M3 Max build
(`CARGO_PROFILE_BENCH_DEBUG=line-tables-only`, same flags and features),
running criterion's `--profile-time` loop on `gg_to_gg` and the 2→6, scalar and
lanes4. Symbolicated against each binary's symbol table.

- **Arm-level breakdowns match to 0.5 points.** Interpreter work is 78.8% /
  79.3% (`match` / threaded) on `gg_to_gg` scalar, 75.8 / 76.3 at lanes4, 91.9 /
  92.2 on the 2→6 scalar and 89.6 / 90.1 at lanes4. The criterion loop,
  read-out and external wavefunctions take the same share in both.
- **No accidental overhead in the threaded arm.** There is no `memcpy` or
  `memmove`, no allocation in the hot loop (malloc's 0.3% is evaluator
  construction), and no kernel called out of line in any of the 212 handler
  instances.
- **The `Metric` opcode, instruction by instruction** (`f64`, the hottest in
  both profiles):
  - The `match` arm is 40 instructions plus the loop's shared 10-instruction
    dispatch block: 50 per VM instruction. The arena pointers and lengths stay
    in registers across the loop.
  - The threaded handler is 57 instructions. Its body is the arm's instruction
    sequence exactly: the same 20 floating-point ops, 5 paired loads and 1
    store. The +7 are structural:
    - a frame-record push and pop. The cold panic calls make every handler
      non-leaf, and the Apple arm64 ABI then requires a frame record.
    - four loads of arena pointers and lengths from `Vm`. Six arenas' pointer
      and length pairs do not fit the argument registers a `become` chain
      carries.
    - the remaining-stream check and the kind check.
  - `Metric` is one of the lean handlers, and it understates the cost. A census
    of every handler's prologue in the same binary finds callee-saved register
    saves on top of the frame record. At `f64`, 16 of the 53 handlers save
    between 2 and 16 registers, among them the propagators (14 registers each),
    the multivector `Fin`/`Fout` kernels (16) and the external wavefunctions (6).
    At lanes4, 32 of 53 save up to 18. The `match` loop saves those registers
    once per pass, in `fill_arenas`'s prologue. A threaded handler saves and
    restores them on every instruction it dispatches, because `become` keeps the
    Rust ABI and every handler is a fresh callee. §6 removes this cost.
- **Two costs common to both arms, found on the way.** In the 2→6 the fermion
  propagators call libm `hypot` once per execution: `ComplexFloat::recip` on
  the propagator denominator is num_complex's overflow-safe reciprocal. That is
  ≈1% of scalar time. At lanes4 the call runs once per lane, alongside two
  `memset_pattern16` calls from the lane field's per-lane fallback, ≈2–3% in
  all. Fixed since: the propagators now take `Complex::inv` (`conj / |z|²`),
  and the evaluator makes no `hypot` calls; the amplitude oracle and the
  banked gates pass.
- **Two artefacts not to chase.** `libsystem_kernel` at ~4.5% in every
  profile is wall-clock samples taken while the thread was blocked (criterion's
  gnuplot probe, a rayon latch), not CPU time. Samples on the dylib import
  stubs symbolicate to whatever text symbol precedes the stub section, here
  `RawVec::reserve`, which reads as a per-event allocation that does not exist.
- **Sample skid rules out a ledger.** The dispatch tail (next-opcode load to
  `br`) holds 41% of a `gg_to_gg` handler's samples, and the `match` loop's
  dispatch block 42% of `fill_arenas`'s. A sample lands on whichever
  instruction waits to retire, so those shares say where the core waits, not
  what each instruction costs. The static comparison above is the finer
  evidence.

## 6. The `preserve_none` calling convention

`become` guarantees a tail call and nothing else: the handlers keep the Rust
ABI, with its callee-saved registers and eight argument registers. LLVM's
`preserve_none` convention is the one threaded interpreters use to drop both
(CPython's tail-calling interpreter does). No register is callee-saved, and on
AArch64 arguments go in `x20`–`x28`, then `x0`–`x7` and `x9`–`x15`, 24 in all.
rustc exposes it on the pinned nightly as `extern "rust-preserve-none"` under
`#![feature(rust_preserve_none_cc)]`, and it composes with `become` through the
handler table. The `preserve-none-dispatch` feature (implying
`threaded-dispatch`) builds the handlers this way. Two steps, each
bit-identical to the `match` loop under the dispatcher test, with 807 lib tests
passing:

- **Step 1, the convention alone.** Same handlers, same `Vm` argument. The
  prologue census (§5) drops to no callee-saved saves at `f64` and lanes2, 1
  handler at lanes4 and 2 at lanes8, down from 32 / 22 / 32 / 45 instances. The
  stack loads and stores in handler code fall 43% at `f64`, 19% at lanes4 and
  12% at lanes8. The frame record stays in all 53: it comes from the handlers
  being non-leaf, not from the convention.
- **Step 2, the arenas as arguments.** Each handler takes the rest of the
  stream and nine slices: the six result arenas, the momenta, the operand table
  and the momentum operands, 20 registers in all. One pointer carries the
  constant pools and the environment. The arena pointers and their bounds stay
  in registers down the whole chain, and the per-handler `Vm` loads are gone. The
  cost is new stack traffic. Only `x20`–`x28` survive a call into a Rust-ABI
  function, so the arena arguments in `x0`–`x15` spill around every
  out-of-line kernel or panic call. Stack loads and stores go back up, to 675
  at `f64` against step 1's 342. That is still below the Rust ABI's per-call
  saves, since they sit on the call paths, not in the prologue.

### x86-64: an LLVM bug, so step 2 is AArch64-only

Built for `x86_64-apple-darwin` and run under Rosetta, step 2 segfaults in the
dispatcher test. The Rust-ABI handlers pass there. A standalone probe narrows
it to a codegen bug:
- The probe is a `preserve_none` function that `become`s itself through a
  function-pointer table.
- It is correct with up to 11 integer arguments.
- It returns garbage at 12 under `-O` and segfaults at `-O0`.
- A plain (non-tail) call with 12 arguments is correct.

The assembly shows the cause. x86-64's `preserve_none` passes 12 arguments in
registers, `rax` last. With all 12 in use, LLVM loads the indirect tail call's
target into `rax` (`movq (%r10), %rax; jmpq *%rax`), and the callee receives
its own address as its last argument. Step 2 needs 20 argument registers, so
it cannot work there even without the bug.

Off AArch64, `preserve-none-dispatch` builds step 1: the `Vm` handlers under
`preserve_none`, with 3 argument registers. Its x86-64 build passes the
dispatcher test under Rosetta. Its timing on x86-64 is not measured. The bug is
not reported upstream yet. The probe is 25 lines and would make a clean LLVM
issue.



The first two `preserve_none` sweeps disagreed on the scalar rows by more than
their own round-to-round spread, and the `match` binary, byte-for-byte the same
file in both, moved by +10% on the 2→6 and −17% on `ee_to_mumu` between them.
Every round of a sweep ran in one memory layout, and the layout, not the code,
moved. It follows the process's environment:

- `ee_to_mumu` under `match` runs in 3.92 µs with 0–128 bytes of extra
  environment, 4.80 µs (+22%) with 136–300 bytes and again around 640, and
  3.9–4.0 µs everywhere from 768 bytes to 8 KiB. Each point is steady to 1%.
- The three variables `bench_dispatch.sh` exports put the `match` binary in
  that slow window. The unchanged `preserve_none` step-1 binary reproduced to
  0.3% across both sweeps, and the step-2 binary sits in its own slow window
  (5.06 against 4.18 µs) at a different padding.
- The mechanism is not isolated. The windows are not periodic, and a
  standalone three-row run did not reproduce the 2→6's slow state, so heap
  placement after the preceding benchmarks in the same process is a likelier
  mechanism than stack alignment alone.

`bench_dispatch.sh` now pads the environment by a different length each round,
the same for every arm, and min over rounds takes each cell's best layout among
those drawn. Two consequences:
- The M3 Max scalar figures of §4 were measured in a single layout. The lanes
  columns moved by at most 2 points between layouts here, but a single scalar
  row can move by 10–22%, so their scalar geomeans carry a few points of layout
  error. The Cascade Lake medians were not examined for it.
- The shuffle's 2.2× on the 2→6 is far outside it and stands.

### Result

M3 Max, op-blocked order, min over 6 rounds in 6 layouts, geomean over the 8
rows of time relative to `match`:

| arm | scalar | lanes4 | lanes8 |
|---|--:|--:|--:|
| `preserve_none` step 1 | 1.101 [1.06..1.18] | 1.041 [1.01..1.08] | 1.017 [0.99..1.05] |
| `preserve_none` step 2 | 1.063 [1.01..1.11] | 1.014 [0.98..1.04] | 0.998 [0.98..1.03] |

A second 6-layout sweep ran `match`, the Rust-ABI handlers and step 2:

| arm | scalar | lanes4 | lanes8 |
|---|--:|--:|--:|
| threaded, Rust ABI | 1.113 [1.07..1.20] | 1.063 [1.04..1.10] | 1.043 [1.02..1.07] |
| `preserve_none` step 2 | 1.058 [1.01..1.11] | 1.013 [0.99..1.03] | 1.000 [0.99..1.02] |

- **Step 2 reproduces** across the two sweeps to half a point on every width.
- **Against the Rust ABI, `preserve_none` with register arenas recovers 4–6
  points:** 5.5 at scalar, 5.0 at lanes4 and 4.3 at lanes8.
- **The two steps split that gain.** Comparing across the two sweeps, step 1,
  dropping the register saves, is worth 1.2 / 2.2 / 2.6 points. Step 2, the
  arenas in registers, adds 3.8 / 2.7 / 1.9. The second step needs the first:
  under the Rust ABI, 20 argument registers do not exist.
- The Rust-ABI arm's §4 figures (1.087 / 1.021 / 1.010) came from a single
  layout. Under randomised layouts it is 1.113 / 1.063 / 1.043.

- **Step 2 beats step 1 on every scalar row and on 15 of the 16 lanes rows.**
  Keeping the arenas in registers is worth more than the spills it adds around
  calls.
- **At lanes8 it ties the `match` loop, and it wins on four rows by 1–2%.** At
  lanes4 it is within 1.5%. At scalar precision it still loses 6%, with the 2→6
  11% behind.
- **What remains is structural at scalar width.** It is the frame record, the
  argument spills around out-of-line calls, and the per-instruction stream and
  kind checks. Those are fixed costs per VM instruction, so they weigh most
  where an instruction is cheapest, and the `match` loop pays none of them.

## 7. What this leaves

- **Production order: unchanged.** Op-blocked is the best or tied order for
  both dispatchers on every width and both hosts but one cell (Cascade Lake
  lanes8 2→6). It gives both level grouping and a predictable stream. The
  threaded dispatcher gives no reason to revisit it.
- **Threaded dispatch: not adopted, archived.** It ties at best, needs
  nightly, and would put an incomplete language feature (`explicit_tail_calls`
  still warns as incomplete) into the evaluator. Keeping it buildable would
  pin a nightly, a CI job and the interpreter's shape for a dispatcher that
  loses on the hosts that matter. The code, its bit-identity test and the CI
  job are under the `study/threaded-dispatch` tag; the parts that apply to
  any dispatcher (the control orders, the sweep and counter tooling, §6's
  layout protocol) stayed. The prediction gain is real and measured: on
  `gg_to_gg` threading cuts discards from 8.3% to 3.7% and delivery stalls
  from 9.2% to 5.2% (§4). The Rust ABI spends it on per-handler register
  saves and arena reloads (§5). Under `preserve_none` with the arenas as
  arguments, both are gone and threaded ties `match` at lanes8 and loses 1.4%
  at lanes4 and 6% at scalar (§6). That is still no case for adoption: nightly
  twice over, for a tie at best. The step left untried is a leaf handler, with
  every panic path moved behind a `preserve_none` cold call, so it drops the
  frame record and the argument spills around it. Only the out-of-line kernel
  calls would still spill.
- **A lane-aware order fallback** (open, measure-first): pick `minlive` or
  arena order when `arena bytes × lane width` exceeds L2, which is exactly the
  lanes8 2→6 cell. `Program` is built once per evaluator and shared by every
  `F`, so this needs a per-width program or a per-width order. Size the
  payoff at the width a production lane path would use: lanes4 is where lanes
  would ship on a v3 target, and there the effect is absent. On the M3 Max it
  is absent at every width (16 MiB L2). Any fallback order must stay
  predictable on long programs (§4): a live-width order that dispatches
  randomly would trade an L2 miss for a mispredict per instruction.

## Reproduce

The threaded dispatcher, both nightly features, `scripts/bench_dispatch.sh` and
the bit-identity test live only under the `study/threaded-dispatch` tag
(`4657c92`). Check it out to rerun §1–§6's dispatcher arms. On this branch:

```
# execution-order sweeps of the `match` interpreter (stable toolchain):
scripts/bench_schedule.sh 4 opblocked levelmix levelshuffle arena
# rounds cycle through environment paddings (§6); BENCH_ARMS="name=path" adds a
# prebuilt eval_strategies binary, e.g. one built at the tag, as another arm
# Apple silicon bottleneck breakdown (useful / delivery / processing / discarded):
scripts/xctrace_bottlenecks.sh <eval_strategies binary> 'eval_m2/forward/uux_to_ccx_emmm_qcd0$' \
    opblocked levelmix levelshuffle arena
cargo test -p vibegraph-lib --lib --features eval-schedule-study -- schedule
# order metrics (mean run, live bytes, distances):
cargo test -p vibegraph-lib --lib --features eval-schedule-study -- \
    --ignored --nocapture execution_order_metrics

# at the tag:
git checkout study/threaded-dispatch
DISPATCH_ARMS="match threaded preservenone" scripts/bench_dispatch.sh 6 opblocked
cargo +nightly-2026-09-24 test -p vibegraph-lib --lib \
    --features preserve-none-dispatch -- threaded
```
