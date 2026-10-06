---
type: Backlog Item
title: σ calibration comments written before e73b158 no longer reproduce
description: Seed-sweep figures recorded in validate_sigma.rs before e73b158 drifted when the note-34 draw commits moved the sampling streams, and were never re-recorded.
area: validation
state: open
priority: medium
closes_when: Every recorded seed calibration in validate_sigma.rs reproduces at the current commit when the same seeds are rerun at the same budget, and the attribution falsifier has been run once.
blocked_by: []
opened: 2026-09-07
tags: [vegas, seed-sweep, calibration, gate-comments]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-06}
sources:
  - {id: todo, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L414-L442", title: "TODO.md entry T029"}
---
The seed-headroom census reran recorded calibrations with the same seeds, the
same budgets and the same helper. The result splits cleanly on when each comment
was written:
- **Written in `e73b158`:** all 13 reproduce to the digit.
- **Written in `5cc41de` / `35ab3f1`:** none reproduce.

Rows that drifted, recorded → measured worst `|rel|`:

| row | recorded | measured |
|---|---|---|
| `gg_to_gg` | 1.4e-3 | 1.972e-3 |
| `gg_to_ttx` | 8.6e-4 | 6.683e-4 |
| `uux_to_uux` | 1.1e-3 | 1.042e-3 |
| `uux_to_epemg` | 3.9e-3 | 4.164e-3 |
| `gu_to_epemu` | 1.35e-3 | 1.841e-3 |
| `ud_to_epemud_qcd0` | 2.6e-3 | 3.954e-3 |
| `ee_to_mumu_tata_qcd0` | 4.5e-3 | 6.716e-3 |

`ee_to_mumua`'s gate-seed pull also moved, 2.83 → 3.56. The ℓℓj rows were
corrected in the census commit. The rest still stand stale; for example,
`validate_sigma.rs:329`, `:433` and `:457` quote the old figures.

The drift is attributed to the note-34 draw-performance commits (`f85718d`,
`c48fc69`, `f3d6e8b`), which move the sampling streams. The falsifier for that
attribution is one run of `probe_gate_row_seed_headroom` at `f85718d^`.

The fix is a mechanical re-recording session.

Detail: [note 36a §1c](../../36a-seed-headroom-census.md).
