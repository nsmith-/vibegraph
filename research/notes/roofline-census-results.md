# Is the evaluator bound by FLOPs or by bytes? — a roofline census

**Status: measurement record, 2026-10-03.** Tree: `db5fd03` plus the census
module this note adds (`helas/eval/roofline.rs`). The question: is the helicity
evaluator limited by floating-point throughput or by moving values from the
caches into registers? No host used so far exposes a PMU, so this reading
avoids counters. It counts the work exactly and divides by measured time.

**Answer.** Not bytes; at scalar width, closer to FP issue than peak FLOPs
suggest. Arena traffic runs at 5–9 B/cycle, under a tenth of L1 bandwidth.
Only 14–26% of FP operations are FMAs, so peak FLOPs is the wrong ceiling.
Against the FP-issue floor of the actual operation mix (§4a), the large
scalar rows run at 51–68% and the small ones at 37–53%. A perfect schedule of
the same operations would be at most about 1.5–2× faster. Dispatch is a small
part of the rest: mispredicts cost about 2% of cycles on the 2→6
(`threaded-dispatch-study-results.md` §4) and bounds checks 3.5–5.5% (note 17
§10). Dependency latency is the measured suspect for the remainder. The lane
paths have the headroom: 25–52% of their floor. Lanes shift the balance. At lanes8 FP ports sit at 25–40% and L1 bandwidth well under its
limit, but the 2→6 stops scaling: lanes8 costs the same per event as lanes4.
Its 3.6 MiB lanes8 working set overflows the 2 MiB L2. That cell is
memory-bound, by cache capacity. The arithmetic intensity is 0.15–0.29
flop per byte of arena traffic on every row. The machine balance is about
0.17 flop/B at L1 (32 flops against 192 B of loads and stores per cycle) and
0.5 at L2. So an evaluator stripped of its interpreter overhead would be
FP-bound while its working set fits L1, and bandwidth-bound once it spills to
L2.

## 1. Host

Intel Xeon family 6 model 207 (Emerald Rapids), 4-vCPU Firecracker VM, 48 KiB
L1d and 2 MiB L2 per core, 260 MiB L3. AVX-512 (`target-cpu=native`, so
`HARDWARE_FMA` and lanes8 on zmm).

**Clock: measured, 3.2 GHz.** The VM reports only its 2.1 GHz base. A chain of
dependent integer `add`s, which take 1 cycle each, ran at 3.16–3.32 G adds/s
pinned to one idle core (five repeats). Every per-cycle figure below uses
3.2 GHz. AVX-512 frequency licensing can lower the clock under zmm load, which
would make the lanes8 per-cycle figures somewhat higher than shown.

**Peaks**, from the Golden Cove / Raptor Cove microarchitecture, not measured
here:
- FP issue: two FMA-capable ports at every width. Scalar and ymm adds also go
  to a third port. 4 / 16 / 32 flops per cycle at scalar / ymm / zmm.
- L1d: three loads per cycle up to 256 bits, two at 512, plus two stores. That
  is 96–128 B/cycle of loads.
- L2: 64 B/cycle peak.

## 2. The census

`roofline_census` (ignored test, `helas/eval/roofline.rs`) runs on the eight
`eval_strategies` bench rows, with the bench's seed and its 16 points per row.

- **Arithmetic is counted, not modelled.** `eval_m2` runs at `F = Counted`, an
  `f64` newtype that tallies every operation it performs. It is the production
  code path at another `F`. It covers the kernels, the momentum pool, the
  external wavefunctions and the helicity and colour read-out. Every counted
  `|M|²` equals the `f64` one bit for bit (asserted per point). A dozen
  operations per event come from the partonic-CM input check, which debug
  builds run.
- **Traffic is counted from the compiled program.** An instruction reads its
  `arena_reads` operands plus the pool and momentum entries it indexes, and
  writes one element of its own class. `arena_reads` is the edge set slot
  recycling relies on, so a read missing from it would be a live value the
  allocator could overwrite. Sizes are at `f64`. A lane pack moves N× the bytes
  for N events, so bytes per event do not change with width.
- **What traffic leaves out.** It counts values consumed and produced, not
  machine loads. The instruction stream, `Vec` headers, spills and a kernel's
  by-value operand copies are not in it.
- **Arena KiB** is the allocated arenas, each class's peak slot count times its
  element size, summed. It is the footprint the pass touches, and larger than
  the joint live peak (302 KB on the 2→6) that
  `threaded-dispatch-study-results.md` quotes.
- **Counting conventions.** An FMA is two flops. Negations are counted apart and
  excluded: 17% of FP operations on the 2→6, often folded into an FMA
  variant. "FP instructions" counts element operations at scalar width and
  vector instructions (÷ N) for lanes. The scalar build SLP-packs some complex
  pairs into xmm, so its count is an upper bound.

| process | VM instrs | flops/event | FMA share of FP ops | arena B/event | flop/B | arena KiB (f64) |
|---|--:|--:|--:|--:|--:|--:|
| `ee_to_mumu` | 61 | 1 629 | 16% | 5 904 | 0.28 | 1.4 |
| `ee_to_wpwm` | 425 | 9 372 | 15% | 53 256 | 0.18 | 6.0 |
| `uux_to_uux` | 123 | 2 662 | 18% | 12 032 | 0.22 | 2.3 |
| `gg_to_gg` | 682 | 10 186 | 14% | 66 832 | 0.15 | 7.4 |
| `gg_to_ttx` | 328 | 6 426 | 23% | 27 792 | 0.23 | 3.6 |
| `ee_to_mumua` | 346 | 10 915 | 23% | 38 224 | 0.29 | 6.2 |
| `ee_to_mumu_tata_qcd0` | 1 739 | 51 587 | 25% | 184 096 | 0.28 | 22.3 |
| `uux_to_ccx_emmm_qcd0` | 36 506 | 1 060 891 | 26% | 3 887 728 | 0.27 | 450.9 |

A VM instruction averages 15–32 flops and 85–125 bytes. The 2→6's 36 506
instructions are within 17 of the 36 523 that
`threaded-dispatch-study-results.md` records for it.

## 3. Achieved rates

Timings come from `eval_strategies` (`forward`, `lanes2/4/8`), criterion
medians, one run, `target-cpu=native`, on the same VM. Criterion's intervals
are ±5–10% per cell. Scalar rows also carry the up-to-22% memory-layout
confound recorded in `threaded-dispatch-study-results.md` §6. Every reading
below rests on factor-level differences.

| process | N | ns/event | FP instrs/cycle | flops/cycle | B/cycle | arena × N |
|---|--:|--:|--:|--:|--:|--:|
| `ee_to_mumu` | 1 | 385 | 1.14 | 1.32 | 4.8 | 1 KiB |
| | 4 | 142 | 0.77 | 3.58 | 13.0 | 6 KiB |
| | 8 | 100 | 0.54 | 5.07 | 18.4 | 11 KiB |
| `ee_to_wpwm` | 1 | 1 803 | 1.42 | 1.62 | 9.2 | 6 KiB |
| | 4 | 673 | 0.95 | 4.35 | 24.7 | 24 KiB |
| | 8 | 480 | 0.67 | 6.10 | 34.7 | 48 KiB |
| `uux_to_uux` | 1 | 639 | 1.11 | 1.30 | 5.9 | 2 KiB |
| | 4 | 221 | 0.80 | 3.76 | 17.0 | 9 KiB |
| | 8 | 177 | 0.50 | 4.71 | 21.3 | 18 KiB |
| `gg_to_gg` | 1 | 2 482 | 1.12 | 1.28 | 8.4 | 7 KiB |
| | 4 | 889 | 0.78 | 3.58 | 23.5 | 30 KiB |
| | 8 | 629 | 0.55 | 5.06 | 33.2 | 59 KiB |
| `gg_to_ttx` | 1 | 1 415 | 1.16 | 1.42 | 6.1 | 4 KiB |
| | 4 | 457 | 0.90 | 4.39 | 19.0 | 14 KiB |
| | 8 | 322 | 0.64 | 6.24 | 27.0 | 29 KiB |
| `ee_to_mumua` | 1 | 1 678 | 1.66 | 2.03 | 7.1 | 6 KiB |
| | 4 | 573 | 1.21 | 5.95 | 20.8 | 25 KiB |
| | 8 | 459 | 0.76 | 7.44 | 26.0 | 50 KiB |
| `ee_to_mumu_tata_qcd0` | 1 | 7 562 | 1.71 | 2.13 | 7.6 | 22 KiB |
| | 4 | 2 512 | 1.29 | 6.42 | 22.9 | 89 KiB |
| | 8 | 1 999 | 0.81 | 8.06 | 28.8 | 178 KiB |
| `uux_to_ccx_emmm_qcd0` | 1 | 173 844 | 1.52 | 1.91 | 7.0 | 451 KiB |
| | 2 | 96 937 | 1.36 | 3.42 | 12.5 | 902 KiB |
| | 4 | 64 512 | 1.02 | 5.14 | 18.8 | 1.8 MiB |
| | 8 | 62 426 | 0.53 | 5.31 | 19.5 | 3.6 MiB |

### Reading

- **Scalar FP issue is busy but not saturated.** It runs at 1.1–1.7 per cycle,
  an upper bound because of SLP packing, against 2–3 FP-capable ports. That is
  33–53% of the 4 flops/cycle scalar peak. The large rows sit at the top of the
  range.
- **Lanes use the FP units less, not more.** Vector FP issue falls with width,
  to 0.5–0.8 per cycle at lanes8: 25–40% of two zmm ports, 15–25% of the
  32 flops/cycle peak.
- **Bytes are far from the L1 limit.** Arena traffic peaks at 35 B/cycle
  (`ee_to_wpwm` lanes8), against 96–128 B/cycle of L1 loads. Scalar rows move
  5–9 B/cycle.
- **The 2→6 has a capacity cliff.** It gains 1.79× from width 1 to 2 and 1.50×
  from 2 to 4. From 4 to 8 it gains 1.03×, where the other rows gain
  1.2–1.4×. Its arenas are 1.8 MiB at lanes4, about the 2 MiB L2, and 3.6 MiB
  at lanes8, past it. This is the Cascade Lake lanes8 effect of
  `threaded-dispatch-study-results.md` §3, reproduced on a host with twice the
  L2. Here it costs the whole 4→8 gain.

## 4. Width-independent vs per-lane cost

Every lane width executes the same VM instructions: the same dispatch, bounds
checks and dependency chains, on wider registers. So the cycles one pass
spends per instruction, `T(N) = time × N / instrs`, split into a part that
does not depend on N and a part that grows with it. The second part is what
FLOPs and bytes cost.

| process | cycles per pass-instruction, N = 1 / 2 / 4 / 8 | 1→4 slope (cycles/lane) | share of T(1) independent of N |
|---|---|--:|--:|
| `ee_to_mumu` | 20.2 / 26.2 / 29.9 / 42.2 | 3.0 | 85% |
| `ee_to_wpwm` | 13.6 / 16.9 / 20.3 / 28.9 | 2.2 | 84% |
| `uux_to_uux` | 16.6 / 21.5 / 23.0 / 36.8 | 1.9 | 88% |
| `gg_to_gg` | 11.6 / 14.0 / 16.7 / 23.6 | 1.7 | 86% |
| `gg_to_ttx` | 13.8 / 15.4 / 17.8 / 25.1 | 1.3 | 90% |
| `ee_to_mumua` | 15.5 / 18.4 / 21.2 / 33.9 | 1.8 | 88% |
| `ee_to_mumu_tata_qcd0` | 13.9 / 16.4 / 18.5 / 29.4 | 1.5 | 89% |
| `uux_to_ccx_emmm_qcd0` | 15.2 / 17.0 / 22.6 / 43.8 | 2.5 | 83% |

The share is `1 − slope / T(1)`, with the slope a least-squares fit over widths 1, 2 and 4. Those
widths stay on xmm and ymm, where the scalar and lane code use the same ports.
The 4→8 step is steeper on every row (zmm on two ports, possible licence
downclock) and very steep on the 2→6, the capacity cliff.

- **At scalar width 83–90% of a VM instruction's time is width-independent.**
  It is the 10–18 cycles a pass-instruction costs before any lane is added.
  That bucket holds dispatch, operand indexing and bounds checks, call and
  copy glue, each kernel's dependent FP chain, and FP issue itself. One ymm
  FMA takes the same port slot as one scalar FMA, so issue cost does not grow
  with width up to 256 bits. Width-independence therefore cannot separate
  dispatch from arithmetic; §4a weighs the arithmetic directly.
- **The per-lane cost is FP throughput more than bytes.** Each lane adds
  1.3–3.0 cycles for the 13–26 FP operations and 85–125 bytes of a VM
  instruction. That is 8–16 element FP operations per marginal cycle, at or
  just past what two to three ymm FP ports issue (8–12), and 32–73 B, a third
  to two thirds of L1 load bandwidth. Once the fixed part is amortised, the FP
  units are the first width-proportional limit. At lanes8 the per-lane part is
  about half of the per-instruction time.

## 4a. The FP-issue floor of the operation mix

Peak FLOPs assumes every operation is an FMA. A floor for this code uses the
census's actual mix and the core's ports:
- **Scalar and ymm:** multiplies and FMAs go to two ports and adds to three, so
  the floor is `max((mul + fma) / 2, (all FP ops) / 3)` cycles per event, ÷ N
  at width N.
- **zmm:** every FP operation goes to two ports, `(all FP ops) / 2 / 8`.

The second column of the table also counts negations, which may be folded
into FMA variants, as port work.

| process | scalar | lanes4 | lanes8 |
|---|--:|--:|--:|
| `ee_to_mumu` | 38–44% | 26–30% | 27–32% |
| `ee_to_wpwm` | 47–53% | 32–36% | 33–37% |
| `uux_to_uux` | 37–44% | 27–32% | 25–30% |
| `gg_to_gg` | 37–42% | 26–29% | 28–31% |
| `gg_to_ttx` | 39–46% | 30–35% | 32–38% |
| `ee_to_mumua` | 55–66% | 40–48% | 38–45% |
| `ee_to_mumu_tata_qcd0` | 57–68% | 43–52% | 40–49% |
| `uux_to_ccx_emmm_qcd0` | 51–61% | 34–41% | 26–32% |

The floor assumes a perfect schedule and treats each counted element operation
as an FP instruction. The scalar build SLP-packs some complex pairs, which
lowers its true floor, and shuffles raise it, so read these to about ±10%.

- **Scalar is not far from its ceiling.** The large rows sit at half to two
  thirds of the floor, so removing every other cost would buy at most about
  1.5–2×. Of the gap, dispatch accounts for under a tenth of cycles
  (mispredicts and bounds checks above). Dependency latency is the measured
  suspect for the rest: interning order, which puts dependent instructions
  back to back, costs 19% over op-blocked. Further scalar gains need fewer FP
  operations or shorter chains.
- **The lane paths are where the headroom is**, at 25–52% of their floor. The
  costs that grow with width are what hold them there: zmm's two FP ports,
  N× the bytes per instruction including by-value copies of 512-byte
  temporaries, and on the 2→6 the L2 overflow.

## 5. What this does not settle

- **Machine loads are not counted.** The traffic is values. Header reloads,
  spills and by-value kernel copies all add loads, so the real L1 load rate is
  higher than the B/cycle column. The L1 margin (3–20×) is large enough that
  this does not change the reading, but a per-row machine-load count needs a
  PMU or an instrumented build.
- **L1 hits versus L2 traffic.** The arena column gives a working set, not a
  miss rate. Most reads are of recently written values. Only the 2→6's
  stalled scaling shows a capacity effect directly.
- **The gap between scalar time and the FP-issue floor is not split** into
  latency, non-FP issue (loads, address arithmetic) and front-end stalls. A
  top-down reading on a host with a PMU would give the split.
- **One host, one run.** The census counts do not depend on the host. The
  rates do, and the M3 Max and Cascade Lake have different port counts and L2
  sizes.

## Reproduce

```
# the census (counts are host-independent; FMA vs mul+add split follows HARDWARE_FMA)
RUSTFLAGS="-C target-cpu=native" cargo test -p vibegraph-lib --lib roofline -- \
    --include-ignored --nocapture
# timings on the same host
RUSTFLAGS="-C target-cpu=native" cargo bench -p vibegraph-lib --bench eval_strategies -- \
    'eval_m2/(forward|lanes2|lanes4|lanes8)/'
```

Clock probe (scratch, not in tree): 20 dependent `add %0,%0` per iteration in
inline asm, 2×10⁸ iterations, pinned with `taskset`; adds per second is the
clock.
