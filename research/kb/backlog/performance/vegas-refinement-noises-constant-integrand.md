---
type: Backlog Item
title: VEGAS refinement turns a constant integrand noisy
description: A single-channel two-body width is constant, yet grid refinement follows the first iteration's noise and the result reads ±0.03% instead of exact.
area: performance
state: open
priority: low
closes_when: A single-channel two-body (constant-integrand) run integrates once without grid refinement and reports the exact width, with a test pinning it.
blocked_by: []
opened: 2026-09-25
tags: [vegas, decays, width, process-grammar]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-06}
sources:
  - {id: todo, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L984-L987", title: "TODO.md entry T091"}
---
A single-channel two-body partial width has a constant integrand. VEGAS grid
refinement makes its bins follow the first iteration's sampling noise, so the
result reads ±0.03 % instead of exact. Such a run should integrate once and stop.
Nothing in `vibegraph-cli/src/integrate.rs` or `vibegraph-lib/src/vegas.rs`
skips refinement today (checked 2026-10-06).

Detail: [note 38 §4 D1](../../history/notes/38-process-grammar-sprint-plan.md) ("Findings for later").
