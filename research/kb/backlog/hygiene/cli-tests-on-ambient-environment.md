---
type: Backlog Item
title: Seven CLI test binaries still run the binary in the developer's ambient environment
description: "The extended-validation CLI tests and cli_generate invoke vibegraph through common::vibegraph_ambient, so an ambient VIBEGRAPH_PDF_DIR, HOME or network setting can change what they read."
area: hygiene
state: open
priority: low
closes_when: "cli_integrate, cli_decay_chain, cli_decay_chain_events, cli_generate_proton, cli_reweight_proton, validate_samples_proton and cli_generate run through the isolated common::vibegraph(dir, home) (or a variant keeping only what each needs), checked by running them with their data present."
blocked_by: []
opened: 2026-10-21
tags: [tests, cli, isolation]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-21}
sources:
  - {id: f3, resource: "../../sprints/cut-refusal/sessions/F3-report.md", title: "cut-refusal F3 report, Found"}
---
Since the [cut-refusal sprint](../../sprints/cut-refusal/closeout.md) every CLI
test file goes through `vibegraph-cli/tests/common`. The six
`extended-validation` binaries were moved onto `vibegraph_ambient()` rather
than the isolated helper because their data was absent where the change was
made, so isolation could not be run; `cli_generate` stayed ambient to keep
its edit minimal while another PR changes it.
