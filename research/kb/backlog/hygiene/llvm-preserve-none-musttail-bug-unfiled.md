---
type: Backlog Item
title: An LLVM preserve_none tail-call miscompile is not reported upstream
description: x86-64 preserve_none with an indirect musttail and 12 arguments miscompiles silently in rustc; a 25-line reproducer exists but no issue is filed.
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
(`movq (%r10), %rax; jmpq *%rax`), so the callee receives its own address as
its last argument. LLVM reports "ran out of registers"; rustc drops that into
a silent miscompile (garbage at `-O`, segfault at `-O0`; correct with ≤ 11
arguments or a plain call). The reproducers are under the
`study/threaded-dispatch` tag (`4657c92`, on origin; not fetched in every
clone). Filing an upstream issue publicly is the user's call. Detail:
[threaded-dispatch study](../../history/notes/threaded-dispatch-study-results.md).
