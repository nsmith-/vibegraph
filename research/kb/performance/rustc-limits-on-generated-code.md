---
type: Caveat
title: "rustc and LLVM limits on huge generated functions"
description: "A 36k-call generic function OOMs rustc's MIR ReferencePropagation at 13.9 GiB and a 36k-statement one takes 52 min in LLVM's scheduler; generated Rust must be chunked."
status: draft
tags: [performance, code-generation, rustc, llvm, compile-time]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
measured:
  - {commit: 03c31e6, host: "Intel Xeon Emerald Rapids (family 6 model 207), 4-vCPU Firecracker VM, 15 GiB", command: "cargo bench --no-run --bench aot_kernels (aot-study-large)"}
  - {commit: b504391, host: "Intel Xeon Emerald Rapids (family 6 model 207), 4-vCPU Firecracker VM, 15 GiB", command: "cargo bench --no-run --bench aot_kernels (aot-mg-study), per-rustc RUSTC_WRAPPER"}
sources:
  - {id: aot6, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/aot-kernels-study-results.md#L227-L251", title: "AOT study §6, compile cost"}
  - {id: aotm4, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/aot-kernels-study-results.md#L403-L432", title: "AOT study §M4, the 2→6 in one function does not compile"}
  - {id: aotm6, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/aot-kernels-study-results.md#L494-L549", title: "AOT study §M6–§M7, compile cost side by side and reading"}
---

# rustc and LLVM limits on huge generated functions

Any code generator that emits Rust for the large processes must split its output into
functions of a few thousand statements. One function per process fails on the 2→6
(`uux_to_ccx_emmm_qcd0`, about 36 500 VM instructions), at two different stages of the
compiler depending on the form of the code.[^aot6][^aotm4]

| rendering of the 2→6 | where it fails | cost |
|---|---|---|
| by value, one generic function, ~16 k kernel calls | LLVM pre-RA `MachineScheduler` | 3 135 s (52 min) build, 3.7 GiB peak, fat LTO |
| by reference (MadGraph style), one generic function, 36 476 calls, 2.6 MB of source | rustc MIR `optimized_mir` → `ReferencePropagation` | library crate OOM-killed at 13.9 GiB RSS on a 15 GiB host, about 2 min in, twice |
| by reference, 19–20 functions of ≤ 2 000 instructions | builds | library crate 53.6 s at 1.94 GiB; bench crate 190 s at 1.15 GiB (thin LTO, cold, 2 jobs) |
| by value, chunks of 2 000 (20 functions) / 500 (75 functions) | builds | 860 s / 641 s wall, 3.9 / 3.0 GiB (fat LTO) |

**The LLVM failure.** Three `gdb` backtraces 38 minutes into the monolithic by-value
build were all in `MachineScheduler` → `ScheduleDAGMI::moveInstruction` →
`SlotIndexes::insertMachineInstrInMaps`: the pre-RA scheduler moving instructions inside
one basic block of about a million, which renumbers slot indices on every move. Chunking
shrinks the block, and the build time follows.

**The rustc failure** happens before any LLVM work. `-Z time-passes` (via
`RUSTC_BOOTSTRAP=1`) and backtraces at 3.5–10.5 GiB placed it:

| phase | time | RSS after |
|---|--:|--:|
| `type_check_crate` | 22–30 s | 1.7 GiB |
| `MIR_borrow_checking` | 30–48 s | 2.8–3.4 GiB |
| `optimized_mir` → `ReferencePropagation` → `MaybeStorageDead::iterate_to_fixpoint` | — | > 10.5 GiB, killed |

`optimized_mir` runs for a generic function whenever its crate's metadata is encoded, so
the library crate fails, and fat versus thin LTO does not enter. `cargo check` of the
same crate, which skips optimised MIR, completes in 53 s at 3.4 GiB. Why this pass blows
up is inferred, not measured: its storage-liveness dataflow keeps a bitset of locals per
basic block, every call ends a block and every `&a.v[3]` operand is a fresh MIR
temporary, so memory grows with the product of the two. Why the by-value rendering got
through the same pass is not established; it has fewer calls and its operands are named
locals rather than fresh references.

**Chunking is free at run time in the slot-array form**: nothing crosses a function
boundary except through the slot arrays. In a form that passes values in registers it is
not free. Op-blocked order puts most readers one ASAP level later, usually in a later
chunk, so almost every value crosses a boundary (about 31 000 stores and 33 500 loads per
event at 2 000 instructions per chunk), and the chunked by-value code ran no faster than
the monolithic one.

**Compile cost of what does build.** The three small rows (`ee_to_mumu`, `gg_to_gg`,
`ee_to_mumu_tata_qcd0`) add nothing measurable to a 78–80 s build. The chunked
MadGraph-style 2→6 adds about 28 s and 0.9 GiB to the library crate and 140 s to the bench
crate. Its fat-LTO cost was not measured: both container restarts on this host happened
during fat-LTO builds containing it, and the study moved to a thin-LTO profile.[^aotm6]

Caveats: one host, a shared 4-vCPU VM with 15 GiB of memory; a 16 GiB limit is what
"does not compile" means here. The study code is not in the tree. It lives at the two
commits in `measured`, and its large builds take 11–52 minutes, which is why it was
removed. The run-time results of the same renderings are in
[ahead-of-time compilation](aot-compilation-study.md); the trace-form study's generated
closed forms, if ever emitted as Rust, fall under the same limit (see
[trace-form |M|²](trace-form-msq-feasibility.md)).

[^aot6]: AOT study §6 at `03c31e6`.
[^aotm4]: AOT study §M4 at `b504391`.
[^aotm6]: AOT study §M6–§M7 at `b504391`.
