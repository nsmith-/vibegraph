# Draft: LLVM issue

For the user to review, edit and post on llvm/llvm-project. Each claim below
was re-run on 2026-10-16. LLVM's AI tool policy
(<https://llvm.org/docs/AIToolPolicy.html>) requires the poster to have read
and checked the report and to be able to answer questions on it. Keep or
reword the `Assisted-by` line at the end; it is there to disclose the tool use.
Suggested labels: `backend:X86`, `llvm:regalloc`.

---

**Title:** [X86] preserve_none indirect musttail with 12 integer arguments: "ran out of registers", target loaded into an argument register

`preserve_none` on x86-64 passes integer arguments in
`r12 r13 r14 r15 rdi rsi rdx rcx r8 r9 r11 rax`. When all 12 are used by an
indirect `musttail` call, no register is left for the call target. `llc`
reports an error but still writes output, and that output loads the target
into `rax`, which overwrites the 12th argument.

```llvm
target triple = "x86_64-unknown-linux-gnu"

define preserve_nonecc i64 @f(ptr %fp, i64 %a, i64 %b, i64 %c, i64 %d, i64 %e, i64 %f, i64 %g, i64 %h, i64 %i, i64 %j, i64 %k) {
  %t = load ptr, ptr %fp
  %r = musttail call preserve_nonecc i64 %t(ptr %fp, i64 %a, i64 %b, i64 %c, i64 %d, i64 %e, i64 %f, i64 %g, i64 %h, i64 %i, i64 %j, i64 %k)
  ret i64 %r
}
```

`llc -O2 repro.ll -o -` exits with status 1:

```
error: <unknown>:0:0: ran out of registers during register allocation in function 'f'
```

and emits:

```
f:
        movq    (%r12), %rax
        jmpq    *%rax           # TAILCALL
```

**Expected:** either valid code, or a diagnostic that names the cause (no
register left for the indirect tail-call target) rather than a generic
register-allocation failure. `r10` and `rbx` are free at the call, and neither
is callee-saved under `preserve_none` (`CSR_64_NoneRegs` is only `rbp`).

**Reproduced with:**
- `llc` trunk, with and without assertions, on Compiler Explorer
  (2026-10-16). There is no assertion failure, only the error above.
- LLVM 23.1.4, from Rust's `llvm-tools`. Same result for
  `-mtriple=x86_64-pc-windows-msvc` and `x86_64-apple-darwin`.
- 22.1.0 and 19.1.0 on Compiler Explorer, at `-O0` and `-O2`, with the
  default, `pic` and `static` relocation models.

**Controls:**
- Drop `%k`, so 11 integer arguments: exit 0, and the target goes into the
  now-free `rax`.
- Make the call direct (`musttail call ... @g(...)`): exit 0, `jmp g@PLT`.
- Call `%fp` itself instead of the loaded `%t`: the same error. The load is not
  the cause.

**Analysis.** `-stop-after=finalize-isel` shows that the target vreg has class
`gr64_tc`, the operand of `TCRETURNri64` (`ptr_rc_tailcall`):

```
%12:gr64_tc = MOV64rm %0, 1, $noreg, 0, $noreg :: (load (s64) from %ir.fp)
TCRETURNri64 killed %12, 0, csr_64_noneregs, implicit $rsp, ..., implicit $r11, implicit $rax
```

`GR64_TC` (`X86RegisterInfo.td`) is `RAX RCX RDX RSI RDI R8 R9 R11` plus
`RIP RSP`. All eight allocatable members are `preserve_none` argument
registers, so with 12 arguments the class has no free register.
`RegAllocBase::getErrorAssignment` then emits `DiagnosticInfoRegAllocFailure`
and assigns `AllocOrder.front()`, which is `RAX`. The tail-call patterns
already choose the class by calling convention: `TCRETURN_WIN64ri` uses
`GR64_TCW64`, which includes `R10`, and HiPE on 32-bit uses `GR32`. Nothing
does the same for `preserve_none`. A possible fix is a `preserve_none` variant
of the class that adds `R10` (and perhaps `RBX`). That variant would need to
account for `nest` (`R10`) and the base pointer (`RBX`), and would have to
stay in sync with `X86DAGToDAGISel::checkTCRetEnoughRegs`. If that is not
wanted, an earlier, specific diagnostic would still be better than the
current output.

Downstream effect: rustc ignores this diagnostic kind, so a Rust program using
`extern "rust-preserve-none"` with `become` compiles with no error and runs
wrongly (rust-lang/rust#146393).

Assisted-by: claude-code:claude-opus-5-5
