---
type: Backlog Item
title: The restrict-card lookup is written three times with opposite precedence
description: "gen_standalone_jamps.py and build.sh prefer the authored card under validation/madgraph/cards/; tests/common/manifest.rs prefers the model's own card; no name collides today."
area: hygiene
state: open
priority: low
closes_when: "One rule locates a restrict card for MadGraph and Rust alike, and a check fails if a card name exists in both places."
blocked_by: []
opened: 2026-10-15
tags: [duplication, validation-scripts, ufo, restrict-card]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-15}
sources:
  - {id: ch, resource: "../../sprints/amplitude-signs/sessions/C-H-report.md", title: "C-H report, Found"}
---
Sites: `gen_standalone_jamps.py` `Row.ufo_paths`, `build.sh` `model_card_dir`
(by copy order), `tests/common/manifest.rs` `RowModel::restrict_card`. A
future collision would make the Rust and MadGraph sides compare different models.
