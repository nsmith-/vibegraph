---
type: Backlog Item
title: "The library's generate accepts nevents = 0"
description: "Only the CLI refuses zero events; vibegraph::generate::generate given nevents = 0 runs the full scan and writes an empty file."
area: hygiene
state: open
priority: low
closes_when: "vibegraph::generate::generate refuses nevents = 0 with a GenerateError variant and a test, and the CLI keeps its own wording through cli_error."
blocked_by: []
opened: 2026-10-26
tags: [generate, refusal, library-api]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-26}
sources:
  - {id: hy1, resource: "../../sprints/gridpack-generate/sessions/HY1-report.md", title: "gridpack-generate HY1 report, Found 2"}
---
`vibegraph-cli/src/generate.rs` refuses `settings.nevents == 0` (near :365) with
a message that distinguishes `--nevents 0` from a recorded `nevents = 0`. The
library entry point (`vibegraph-lib/src/generate.rs`, `pub fn generate`) checks
the rounding refusals but not the count, so a library caller gets a full scan
and an empty file ([HY1](../../sprints/gridpack-generate/sessions/HY1-report.md)).
The rounding refusals already show the pattern: a library variant, mapped back
to CLI wording by `cli_error`.
