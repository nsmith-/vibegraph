# `fill_arenas` instruction-level study

**Date:** 2026-08-04 · **Host:** Apple M3 Max, macOS 15.7.7, `aarch64-apple-darwin` · **Repo:** `/Users/ncsmith/src/generators/vibegraph` @ `9bad54c` (shared main checkout; no source edited, nothing committed, `validation/` untouched)
**Target:** `vibegraph::helas::eval::run::fill_arenas` (`vibegraph-lib/src/helas/eval/run.rs:856`), `F = f64`, `release-debug` (thin LTO, `opt-level=3`, `debug=1`), `extended-validation`.

## 0. Headline

| verdict | answer |
|---|---|
| Jump table? | **Yes** — 38-entry `u16` offset table, `adr`/`ldrh`/`add`/`br`. Not a compare chain. |
| Time split | **30.5% loop control + dispatch** (10 instructions), **22.9% loads**, **20.4% FP arithmetic**, **20.4% bounds checks**, 3.5% stores |
| Top 3 regions | loop control **18.50%**, `GammaVout` (inlined `gamma_vout_bare`) **18.17%**, `Metric` (inlined `metric_bare`) **14.09%**; dispatch block **11.99%** on 5 instructions |
| Biggest structural finding | **~1 instruction in 5 is arithmetic.** `MulScalarR` is 17 instructions for one `fmul.2d`. The arena `Vec` ptr/len fields are re-loaded from memory on **every** instruction — 143 loads off `x19` — because LLVM can't prove the arena stores don't alias the `ScratchSpace` headers. |

`fill_arenas` = **33.6%** of busiest-thread self time here, reproducing note 30 §7.2's 34.2% on the same gate.

## 1. Jump-table verdict: jump table, 38 `u16` entries

```
;         if self.index < self.len {
10014e2a8: eb18033f     cmp  x25, x24
10014e2ac: 5400ace0     b.eq 0x10014f848                    ; loop exit
;         unsafe { intrinsics::offset(self, count) }
10014e2b0: 9b0d532c     madd x12, x25, x13, x20             ; instr ptr = base + i*20
;     for (instr, &loc) in prog.instrs.iter().zip(prog.loc.iter()) {
10014e2b4: b8797b9a     ldr  w26, [x28, x25, lsl #2]        ; loc[i]
;             self.index += 1;
10014e2b8: 91000739     add  x25, x25, #0x1
;         match *instr {
10014e2bc: 39400188     ldrb w8, [x12]                      ; discriminant byte
10014e2c0: 10ffff49     adr  x9, 0x10014e2a8                ; table-relative base
10014e2c4: 786879ca     ldrh w10, [x14, x8, lsl #1]         ; JT[disc] : u16
10014e2c8: 8b0a0929     add  x9, x9, x10, lsl #2
10014e2cc: d61f0120     br   x9
```

`x14` is the table base, materialized once in the preheader (`adrp x14, 0x10025a000` + `add x14, x14, #0xef8` → `0x10025aef8`). There is **exactly one `br`** in the whole function and no `cmp #k`/`b.eq` chain on the discriminant.

The table itself (`xxd` at file offset `0x25aef8`; `__TEXT` is `vmaddr 0x100000000, fileoff 0`, verified via `otool -l`):

```
0025aef8: 000a 0184 028b 0190 0136 014c 01c8 0332
0025af08: 0163 01a7 02ff 00f8 0299 00d0 02d3 0321
0025af18: 03eb 02e8 02f3 03df 03f7 027f 01fe 04e6
0025af28: 0054 0422 0472 035e 02c4 03d0 0120 00a4
0025af38: 0030 003e 0015 0000 0000 0000 | 00f9 00fe
                       ^entry35 ^36 ^37
```

Three checks pin the reading:
- **38 entries** (`0x25aef8`..`0x25af44`) and `Instr` has **38 variants** (`layout.rs:76`).
- **entry[0] = 0x000a** → `0x10014e2a8 + 0x0a*4 = 0x10014e2d0`, which the interleaved source labels `Instr::ComplexConst { pool } => …`. ✅
- **entries[35,36,37] = 0x0000** → target `0x10014e2a8 + 0` = the loop back-edge itself. Those are the last three variants, `Instr::Flows | Instr::Hels | Instr::Configs => {}` (`run.rs:1100`) — the empty arms branch straight into loop control. ✅

**The pre-LTO view agrees.** `cargo asm -p vibegraph-lib --lib --rust fill_arenas` emits the identical sequence against `LJTI1725_0`:

```
	ldrb w8, [x12]
	adr x9, LBB1725_29
	ldrh w10, [x14, x8, lsl #1]
	add x9, x9, x10, lsl #2
	br x9
```

with the same call census. **2050 mnemonics (cargo-asm, pre-LTO) vs 2041 (linked binary)** — thin LTO changed this function by 9 instructions.

## 2. Hot regions inside `fill_arenas`

Busiest thread `sigma_gate_matches_madgraph`: **41 681** sampled stacks @ 1000 Hz. Leaf inside `fill_arenas`: **14 009 = 33.6%**. `fill_arenas` anywhere in stack: 19 859 = 47.6% (the extra 14% is the four non-inlined kernels). **580 of 2041 instructions took ≥1 sample.** Percentages are of 14 009.

| # | region | insns | samples | % | smp/insn | what the instructions do |
|---:|---|---:|---:|---:|---:|---|
| 1 | **LOOP CONTROL** (`zip::next`, `run.rs:862`) | 31 | 2592 | **18.50%** | 83.6 | back-edge `cmp`/`b.eq`, `madd` (20-byte `Instr` stride), `ldr` of `loc[i]`. 2326 (16.60%) on the single `cmp` at `0x10014e2a8` — the merge point every arm branches to. |
| 2 | **`GammaVout`** — `gamma_vout_bare` inlined | 139 | 2546 | **18.17%** | 18.3 | dense scalar-lane FP: `fmul d`, `fmadd`, `fnmsub`, `fmsub`, `fneg` on `d0–d31`, fed by `ldp d,d`. **Not vectorized.** |
| 3 | **`Metric`** — `metric_bare` inlined | 56 | 1974 | **14.09%** | 35.2 | complex Lorentz dot product: `ldp q` from both vector arenas, `fsub`/`mul_add` chain. Highest density of any real kernel. |
| 4 | **DISPATCH** (`run.rs:864`) | 5 | 1679 | **11.99%** | **335.8** | `ldrb`/`adr`/`ldrh`/`add`/`br`. **1523 (10.87%) on the `ldrh` alone.** |
| 5 | `MulScalarR` | 32 | 1024 | 7.31% | 32.0 | one `fmul.2d`; 16 of 17 instructions are overhead (§4.1) |
| 6 | `MulScalarC` | 33 | 622 | 4.44% | 18.8 | `fmul.2d` + `ext.16b` + `fsub.2d`/`fadd.2d` + `mov.d` lane merge |
| 7 | `AddScalar` | 54 | 590 | 4.21% | 10.9 | variadic loop: `ldr w11,[x8],#0x4`, `and x0,x11,#0x1fffffff` (unpack `OperandRef`), bounds check, `ldr q1`, `fadd.2d` |
| 8–9 | `GammaFin` / `GammaFout` | 91 / 91 | 384 / 340 | 2.74% / 2.43% | 4.2 / 3.7 | inlined `off_shell_f{in,out}_bare` |
| 10 | `ScaleFoutC` | 54 | 285 | 2.03% | 5.3 | `Bispinor * Complex`, 4× complex-mul blocks |
| 11 | `FfvVout` | 58 | 247 | 1.76% | 4.3 | **not inlined** — sret call + 64-byte stack→arena copy (§4.3) |
| 12 | `PropagateVector` | 62 | 207 | 1.48% | 3.3 | **not inlined** |
| 13 | `ScaleVecR` | 36 | 185 | 1.32% | 5.1 | `ldr d0` + 2× (`ldp q,q` / `fmul.2d` ×2 / `stp q,q`) |
| 14–15 | `FfvFout` / `FfvFin` | 135 / 137 | 181 / 169 | 1.29% / 1.21% | 1.3 / 1.2 | inlined `ffv_f{out,in}_bare` |
| 16 | `ExternalFin` / `ExternalFout` | 32 / 32 | 137 / 137 | 0.98% each | 4.3 | `bl build_external_slot` |

Remaining 24 arms total **3.0%**.

**Independent cross-check** — bucketing the same 14 009 samples by *innermost* inlined frame instead of outermost:

```
   4724  33.72%  fill_arenas body itself (dispatch, stores, bounds cmps)
   2390  17.06%  loop control (zip iterator next)
   2085  14.88%  Vec header deref (as_slice / as_mut_slice / RawVecInner::non_null)
   1565  11.17%  plain f64 arith ops (add/sub/mul/neg/div)
   1186   8.47%  FMA (core::f64::math::mul_add)
   1008   7.20%  vibegraph repr arithmetic (ComplexVector / Bispinor)
    891   6.36%  slice bounds-check indexing (SliceIndex::index / index_mut)
    160   1.14%  other
```

17.06% for `zip::next` matches the 18.50% loop-control bucket; 14.88% + 6.36% = **21.2% in `Vec` header deref + slice bounds-check indexing**, matching §3's 20.4% opcode-level bounds-check figure from a completely different classifier.

## 3. Dispatch vs arithmetic vs memory vs bounds checks

All 2041 instructions classified by opcode, weighted by samples. Bounds checks identified *structurally*: a conditional branch whose target lies past the epilogue `0x10014f848` (i.e. into the `panic_bounds_check` tail), plus the feeding `cmp` and the `ldr` of the container length.

```
samples       %  #insns  kind
   4266  30.45%      10  LOOP CONTROL + DISPATCH block  (0x10014e2a8..0x10014e2cc)
   3211  22.92%     355  load
   2864  20.44%     451  FP arithmetic
   2854  20.37%     302  bounds check (cmp / branch / len load)
    485   3.46%      95  store
    262   1.87%     586  integer / address math
     63   0.45%      27  SIMD shuffle / lane glue
      4   0.03%     215  branch / dispatch (other)
```

As a budget:

- **~30% is per-iteration dispatch overhead.** Ten instructions carry 30.45%. They execute **once per program instruction** (100% frequency) while any arm executes only at its share — ten loop instructions against a mean executed arm body in the low twenties makes ~30% arithmetically unsurprising. Real overhead, not an attribution artifact.
- **~20% is genuine floating-point work** — the actual matrix element, 451 FP instructions.
- **~20% is bounds checking**, on 302 instructions and **108 distinct `panic_bounds_check` call sites**. Nearly all avoidable: `loc`/operand indices against loop-invariant arena lengths.
- **~23% loads / 3.5% stores.** Much of the load traffic isn't data: **143 loads off `x19`** (the `ScratchSpace` pointer) at 13 distinct offsets re-fetch arena `ptr`/`len`.
- Integer/address math and non-dispatch branches are essentially free (1.9% on 586 insns; 0.03% on 215).

Only **12 of 14 009** samples land at or past the epilogue, so the cold panic tail costs nothing at run time — its cost is the ~20% of *hot-path* instructions guarding it plus layout pressure from 108 landing pads.

## 4. Other structural findings for a sprint session

### 4.1 The exhibit: `MulScalarR` is 17 instructions for one multiply (7.31%)

```
;             Instr::MulScalarR { s, r } => {
10014ef2c: b9400580     ldr  w0, [x12, #0x4]        ; operand s
10014ef30: f9401661     ldr  x1, [x19, #0x28]       ; scalars.len  <-- reload   329 smp (2.35%)
10014ef34: eb00003f     cmp  x1, x0                 ; bounds check
10014ef38: 54005869     b.ls 0x10014fa44             ; -> panic_bounds_check
10014ef3c: b9400988     ldr  w8, [x12, #0x8]        ; operand r
10014ef40: f9400a69     ldr  x9, [x19, #0x10]       ; reals.len    <-- reload   207 smp (1.48%)
10014ef44: eb08013f     cmp  x9, x8                 ; bounds check
10014ef48: 54005849     b.ls 0x10014fa50             ; -> panic_bounds_check
10014ef4c: eb1a003f     cmp  x1, x26                ; bounds check (dest)       218 smp (1.56%)
10014ef50: 540058a9     b.ls 0x10014fa64             ; -> panic_bounds_check
10014ef54: f9400669     ldr  x9, [x19, #0x8]        ; reals.ptr    <-- reload
10014ef58: fc687920     ldr  d0, [x9, x8, lsl #3]   ; the real
10014ef5c: f9401268     ldr  x8, [x19, #0x20]       ; scalars.ptr  <-- reload
10014ef60: 3ce07901     ldr  q1, [x8, x0, lsl #4]   ; the complex
10014ef64: 4fc09020     fmul.2d v0, v1, v0[0]       ; <-- THE ARITHMETIC (1 of 17)
10014ef68: 3cba7900     str  q0, [x8, x26, lsl #4]
10014ef6c: 17fffccf     b    0x10014e2a8             ; back to loop control
```

Four header reloads and three bounds-check pairs around one SIMD multiply — and the two hottest instructions in the arm are the two `len` reloads. Hoisting the arenas into local slices once before the loop would let LLVM keep `ptr`/`len` in registers across all 38 arms and make the bounds checks hoistable.

### 4.2 The dispatch's own stall: load-dependent indirect branch

```
0x10014e2a8  +0x258  2326  16.604 %   cmp x25, x24      <- merge point of all 38 arms
0x10014e2ac  +0x25c    60   0.428 %   b.eq (loop exit)
0x10014e2b0  +0x260   145   1.035 %   madd x12, x25, x13, x20
0x10014e2b4  +0x264    52   0.371 %   ldr w26, [x28, x25, lsl #2]
0x10014e2b8  +0x268     4   0.029 %   add x25, x25, #1
0x10014e2bc  +0x26c    59   0.421 %   ldrb w8, [x12]
0x10014e2c0  +0x270     8   0.057 %   adr x9, ...
0x10014e2c4  +0x274  1523  10.872 %   ldrh w10, [x14, x8, lsl #1]   <- jump-table load
0x10014e2c8  +0x278    67   0.478 %   add x9, x9, x10, lsl #2
0x10014e2cc  +0x27c    22   0.157 %   br x9
```

The `ldrh` at 10.87% is a table load the `br` consumes two instructions later — a 38-way indirect branch on a data-dependent stream is the classic interpreter mispredict, and the cost lands on the load that resolves it. **Actionable shapes:** replicate the dispatch tail per arm (a `br` at the end of every arm, giving the predictor 38 independent histories); or sort/block the program by opcode so runs of identical instructions dispatch once (`AddScalar` + `MulScalar*` alone are 16%).

### 4.3 Non-inlined kernels pay a stack round-trip *and* clobber the loop constants

Exactly four kernels didn't inline — `ffv_vout_bare`, `propagate_vector_bare`, `propagate_fin_bare`, `propagate_fout_bare`, one call site each (their own self time is charged to their own symbols in note 30, *outside* this 33.6%). The call sites cost inside `fill_arenas`:

```
10014f6a4: 9105c3e0     add  x0, sp, #0x170          ; sret temp on the stack
10014f6a8: 94003bbd     bl   0x10015e59c <...ffv_vout_bare>
10014f6ac: f9402261     ldr  x1, [x19, #0x40]        ; vectors.len reload
10014f6b0: eb1a003f     cmp  x1, x26
10014f6b4: 54003da9     b.ls 0x10014fe68              ; -> panic_bounds_check
10014f6b8: f9401e68     ldr  x8, [x19, #0x38]        ; vectors.ptr reload
10014f6bc: 8b1a1908     add  x8, x8, x26, lsl #6
;                 scratch.vectors[loc] = out;
10014f6c0: ad4b87e0     ldp  q0, q1, [sp, #0x170]    ; 64-byte copy stack -> arena
10014f6c4: ad000500     stp  q0, q1, [x8]
10014f6c8: ad4c87e0     ldp  q0, q1, [sp, #0x190]
10014f6cc: ad010500     stp  q0, q1, [x8, #0x20]     ; 77 smp (0.55%)
10014f6d0: 5280028d     mov  w13, #0x14              ; rematerialize Instr stride
10014f6d4: f000084e     adrp x14, 0x10025a000        ; rematerialize jump-table base
10014f6d8: 913be1ce     add  x14, x14, #0xef8
```

Three taxes: the kernel writes its 64-byte result into a stack temp then copies it to the arena through four q-register moves (store-to-load forwarding of callee-written 32-byte pairs — pure redundant traffic); and `w13`/`x14`, the loop's two invariant constants, are caller-saved and rebuilt after every such call (`mov w13,#0x14` appears exactly twice in the function — preheader and here). Passing `&mut scratch.vectors[loc]` into the kernel eliminates the copy; inlining them eliminates the rematerialization.

### 4.4 Large frame, 10 stack round-trips

`sub sp, sp, #0x280` — a **640-byte** local frame beyond the 128-byte register save area, with 10 `ldp q,q,[sp,…]` and 8 `stp q,q,[sp,…]` pairs.

### 4.5 `gamma_vout_bare` is scalar-lane, not packed

The largest kernel (18.17%) computes complex arithmetic in `d` registers (`fmul d3,d3,d4` / `fmadd d2,d2,d5,d3` / `fnmsub` / `fneg`) while the cheap arms use packed `.2d` on `q`. Whether a packed complex layout beats the current scalar-FMA form is open — the FMA chains are already dense — but it's the one representation change touching 18% of the symbol.

### 4.6 Tail merging

219 of 2041 instructions (10.7%) carry DWARF **line 0** — LLVM tail-merged blocks shared between arms with identical shapes (`ScaleFinC`/`ScaleFoutC`, the `Add*` operand loops). 1080 samples (7.71%) landed on them.

## 5. Anomalies and limitations

1. **PC attribution / skid.** samply samples the thread PC via timer interrupt; the reported PC is roughly the oldest un-retired instruction, so a stall is charged to the *waiting* instruction. This is why the `cmp` at the arms' merge point carries 16.60% — some is genuine loop control, some is the preceding arm's store/FP chain draining. **The block total (30.45%) is robust; the split within the block is not.** Don't quote "the `cmp` costs 16.6%" as an isolated instruction cost.
2. **Line-0 forward fill.** 10.7% of instructions have no source line; they were attributed to the nearest preceding *resolved* instruction's arm, moving 1080 samples (7.71%). Two of the largest were hand-checked against the disassembly and agreed (`0x10014eefc` → `AddScalar`'s operand loop, confirmed by interleaved `value = value + scratch.scalars[op.index()]`; `0x10014f318` → `ScaleFoutC`'s complex-multiply block, confirmed by `scratch.fout[loc] = scratch.fout[f] * scratch.scalars[scale]` at `0x10014f2ac`). The rest weren't individually verified — treat sub-1% arm shares as ±0.5% uncertain. Raw-vs-filled counts per arm are in `fas-bucket2.log`.
3. **cargo-show-asm could not target the test binary.** `cargo asm … --test validate_sigma` fails with `Error: Cannot locate the path to the asm file` under `--profile release-debug` — cargo-show-asm 0.2.62 gets bitcode, not `.s`, for that target. `--lib` worked and is what §1 quotes. For *this symbol* the views agree to 9 instructions out of ~2045 with identical dispatch code and call census; that is a measured coincidence for `fill_arenas`, **not** a general licence to read cargo-asm as the linked binary under LTO. All §2–§4 numbers come from the linked binary.
4. **Symbolication.** No `llvm-symbolizer` on this host (`xcrun -f llvm-symbolizer` fails); used `atos -i` against a `dsymutil` dSYM, which reports the full inline chain. The samply profile itself is *unsymbolicated* (`nativeSymbols.length == 0`; names live in the `.syms.json` sidecar), so frames were mapped by raw library-relative address against `nm`, with `__TEXT vmaddr = 0x100000000` verified by `otool -l`. **Per-address granularity was achieved** — the finest the brief asked for.
5. **One binary, one run.** Single 41.8 s profile. Profiled binary sha256 `44c5c8323a8377ca924eb7685ba0011caccbea5569ec8151b2d079ff495e14c9` (`validate_sigma-2080c22985a752c1`), re-verified unchanged *after* the cargo-asm runs (which produced a differently-hashed `validate_sigma-9897a11f4a43edff` but didn't disturb the profiled one). No repeat run; hybrid P/E placement (no affinity set) can move absolute times but shouldn't move within-symbol shares.
6. **The gate, not one process.** `sigma_gate_matches_madgraph` loops over every banked partonic directory, so the arm mix is the σ-gate's process mix weighted by integration cost. A `gg → ttx`-only profile would shift the `GammaVout` / `Metric` / `Ffv*` balance.
7. **No zombie processes.** Nothing was killed; build (4.13 s — `target/` was *not* cold as the brief anticipated), the 41.8 s profile, `dsymutil`, and both cargo-asm runs all exited 0. `ps` shows no stray `cargo`/`samply`/`rustc`.
8. **`extended-validation` is on**, matching note 30. `validate_arenas` stays compiled in but is `#[inline(never)]`, latches after the first point, and took 12 of 14 009 samples (0.09%).

## 6. Commands, verbatim

All from `/Users/ncsmith/src/generators/vibegraph`; `OD=$(xcrun -f llvm-objdump)`.

**1 — verify checkout**
```
$ git rev-parse --show-toplevel
/Users/ncsmith/src/generators/vibegraph
$ git log --oneline -1
9bad54c Merge perf-baseline-timing: per-stage timing baseline vs MadGraph
```

**2 — build** (backgrounded; `fas-build.log`)
```
$ cargo test --profile release-debug --features extended-validation --test validate_sigma --no-run
   Compiling vibegraph-lib v0.1.0 (/Users/ncsmith/src/generators/vibegraph/vibegraph-lib)
    Finished `release-debug` profile [optimized + debuginfo] target(s) in 4.13s
  Executable tests/validate_sigma.rs (target/release-debug/deps/validate_sigma-2080c22985a752c1)
BUILD EXIT: 0
```

**3 — locate the symbol**
```
$ nm -n target/release-debug/deps/validate_sigma-2080c22985a752c1 | grep -A1 fill_arenas
000000010014e050 t __ZN9vibegraph5helas4eval3run11fill_arenas17h618c857602bc932aE
0000000100150034 T __ZN9vibegraph5helas4eval3run15validate_arenas17hf13b9fc3c73a9429E
```
→ `[0x10014e050, 0x100150034)` = 8164 B = **2041 instructions**; exactly one monomorphization.

**4 — record the profile** (backgrounded; `fas-profile.log`)
```
$ samply record --save-only -o target/fill-arenas-study/integrate.json.gz \
      --unstable-presymbolicate \
      target/release-debug/deps/validate_sigma-2080c22985a752c1 \
      sigma_gate_matches_madgraph --test-threads=1
running 1 test
test sigma_gate_matches_madgraph ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 20 filtered out; finished in 41.80s
PROFILE EXIT: 0
```
(`scripts/profile.sh` was read first; it forwards only a single filter arg and cannot pass `--test-threads=1`, so `samply record` was invoked directly with the same flags. Banked MadGraph reference data used as-is; nothing regenerated.)

**5 — plain disassembly**
```
$ $OD -d --demangle --start-address=0x10014e050 --stop-address=0x100150034 \
      target/release-debug/deps/validate_sigma-2080c22985a752c1 > target/fill-arenas-study/fas-disasm-plain.txt
$ grep -cE '\bbr\s+x' target/fill-arenas-study/fas-disasm-plain.txt
1
```

**6 — source-interleaved disassembly**
```
$ $(xcrun -f dsymutil) target/release-debug/deps/validate_sigma-2080c22985a752c1 \
      -o target/fill-arenas-study/validate_sigma.dSYM          # DSYMUTIL EXIT: 0
$ $OD -d -S --demangle \
      --dsym=target/fill-arenas-study/validate_sigma.dSYM/Contents/Resources/DWARF/validate_sigma-2080c22985a752c1 \
      --start-address=0x10014e050 --stop-address=0x100150034 \
      target/release-debug/deps/validate_sigma-2080c22985a752c1 > target/fill-arenas-study/fas-disasm-src.txt
```
(2911 lines; source of every quoted listing in §1, §4.1, §4.3, re-dumped per-range with narrower bounds.)

**7 — leaf-address histogram**
```
$ gunzip -c target/fill-arenas-study/integrate.json.gz > target/fill-arenas-study/integrate.json
$ python3 target/fill-arenas-study/fas-analyze.py           # fas-analyze.log
busiest thread[1] name='sigma_gate_matches_madgraph' sampled stacks=41681
leaf samples with a stack: 41681
leaf in test binary:       34484 (82.7%)
leaf inside fill_arenas:   14009 (33.6% of thread self time)
fill_arenas anywhere in stack: 19859 (47.6%)
distinct hot addresses:    580 of 2041 instructions

top hot addresses:
  0x10014e2a8  +0x0258   2326  16.60%
  0x10014e2c4  +0x0274   1523  10.87%
  0x10014ef30  +0x0ee0    329   2.35%
  0x10014ec6c  +0x0c1c    234   1.67%
  0x10014ef4c  +0x0efc    218   1.56%
  0x10014eefc  +0x0eac    217   1.55%
  0x10014e53c  +0x04ec    211   1.51%
  0x10014ef40  +0x0ef0    207   1.48%
  ...
```

**8 — symbolize every instruction, bucket by arm**
```
$ python3 -c "
for a in range(0x10014e050, 0x100150034, 4): print(hex(a))" > target/fill-arenas-study/fas-all-addrs.txt   # 2041
$ atos -o target/fill-arenas-study/validate_sigma.dSYM/Contents/Resources/DWARF/validate_sigma-2080c22985a752c1 \
       -l 0x100000000 -i -f target/fill-arenas-study/fas-all-addrs.txt > target/fill-arenas-study/fas-atos-all.txt
$ python3 target/fill-arenas-study/fas-bucket2.py           # fas-bucket2.log
instructions: 2041  with a run.rs line: 1822  line-0 forward-filled: 219 (10.7%)
total fill_arenas leaf samples: 14009

samples       %  arm (forward-filled)   [raw / filled-in]
   2592  18.50%  LOOP CONTROL (zip iterator next)   [ 2590 / +2]
   2546  18.17%  GammaVout                          [ 2389 / +157]
   1974  14.09%  Metric                             [ 1968 / +6]
   1679  11.99%  DISPATCH match *instr              [ 1679 / +0]
   1024   7.31%  MulScalarR                         [ 1007 / +17]
    622   4.44%  MulScalarC                         [  613 / +9]
    590   4.21%  AddScalar                          [  319 / +271]
    384   2.74%  GammaFin                           [  374 / +10]
    340   2.43%  GammaFout                          [  325 / +15]
    285   2.03%  ScaleFoutC                         [   24 / +261]
    247   1.76%  FfvVout                            [  116 / +131]
    207   1.48%  PropagateVector                    [  193 / +14]
    185   1.32%  ScaleVecR                          [   81 / +104]
    181   1.29%  FfvFout                            [  175 / +6]
    169   1.21%  FfvFin                             [  166 / +3]
    137   0.98%  ExternalFin                        [  137 / +0]
    137   0.98%  ExternalFout                       [  137 / +0]
   [... 24 more arms, each < 1% ...]
still unattributed: 1080 raw line-0 samples (7.71%) redistributed by forward fill

arm                                 insns  samples       %  smp/insn
LOOP CONTROL (zip iterator next)       31     2592  18.50%      83.6
GammaVout                             139     2546  18.17%      18.3
Metric                                 56     1974  14.09%      35.2
DISPATCH match *instr                   5     1679  11.99%     335.8
MulScalarR                             32     1024   7.31%      32.0
MulScalarC                             33      622   4.44%      18.8
AddScalar                              54      590   4.21%      10.9
instructions mapped: 2041
```

**9 — structural census**
```
$ grep -E '\bbl\s+0x' fas-disasm-plain.txt | sed 's/.*bl\t//' | sort | uniq -c | sort -rn
 108 <core::panicking::panic_bounds_check>
   5 <core::slice::index::slice_index_fail>
   4 <core::panicking::panic_fmt>
   4 <vibegraph::helas::eval::run::build_external_slot>
   3 <alloc::vec::Vec<T,A>::resize>
   2 <dyld_stub_binder>
   2 <alloc::raw_vec::RawVecInner<A>::reserve::do_reserve_and_handle>
   1 <vibegraph::helas::eval::kernel::propagate_vector_bare>
   1 <vibegraph::helas::eval::kernel::propagate_fout_bare>
   1 <vibegraph::helas::eval::kernel::propagate_fin_bare>
   1 <vibegraph::helas::eval::kernel::ffv_vout_bare>

$ grep -cE 'ldr\s+x[0-9]+, \[x19' fas-disasm-plain.txt
143
$ grep -oE 'ldr\s+x[0-9]+, \[x19, #0x[0-9a-f]+\]' fas-disasm-plain.txt | grep -oE '#0x[0-9a-f]+' | sort | uniq -c | sort -rn
  18 #0x40   16 #0x28   15 #0x38   14 #0x70   14 #0x58   13 #0x20
  12 #0x68   12 #0x50    9 #0x10    8 #0x8     5 #0x88    5 #0x80   1 #0x18
$ grep -cE 'mov\s+w13, #0x14' fas-disasm-plain.txt
2
$ grep -cE 'ldp\s+q[0-9]+, q[0-9]+, \[sp' fas-disasm-plain.txt ; grep -cE 'stp\s+q[0-9]+, q[0-9]+, \[sp' fas-disasm-plain.txt
10
8
$ grep -E 'sub\s+sp, sp,' fas-disasm-plain.txt
10014e074: d10a03ff     sub sp, sp, #0x280
$ otool -l target/release-debug/deps/validate_sigma-2080c22985a752c1 | grep -A6 'segname __TEXT$'
  segname __TEXT
   vmaddr 0x0000000100000000
   fileoff 0
$ xxd -s $((0x25aef8)) -l 96 -e -g 2 target/release-debug/deps/validate_sigma-2080c22985a752c1
   [table bytes quoted in §1]
```

**10 — `Instr` variant count**
```
$ grep -rn 'enum Instr' vibegraph-lib/src/
vibegraph-lib/src/helas/eval/layout.rs:76:pub(super) enum Instr {
$ awk 'NR>=76 && /^}/{exit} NR>=77' vibegraph-lib/src/helas/eval/layout.rs | grep -cE '^\s{4}[A-Z][A-Za-z]+[ ,{]'
38
```

**11 — kind classification + inline-frame grouping**
```
$ python3 target/fill-arenas-study/fas-classify.py     # fas-classify.log — output quoted in §3
$ python3 target/fill-arenas-study/fas-bucket.py       # fas-bucket.log   — grouping quoted in §2
```

**12 — cargo-show-asm, both attempts**
```
$ cargo asm --profile release-debug --features extended-validation --test validate_sigma --rust fill_arenas
Error: Multiple packages found                                   # needs -p

$ cargo asm -p vibegraph-lib --profile release-debug --features extended-validation \
      --test validate_sigma --rust fill_arenas
    Finished `release-debug` profile [optimized + debuginfo] target(s) in 13.79s
Error: Cannot locate the path to the asm file
Artifact paths: .../target/release-debug/deps/validate_sigma-9897a11f4a43edff, (same)

$ cargo asm -p vibegraph-lib --lib --profile release-debug --features extended-validation --rust fill_arenas \
      > target/fill-arenas-study/fas-cargoasm-lib.txt            # EXIT: 0, 4689 lines
$ grep -cE '^\t[a-z][a-z0-9]*(\.[0-9]+[a-z]+)?\s' fas-cargoasm-lib.txt
2050                                                              # vs 2041 in the linked binary
$ grep -oE 'bl\s+_?[A-Za-z_].*' fas-cargoasm-lib.txt | sed 's/bl\s*//' | sort | uniq -c | sort -rn | head -12
 108 core::panicking::panic_bounds_check
   5 core::slice::index::slice_index_fail
   5 alloc::raw_vec::RawVecInner<A>::reserve::do_reserve_and_handle
   5 _bzero
   4 vibegraph::helas::eval::run::build_external_slot
   4 core::panicking::panic_fmt
   1 vibegraph::helas::eval::kernel::propagate_vector_bare
   1 vibegraph::helas::eval::kernel::propagate_fout_bare
   1 vibegraph::helas::eval::kernel::propagate_fin_bare
   1 vibegraph::helas::eval::kernel::ffv_vout_bare
```

**13 — binary integrity after the cargo-asm rebuilds**
```
$ shasum -a 256 target/release-debug/deps/validate_sigma-2080c22985a752c1
44c5c8323a8377ca924eb7685ba0011caccbea5569ec8151b2d079ff495e14c9
$ nm -n target/release-debug/deps/validate_sigma-2080c22985a752c1 | grep fill_arenas
000000010014e050 t __ZN9vibegraph5helas4eval3run11fill_arenas17h618c857602bc932aE
```

**14 — consistency check**
```
$ python3 -c "..."
sum of arm buckets     : 14009 total 14009
sum of kind buckets    : 14009 total 14009
insns classified       : 2041 expected 2041
```

## 7. Artifacts on disk

All under `/Users/ncsmith/src/generators/vibegraph/target/fill-arenas-study/`:
`integrate.json.gz` + `.syms.json` (samply profile; browse with `samply load`), `integrate.json`, `fas-disasm-plain.txt` (2047 lines), `fas-disasm-src.txt` (2911 lines, source-interleaved), `fas-addr-hist.txt` (580 rows: `vaddr / +offset / samples / %`), `fas-atos-all.txt` (inline chains for all 2041 instructions), `fas-arm-detail2.txt` (per-arm per-address with FILL markers), `fas-analyze.py`, `fas-bucket.py`, `fas-bucket2.py`, `fas-classify.py` + their `.log`s, `fas-cargoasm-lib.txt`, `fas-hist.json` / `fas-buckets*.json` / `fas-kinds.json`, `fas-build.log` / `fas-profile.log` / `fas-dsymutil.log`, `validate_sigma.dSYM/`.
