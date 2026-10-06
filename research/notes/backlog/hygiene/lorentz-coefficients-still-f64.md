---
type: Backlog Item
title: Lorentz and fermion-sign coefficients are still f64 leaves
description: Op::Coeff carries f64 Lorentz-structure and fermion-sign coefficients although an exact Op::CoeffRat exists for colour.
area: hygiene
state: open
priority: low
closes_when: The remaining Coeff(f64) leaves are exact rationals (CoeffRat) or a recorded decision keeps them as f64.
blocked_by: []
opened: 2026-07-12
tags: [evaluator, ast, rational, cleanup]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-06}
sources:
  - {id: todo, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L1374-L1376", title: "TODO.md entry T121"}
---
`Op::CoeffRat` (`± i^{imag}·num/den`) carries exact colour coefficients, but
Lorentz-structure and fermion-sign coefficients still use `Coeff(f64)`
(`vibegraph-lib/src/helas/eval/op.rs:272`). Migrating them is optional cleanup;
no consumer is blocked.

Detail: [note 16 §5](../../16-color-flow-design.md).
