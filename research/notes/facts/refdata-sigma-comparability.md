---
type: Caveat
title: Partonic σ is not comparable across some refdata boundaries
description: "A partonic σ from refdata-2 is not comparable to refdata-3 and later (αs 0.130 vs 0.118), nor are the four re-carded runs across refdata-4 → refdata-5."
tags: [validation, refdata, madgraph]
status: draft
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-06}
sources:
  - {id: todo, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L68-L73", title: "TODO.md standing measurement facts"}
---

- A partonic σ quoted from `refdata-2` is **not comparable** to one from
  `refdata-3` or later. MadGraph 3.5.7 applied the PDF set's
  `αs(M_Z) = 0.130` to `lpp = 0` runs; 3.7.1 keeps the model's `0.118`.
- The four re-carded runs' σ are not comparable across the
  `refdata-4` → `refdata-5` boundary: their densities differ, and
  `p p > b b~` moves −9.8%.

Compare a σ only against a reference from the same side of these boundaries.
