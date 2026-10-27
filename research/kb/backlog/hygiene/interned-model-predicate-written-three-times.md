---
type: Backlog Item
title: "Whether a model import is the interned SM is decided in three places"
description: "config.rs (load_ufo_with_identity), the CLI's assets.rs (resolve_ufo_search_path) and generate.rs (model_location) each test import.name == \"sm\", and model_location rebuilds the search path."
area: hygiene
state: open
priority: low
closes_when: "One predicate on ModelImport (or the config returning where it read the model) is used by all three sites."
blocked_by: []
opened: 2026-10-26
tags: [model-identity, duplication, cli]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-26}
sources:
  - {id: hy1, resource: "../../sprints/gridpack-generate/sessions/HY1-report.md", title: "gridpack-generate HY1 report, Found 3"}
---
Sites on `main`: `vibegraph-lib/src/config.rs` (`if import.name == "sm"`, near
:101), `vibegraph-cli/src/assets.rs:157` and `vibegraph-cli/src/generate.rs:284`
(`model_location`, which also rebuilds `config.rs`'s
`ufo_search_path.join(name)`). A fourth spelling would come with the next
interned model ([HY1](../../sprints/gridpack-generate/sessions/HY1-report.md)).
