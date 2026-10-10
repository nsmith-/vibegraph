---
type: Procedure
title: Per-address profile attribution on macOS (samply, atos, objdump)
description: "Map unsymbolicated samply samples to instructions and inlined frames with nm, a dSYM, atos -i and llvm-objdump; why cargo-show-asm cannot stand in for the linked test binary."
status: draft
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-10}]
tags: [profiling, assembly, macos, samply, performance]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: fas-limits, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/fill-arenas-asm-study-results.md#L221-L231", title: "fill_arenas instruction-level study §5: anomalies and limitations"}
  - {id: fas-commands, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/fill-arenas-asm-study-results.md#L232-L450", title: "fill_arenas instruction-level study §6: commands, verbatim"}
measured:
  commit: 9bad54c
  host: "Apple Silicon (arm64), macOS"
  command: "samply record --save-only --unstable-presymbolicate target/release-debug/deps/validate_sigma-<hash> sigma_gate_matches_madgraph --test-threads=1"
---
# Per-address profile attribution on macOS (samply, atos, objdump)

A function-level profile says *which* function is hot. For an interpreter loop
such as `fill_arenas`, where one symbol holds dozens of inlined instruction
arms, the useful question is which arm, which needs attribution per
instruction address. This is the recipe that worked on an Apple Silicon host;
its results are in
[the evaluator program layout](../performance/evaluator-program-layout.md) and
[interpreter overhead budget](../performance/interpreter-overhead-budget.md).
The function-level instruments are [profiling](profiling.md); how benchmark
numbers are to be read is
[microbenchmark protocol](../performance/microbenchmark-protocol.md).

## Recipe

All from the repository root, with `OD=$(xcrun -f llvm-objdump)`.

1. **Build the test binary once and keep its path and hash.**
   ```
   cargo test --profile release-debug --features extended-validation --test validate_sigma --no-run
   shasum -a 256 target/release-debug/deps/validate_sigma-<hash>
   ```
   Every later step reads this one binary. Re-check the hash at the end:
   anything that rebuilds the target in between (cargo-asm does) may produce a
   differently-hashed sibling, and attribution against the wrong binary is
   silently wrong.
2. **Locate the symbol's address range.**
   ```
   nm -n <bin> | grep -A1 fill_arenas
   ```
   The next symbol's address closes the range; check that there is exactly one
   monomorphization. On arm64 every instruction is 4 bytes, so the range length
   divided by 4 is the instruction count (8164 B = 2041 instructions in the
   study).
3. **Record.** `scripts/profile.sh` takes the test name and a filter, passes
   the arguments after `--` to the test binary and those after `--samply` to
   `samply record`:
   ```
   scripts/profile.sh validate_sigma sigma_gate_matches_madgraph -- --test-threads=1 \
       --samply --save-only -o target/<study>/integrate.json.gz --unstable-presymbolicate
   ```
   The script rebuilds the test (`--profile release-debug --features
   extended-validation --no-run`) before recording, which is one more reason
   for the closing hash check.
4. **Disassemble**, plain and source-interleaved. There was no
   `llvm-symbolizer` on the host (`xcrun -f llvm-symbolizer` fails), so build a
   dSYM and point objdump at its DWARF:
   ```
   $(xcrun -f dsymutil) <bin> -o target/<study>/<name>.dSYM
   $OD -d --demangle --start-address=<lo> --stop-address=<hi> <bin>
   $OD -d -S --demangle --dsym=target/<study>/<name>.dSYM/Contents/Resources/DWARF/<bin-name> \
       --start-address=<lo> --stop-address=<hi> <bin>
   ```
5. **Histogram leaf addresses.** The saved profile is **unsymbolicated**
   (`nativeSymbols` is empty; names live in the `.syms.json` sidecar). Take the
   busiest thread's leaf frames and convert them to binary addresses using the
   library-relative offsets plus the `__TEXT` `vmaddr`, verified with
   `otool -l <bin> | grep -A6 'segname __TEXT$'` (`0x100000000` here). Count
   samples per address inside `[lo, hi)`.
6. **Symbolize every instruction, with the inline chain.**
   ```
   python3 -c "for a in range(<lo>, <hi>, 4): print(hex(a))" > addrs.txt
   atos -o <name>.dSYM/Contents/Resources/DWARF/<bin-name> -l 0x100000000 -i -f addrs.txt
   ```
   `-i` prints the full inlined-frame chain, which is what maps an address to
   a source arm (`match` arm, kernel call).
7. **Bucket** samples by arm, and check that the bucket totals equal the total
   samples and the instruction count equals the range length.

The study's analysis scripts lived under `target/` and were not committed;
steps 5–7 are short Python over the profile JSON and the `atos` output.[^fas-commands]

## How far the attribution can be trusted

- **Skid.** samply samples the PC on a timer interrupt; the reported PC is
  roughly the oldest un-retired instruction, so a stall is charged to the
  instruction that waits, not the one that caused it. A loop's merge-point
  `cmp` carried 16.6% that partly belonged to the preceding arm's store and FP
  chain draining. **Block totals are robust; the split within a block is
  not.** Do not quote a single instruction's share as its cost.
- **Line-0 instructions.** About 11% of instructions had no source line. They
  were attributed to the nearest preceding resolved instruction's arm (forward
  fill), moving 7.7% of samples. Two of the largest were hand-checked against the
  interleaved disassembly and agreed; the rest were not, so treat sub-1% arm
  shares as ±0.5%.
- **One binary, one run.** On a host with performance and efficiency cores and
  no affinity set, placement can move absolute times; within-symbol shares
  should hold. Repeat the run before relying on a small difference.[^fas-limits]
- **The profile is the gate's mix.** `sigma_gate_matches_madgraph` loops over
  every banked partonic process, so arm shares are weighted by that mix; a
  single-process run shifts them.

## Why not cargo-show-asm

`cargo asm -p vibegraph-lib --profile release-debug --features
extended-validation --test validate_sigma --rust fill_arenas` fails with
"Cannot locate the path to the asm file": cargo-show-asm 0.2.62 gets bitcode,
not `.s`, for that test target (without `-p` it fails earlier with "Multiple
packages found"). `--lib` works. For `fill_arenas` in that study the `--lib`
listing matched the linked binary to 9 instructions in about 2045, with
identical dispatch code and call census. That is a measured coincidence for
one symbol, **not** licence to read cargo-asm output as the linked binary
under LTO: cross-crate inlining happens at link time. Every number should come
from the linked binary.

## Cheap structural checks on the same disassembly

Counting `bl` targets (`grep -E '\bbl\s+0x' … | sort | uniq -c`) gives the call
census, for example how many `panic_bounds_check` sites the loop carries; a
count of loads off one base register shows how often a struct field is
re-read. Both are reproducible without a profile.

[^fas-limits]: fill_arenas study §5, anomalies and limitations.
[^fas-commands]: fill_arenas study §6, commands verbatim.
