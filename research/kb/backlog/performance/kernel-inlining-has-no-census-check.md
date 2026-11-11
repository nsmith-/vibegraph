---
type: Backlog Item
title: "Nothing catches LTO moving an evaluator kernel out of line"
description: "Unrelated code tipped two unhinted *_bare kernels out of line under fat LTO, costing about 2.5% on every |M|^2 evaluation, and nothing in CI noticed."
area: performance
state: open
priority: medium
closes_when: "A CI or test check counts out-of-line kernel::*_bare symbols in a release binary (nm) against a pinned census and fails when one appears, and the census is documented in evaluator-program-layout.md."
blocked_by: []
opened: 2026-11-10
tags: [performance, evaluator, lto, ci]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-10}
sources:
  - {id: pr124, resource: "https://github.com/nsmith-/vibegraph/pull/124", title: "Configuration shares, with the share-accounting performance commits (found there)"}
---
Adding the share code (nsmith-/vibegraph#124) tipped fat LTO into calling
`off_shell_fin_bare` and `off_shell_fout_bare` out of line: they were the
only `*_bare` kernels without the soft `#[inline]` the others carry, and the
base build had inlined them by luck. That cost about 2.5% on every |M|²
evaluation, with or without shares, and was found only by profiling.
`#[inline]` restored them (`nm` census: base 0, shares 2, fixed 0). The
[program layout](../../performance/evaluator-program-layout.md) concept
already recommends diffing the out-of-line call census; nothing enforces it.
