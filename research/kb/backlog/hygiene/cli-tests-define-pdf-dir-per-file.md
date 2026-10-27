---
type: Backlog Item
title: "Eight CLI test files define their own pdf_dir helper"
description: "fn pdf_dir is defined in eight vibegraph-cli test files (dev_pdf_dir in one) and once in vibegraph-lib's decay_chain_ladder; tests/common has PdfDataPath beside none of them."
area: hygiene
state: open
priority: low
closes_when: "One helper in vibegraph-cli/tests/common/mod.rs (beside PdfDataPath) is used by every CLI test that needs the PDF directory, and no file defines its own."
blocked_by: []
opened: 2026-10-26
tags: [tests, duplication, pdf, cli]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-26}
sources:
  - {id: hy1, resource: "../../sprints/gridpack-generate/sessions/HY1-report.md", title: "gridpack-generate HY1 report, Found 5"}
---
Files on `main` at `cfd9f7a`: `cli_generate_proton.rs`, `cli_integrate.rs`,
`cli_decay_chain.rs`, `cli_decay_chain_events.rs`, `cli_reweight_proton.rs`,
`validate_samples_proton.rs`, `cli_onshell_veto.rs`, `cli_pdf_cache_fetched.rs`
(`dev_pdf_dir`), all under `vibegraph-cli/tests/`, and
`vibegraph-lib/tests/decay_chain_ladder.rs`
([HY1](../../sprints/gridpack-generate/sessions/HY1-report.md)). The same files
are the ones [cli-tests-on-ambient-environment](cli-tests-on-ambient-environment.md)
moves to the isolated runner, so the two fit one session.
