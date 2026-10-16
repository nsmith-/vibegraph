---
type: Backlog Item
title: An LLVM preserve_none tail-call miscompile is not reported upstream
description: x86-64 preserve_none with an indirect musttail and 12 arguments miscompiles silently in rustc; standalone reproducers exist but no issue is filed.
area: hygiene
state: needs-user
priority: low
closes_when: An LLVM (and, if applicable, rustc) issue with the reproducer is filed and linked from the threaded-dispatch study note.
blocked_by: []
opened: 2026-09-25
tags: [llvm, rustc, upstream, threaded-dispatch]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-06}
sources:
  - {id: todo, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L731-L762", title: "TODO.md entry T060"}
---
On x86-64, `preserve_none` passes 12 arguments in registers with `rax` last.
With all 12 in use, LLVM loads an indirect tail call's target into `rax`
(`movq (%r12), %rax; jmpq *%rax`), so the callee receives its own address as
its last argument. LLVM reports "ran out of registers"; rustc drops that into
a silent miscompile (garbage at `-O`, segfault at `-O0`; correct with ≤ 11
arguments or a plain call). Standalone LLVM IR and rustc reproducers, re-run on
LLVM 23.1.4 and rustc nightly 2026-10-15, are in
[`research/upstream/llvm-preserve-none-tail-call/`](../../../upstream/llvm-preserve-none-tail-call/README.md).
The root cause is two bugs. In LLVM, the x86 tail-call register class
`GR64_TC` has no register left under `preserve_none`. In rustc, LLVM 20's
`DK_RegAllocFailure` diagnostic is dropped, which is also the cause of
rust-lang/rust#146393. Both are in the README. The LLVM side is filed as
llvm/llvm-project#232957 (2026-10-16). What remains is the user-written comment
on #146393, which can draw on the evidence sheet
(`rustc-146393-evidence.md`; Rust's LLM policy rules out a drafted comment).
After that, link both from the threaded-dispatch study. Detail:
[threaded-dispatch study](../../history/notes/threaded-dispatch-study-results.md).
