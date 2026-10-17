# Evidence for a comment on rust-lang/rust#146393

**This is not a draft comment, and it should not be posted as one.** The Rust
project's LLM policy (<https://forge.rust-lang.org/policies/llm-usage.html>)
bans posting LLM-originated comments and issue bodies from a personal account.
Content that is clearly quoted and marked as LLM output is allowed only if the
comment stands on its own without it. The repository's agent rules
(`AGENTS.md` in rust-lang/rust) also forbid an agent from writing paste-ready
public comments. This file therefore gives only the facts, the commands that
show them, and what a comment would need to cover. The comment itself must be
written by the user, with a disclosure of the LLM use that the user writes
too. Bug discovery with an LLM is allowed if the bug is personally verified,
so re-run the commands below before relying on them. `repro.rs` here was also
written with LLM help.

## What #146393 already says (read 2026-10-16)

The issue is open; its labels are `A-LLVM A-codegen C-bug O-riscv P-medium
T-compiler regression-from-stable-to-stable`. On RISC-V with every GPR
reserved except x0, x2 and x10, rustc ≥ 1.87 emits wrong code where ≤ 1.86
reported an error. The window is nightly-2025-02-17 to 2025-02-18 (the LLVM 20
bump). `llc-20` on the emitted IR does report "ran out of registers". The only
comment, from a member, says it is "almost certainly an LLVM-level problem".
No root cause has been given and no PR is linked.

## Facts, each checked by experiment unless marked

1. **The cause is in rustc, not LLVM.** LLVM reports the failure. rustc
   receives the report and throws it away.
   - LLVM 20 (llvm/llvm-project#119492, "RegAlloc: Use DiagnosticInfo to
     report register allocation failures") added the diagnostic kind
     `DK_RegAllocFailure` (`DiagnosticInfoRegAllocFailure`). Before that,
     `RegAllocBase` called `LLVMContext::emitError(...)`, which in LLVM 19
     built a `DiagnosticInfoInlineAsm` (`DK_InlineAsm`). *Source:*
     `llvm/lib/CodeGen/RegAllocBase.cpp` and `llvm/lib/IR/LLVMContext.cpp` at
     `llvmorg-19.1.7`, compared with trunk.
   - rustc's `toRust(DiagnosticKind)` in
     `compiler/rustc_llvm/llvm-wrapper/RustWrapper.cpp` has no case for
     `DK_RegAllocFailure`, so it becomes `LLVMRustDiagnosticKind::Other`.
     `Diagnostic::unpack`
     (`compiler/rustc_codegen_llvm/src/llvm/diagnostic.rs`) turns that into
     `UnknownDiagnostic`. `diagnostic_handler_callback_inner`
     (`compiler/rustc_codegen_llvm/src/back/write.rs`) matches it with
     `UnknownDiagnostic(..) => {}`, so the error is dropped without regard to
     its severity. The C++ `handleDiagnostics` returns `true`, so LLVM does not
     print it either. *Source:* rust-lang/rust at 5d8fec06 (2026-10-16).
   - **Verified under gdb** with rustc 1.101.0-nightly (4ebd25825 2026-10-15),
     LLVM 23.1.4, on `repro.rs` at `-C opt-level=3`. Breakpoints on
     `llvm::LLVMContext::diagnose`, on rustc's `diagnostic_handler_callback`,
     and on `llvm::DiagnosticInfoRegAllocFailure::print`. The script is
     below. Output:
     ```
     LLVMContext::diagnose: DI kind=3 severity=0
     rustc diagnostic_handler_callback entered
     rustc exit 0
     ```
     Kind 3 is `DK_RegAllocFailure` and severity 0 is `DS_Error`. `print`, the
     function that renders the message, is never called. The backtrace at the
     diagnostic's constructor goes from `RegAllocBase::getErrorAssignment`
     through `RAGreedy::run` to `LLVMRustWriteOutputFile`, inside rustc's own
     codegen.
   - *Inferred, not tested:* since LLVM 20, `LLVMContext::emitError(Twine)`
     builds `DiagnosticInfoGeneric` (`DK_Generic`), which rustc also maps to
     `Other`. Any backend error reported that way is presumably also dropped,
     not only register-allocation failures.

2. **The window in #146393 is the LLVM 19 → 20 switch.** This is the RISC-V
   reproducer from the issue, on `riscv32i-unknown-none-elf`, with every
   `+reserve-xN` except x0, x2 and x10, at `-Copt-level=3`:

   | toolchain | LLVM | result |
   |---|---|---|
   | nightly-2025-02-17 (5bc623145) | 19.1.7 | `error: ran out of registers during register allocation`, exit 1 |
   | nightly-2025-02-18 (ce36a966c) | 20.1.0 | exit 0, no diagnostic, wrong asm |
   | nightly-2026-10-16 (4ebd25825) | 23.1.4 | exit 0, no diagnostic, wrong asm |

3. **It is not specific to RISC-V or to reserved registers.** On x86-64 with
   no target features, `repro.rs` (`extern "rust-preserve-none"` with 12
   `u64` arguments, then `become` through a function-pointer table) compiles
   with exit 0 and no diagnostic:
   - `-C opt-level=3`: prints `93853139770391 (expected 66)` and the assert
     fails. The handler is `movq (%rbx,%r10,8), %rax; jmpq *%rax`, where `rax`
     is the 12th argument register.
   - `-C opt-level=0`: SIGSEGV.
   - `rustc --emit=llvm-ir` of the same file, then the bundled `llc -O3`:
     `error: ... ran out of registers during register allocation in function
     '_RNvCs3UFjCm3vNXC_5repro4step'`, exit 1. The error exists for rustc's
     own IR.
   - Controls: with 11 arguments it prints 55, which is correct, at `-O0` and
     `-O3`. With a plain call instead of `become` it prints 66, which is
     correct.
   - The x86 trigger itself is an LLVM bug: the tail-call register class has
     no free register under `preserve_none`. That is filed separately as
     llvm/llvm-project#232957. Valid code exists, so it is a missed lowering,
     not an unavoidable failure.

4. **What a fix would need to do.** This is a direction for the user to judge
   and write up; it is not a patch. Errors of a kind rustc does not wrap
   should still be reported. That could mean an explicit arm for
   `RegAllocFailure`/`Generic`, or, in the `UnknownDiagnostic` arm, checking
   the severity (LLVM's C API has `LLVMGetDiagInfoSeverity`) and emitting
   `LLVMRustWriteDiagnosticInfoToString` as an error when it is `DS_Error`. A
   regression test could be the RISC-V reproducer as a `//@ build-fail` UI
   test, or the x86 one behind its feature gates. Rust's policy treats
   codegen as soundness-sensitive, so an LLM-written patch for this is not
   acceptable there.

## What the comment needs to cover

- The issue is rustc dropping an LLVM error, not LLVM generating bad code
  silently. Point at the `UnknownDiagnostic(..) => {}` arm and the
  `DK_RegAllocFailure` kind added in LLVM 20, and show that this matches the
  bisected window.
- The x86-64 reproducer, showing it is not RISC-V-only and needs no reserved
  registers, with a link to llvm/llvm-project#232957 for the separate x86
  bug.
- The gdb evidence, or a simpler equivalent: `--emit=llvm-ir` then `llc`
  reports the error.
- Perhaps a suggestion to retitle the issue, since it is not RISC-V-specific.
- The user's own disclosure of how an LLM was used: investigation, source
  reading and bisection, and the reproducer code.

## Commands

```sh
# x86-64
rustc +nightly --edition 2024 -C opt-level=3 repro.rs && ./repro
rustc +nightly --edition 2024 -C opt-level=3 --emit=llvm-ir repro.rs -o repro.rs.ll
"$(rustc +nightly --print sysroot)"/lib/rustlib/x86_64-unknown-linux-gnu/bin/llc -O3 repro.rs.ll -o /dev/null

# RISC-V, from #146393
cat > rv.rs <<'EOF'
#![no_std]
#[inline(never)]
pub fn foo(x: &[u32; 2]) -> u32 { x[0] + x[1] }
EOF
F=$(for i in 1 $(seq 3 9) $(seq 11 31); do printf "+reserve-x$i,"; done); F=${F%,}
for t in nightly-2025-02-17 nightly-2025-02-18; do
  rustup toolchain install $t --profile minimal -t riscv32i-unknown-none-elf
  rustc +$t --crate-type=lib -Copt-level=3 --target riscv32i-unknown-none-elf \
    -Ctarget-feature=$F --emit=asm rv.rs -o rv_$t.s; echo "$t exit $?"
done
```

gdb script (run as `gdb -q -batch -x gdb.cmd --args "$(rustc +nightly --print
sysroot)"/bin/rustc --edition 2024 -C opt-level=3 repro.rs`). The mangled
callback symbol is specific to this nightly; look it up with
`nm librustc_driver-*.so | grep diagnostic_handler_callback`:

```
set pagination off
handle SIGUSR1 nostop noprint pass
set breakpoint pending on
break llvm::LLVMContext::diagnose
commands
silent
printf "LLVMContext::diagnose: DI kind=%d severity=%d\n", *(int*)($rsi+8), *(char*)($rsi+12)
continue
end
break _RNvNtNtCs3i5n6kOFQ6I_18rustc_codegen_llvm4back5write27diagnostic_handler_callback
commands
silent
printf "rustc diagnostic_handler_callback entered\n"
continue
end
break llvm::DiagnosticInfoRegAllocFailure::print
commands
silent
printf "RegAllocFailure::print called (message rendered)\n"
continue
end
run
```
