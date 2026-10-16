# x86-64 `preserve_none`: indirect tail call clobbers the 12th argument

Standalone reproducers for an upstream report (LLVM, and rustc for the silent
miscompile). They need nothing from vibegraph.

On x86-64, `preserve_none` passes integer arguments in
`r12 r13 r14 r15 rdi rsi rdx rcx r8 r9 r11 rax`. When all 12 are in use, an
indirect `musttail` call has no free register from the tail-call class for its
target. LLVM reports "ran out of registers during register allocation" but
still emits code that loads the target into `rax`, so the callee receives its
own address as argument 12. With 11 arguments, or with a plain call, the code
is correct.

## LLVM (`repro.ll`)

```
llc -O2 repro.ll -o -
```

LLVM 23.1.4 (rust nightly 2026-10-15 `llvm-tools`) exits 1 with
`error: <unknown>:0:0: ran out of registers during register allocation in function 'f'`
and emits:

```
f:
        movq    (%r12), %rax
        jmpq    *%rax           # TAILCALL
```

LLVM 18 does not parse `preserve_nonecc`. On Compiler Explorer (2026-10-16),
`llc` trunk (with and without assertions), 22.1.0 and 19.1.0 all fail the same
way at `-O0` and `-O2`, under the default, `pic` and `static` relocation
models; none crashes. With the 12th argument removed, `llc` exits 0 and puts
the target in the now-free `rax`. `r10` and `rbx` are neither argument nor
callee-saved registers under `preserve_none`, yet are not used for the target.

The closest rustc report is rust-lang/rust#146393 (RISC-V with all but one
register reserved): there too `llc` reports "ran out of registers" and rustc
emits wrong code with no error. Its cause is the same rustc bug (below).

## rustc (`repro.rs`)

```
rustc +nightly --edition 2024 -C opt-level=3 repro.rs && ./repro   # garbage, assert fails
rustc +nightly --edition 2024 -C opt-level=0 repro.rs && ./repro   # SIGSEGV
```

rustc 1.101.0-nightly (4ebd25825 2026-10-15) on x86_64-unknown-linux-gnu
compiles this with exit status 0 and no diagnostic: the LLVM error is dropped.
The `-O3` handler is:

```
        movq    (%rbx,%r10,8), %rax
        jmpq    *%rax
```

Controls, same toolchain: removing the 12th argument prints 55 (correct) at
`-O0` and `-O3`; replacing `become` with a plain call prints 66 (correct).

## Root cause

Two independent bugs combine here. The LLVM one makes the x86 code wrong. The
rustc one hides LLVM's error.

**LLVM: the tail-call target register class.** *Verified:*
`llc -O2 repro.ll -stop-after=finalize-isel` gives the target vreg class
`gr64_tc`, the `ptr_rc_tailcall` operand of `TCRETURNri64`. `GR64_TC`
(`X86RegisterInfo.td`) is `RAX RCX RDX RSI RDI R8 R9 R11 RIP RSP`, and all
eight allocatable members are `preserve_none` argument registers. With 12
arguments, `RegAllocBase::getErrorAssignment` reports
`DiagnosticInfoRegAllocFailure` and assigns `AllocOrder.front()`, which is
`RAX`. Calling `%fp` directly (no load) fails the same way. A direct
`musttail` to a symbol is fine. `x86_64-pc-windows-msvc` and
`x86_64-apple-darwin` fail the same way. *Inferred:* a fix could be a
`preserve_none` tail-call class that adds `R10` (and perhaps `RBX`), following
the existing per-convention classes (`GR64_TCW64` for Win64, `GR32` for HiPE).
That class would have to handle `nest` and the base pointer and stay in sync
with `checkTCRetEnoughRegs`. This is not small enough to sketch a patch for
here.

**rustc: unknown LLVM diagnostic kinds are dropped.** *Verified:* LLVM 20
(llvm/llvm-project#119492) made register-allocation failures report through a
new kind, `DK_RegAllocFailure`; in LLVM 19 they reached rustc as `DK_InlineAsm`.
rustc's `toRust(DiagnosticKind)` maps the new kind to `Other`.
`diagnostic_handler_callback_inner` then discards it in
`UnknownDiagnostic(..) => {}` without checking its severity. Under gdb, the
`DK_RegAllocFailure` diagnostic with severity `DS_Error` reaches rustc's
callback and is never printed, and rustc exits 0. The RISC-V reproducer from
#146393 errors on nightly-2025-02-17 (LLVM 19.1.7) and compiles silently on
nightly-2025-02-18 (LLVM 20.1.0). *Inferred:* `DK_Generic` errors from
`LLVMContext::emitError` are dropped the same way. Commands, outputs and the
gdb script: [`rustc-146393-evidence.md`](rustc-146393-evidence.md).

## Reporting upstream

- **LLVM:** a new issue. [`llvm-issue-draft.md`](llvm-issue-draft.md) is a
  draft for the user to edit and post under LLVM's AI tool policy (a human
  reviews it and can explain it; tool use is disclosed). No existing issue
  matched a search of llvm-project for `preserve_none` on 2026-10-16.
- **rustc:** a comment on rust-lang/rust#146393, not a new issue, because it
  is the same root cause. The Rust LLM policy bans LLM-originated comments, so
  [`rustc-146393-evidence.md`](rustc-146393-evidence.md) is an evidence sheet,
  not a draft. The user writes the comment.
