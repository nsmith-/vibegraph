---
type: Backlog Item
title: pp_to_llj_xqcut_only has no samples comparison
description: The manifest targets validate_samples_proton against pp_to_llj_xqcut_only's banked MadEvent run, but the comparison is not built and the cell is uncovered.
area: validation
state: open
priority: medium
closes_when: validate_samples_proton compares a vibegraph sample of pp_to_llj_xqcut_only against output/pp_to_llj_xqcut_only/Events/run_01 and the manifest cell records the measurement.
blocked_by: []
opened: 2026-10-01
tags: [mlm, samples, coverage, llj]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-06}
sources:
  - {id: todo, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L268-L269", title: "TODO.md entry T010"}
---
In `validation/manifest.toml`, `pp_to_llj_xqcut_only`'s samples cell is
`uncovered`, with the note "target validate_samples_proton against
output/pp_to_llj_xqcut_only/Events/run_01". `vibegraph-cli/tests/validate_samples_proton.rs`
has no row for it.

The row's integrals cell gates (+0.03 %, pull +0.23 on 2026-10-03), and
`validate_mlm_dumps` reproduces all 10000 dumped events. Nothing compares
the event sample by distribution.

Detail: [note 41 §4](../../history/notes/41-mlm-feature-sprint-plan.md) (Z.5).
