---
type: Backlog Item
title: MLM dump gates write no collator row
description: validate_mlm_dumps enforces four samples cells flipped to gate but writes no row, so the collator renders them ⏳; no per-event samples row kind exists.
area: validation
state: open
priority: medium
closes_when: The report schema has a per-event samples row kind, validate_mlm_dumps writes it, and the four MLM samples cells render as measured in the collated report.
blocked_by: []
opened: 2026-10-03
tags: [mlm, collator, report-schema, samples]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-06}
sources:
  - {id: todo, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L203-L205", title: "TODO.md entry T001"}
---
The samples cells of `pp_to_llj_mlm`, `pp_to_llj_mlm_alps2`,
`pp_to_ll_0j2j_mlm` and `pp_to_ttx_0j1j_mlm` are `long`/`gate` in
`validation/manifest.toml`, enforced event by event by
`vibegraph-lib/tests/validate_mlm_dumps.rs` (`pixi run validate-mlm-dumps`,
3/3 on 2026-10-03). That test imports nothing from the report module, so it
writes no row and the four cells render ⏳ (the collator read 208 measured:
200 ✅, 8 ⚠️, 8 ⏳, 40 uncovered).

`SamplesRow` (`vibegraph-lib/tests/common/report.rs:517`) carries KS/χ²
distribution cells only. A per-event kind needs at least: events compared,
events agreeing per field, the tolerance, and the count reported as info
(permuted `P1` on the mixed rows).

Detail: [note 41 §4 Z2](../../41-mlm-feature-sprint-plan.md) ("Flips").
