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
emits wrong code with no error.

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
