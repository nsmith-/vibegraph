---
type: Backlog Item
title: generate reads the artifact before the run card, so run-card refusals come second
description: "generate loads and parses the integrate artifact before loading the run card, so a refusable run card is reported only after the artifact read; integrate refuses it first."
area: hygiene
state: open
priority: low
closes_when: "generate loads and checks the run card before reading the artifact (or says why it cannot), so a run-card refusal comes first in both commands, by a test."
blocked_by: []
opened: 2026-10-20
tags: [cli, refusal, ordering]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-20}
sources:
  - {id: f1, resource: "../../sprints/cut-refusal/sessions/F1-report.md", title: "cut-refusal F1 report, Found 2"}
---
`vibegraph-cli/src/generate.rs` (~:773) reads the artifact, then the run card.
The artifact read is cheap, so this is only uniformity: a bad run card should
be the first refusal in both `integrate` and `generate`. The PDF-from-run-card
design changes what `generate` reads from the run card versus the artifact;
settle the order there or after it lands.
Found by the [cut-refusal F1 session](../../sprints/cut-refusal/sessions/F1-report.md).
