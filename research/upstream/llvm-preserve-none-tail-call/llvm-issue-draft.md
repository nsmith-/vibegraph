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

**Expected:** valid code, which exists. `r10` and `rbx` are free at the call,
and neither is callee-saved under `preserve_none` (`CSR_64_NoneRegs` is only
`rbp`). A memory-indirect jump needs no register at all. I linked each of the
following hand-written bodies of `f` against a `preserve_nonecc` caller and
callee compiled by `llc`, with `@slot` holding the callee and arguments
`1..11`:

| body of `f` | result |
|---|---|
| `movq (%r12), %rax; jmpq *%rax` (llc output) | garbage |
| `movq (%r12), %r10; jmpq *%r10` | 66 (correct) |
| `movq (%r12), %rbx; jmpq *%rbx` | 66 (correct) |
| `jmpq *(%r12)` | 66 (correct) |

<details><summary>Harness</summary>

`driver.ll`, built with `llc -O2 -relocation-model=pic -filetype=obj driver.ll`:

```llvm
target triple = "x86_64-unknown-linux-gnu"
declare preserve_nonecc i64 @f(ptr, i64, i64, i64, i64, i64, i64, i64, i64, i64, i64, i64)
define preserve_nonecc i64 @callee(ptr %fp, i64 %a, i64 %b, i64 %c, i64 %d, i64 %e, i64 %f, i64 %g, i64 %h, i64 %i, i64 %j, i64 %k) {
  %s1 = add i64 %a, %b
  %s2 = add i64 %s1, %c
  %s3 = add i64 %s2, %d
  %s4 = add i64 %s3, %e
  %s5 = add i64 %s4, %f
  %s6 = add i64 %s5, %g
  %s7 = add i64 %s6, %h
  %s8 = add i64 %s7, %i
  %s9 = add i64 %s8, %j
  %s10 = add i64 %s9, %k
  ret i64 %s10
}
@slot = global ptr @callee
define i64 @run() {
  %r = call preserve_nonecc i64 @f(ptr @slot, i64 1, i64 2, i64 3, i64 4, i64 5, i64 6, i64 7, i64 8, i64 9, i64 10, i64 11)
  ret i64 %r
}
```

`main.c`: `long run(void); int main(void) { return run() != 66; }`, linked with
the object above and a `.s` file defining `f` with each body in turn.
</details>

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
and assigns `AllocOrder.front()`, which is `RAX`.

From reading the source, I see two possible fixes; neither is tested:

- **Fold the load into the jump (`TCRETURNmi64`, `jmpq *(%r12)`).**
  `X86DAGToDAGISel::checkTCRetEnoughRegs` rejects this fold here. The check
  counts every GPR argument register against the eight `GR64_TC` registers,
  including `r12`–`r15`, which are not in `GR64_TC` and do not reduce its free
  registers. The address uses `%r12`, an argument that stays live through the
  jump, so the folded load needs no scratch register. A count limited to
  argument registers inside `GR64_TC`, together with base/index registers
  that are already arguments, would accept it.
- **Choose the class by calling convention.** The tail-call patterns already
  do this elsewhere: `TCRETURN_WIN64ri` uses `GR64_TCW64`, which includes
  `R10`, and HiPE on 32-bit uses `GR32`. A `preserve_none` variant that adds
  `R10` (and perhaps `RBX`) would cover cases the fold cannot, such as a target
  computed in a register. That variant has to exclude `R10` when there is a
  `nest` argument and `RBX` when it is the base pointer, and it has to stay in
  sync with `checkTCRetEnoughRegs`.

Even where no register is free, a specific diagnostic would be better than a
generic regalloc error followed by wrong code.

Downstream effect: rustc ignores this diagnostic kind, so a Rust program using
`extern "rust-preserve-none"` with `become` compiles with no error and runs
wrongly (rust-lang/rust#146393).

Assisted-by: claude-code:claude-opus-5-5
