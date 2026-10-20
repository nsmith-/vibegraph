---
type: Backlog Item
title: An unimplemented cut is refused only after diagram enumeration, under a misleading prefix
description: "An active unimplemented cut is detected at cut compilation, after full enumeration (up to 92 s), and reported as 'failed to decompose into flavour groups: cut compilation failed'."
area: hygiene
state: open
priority: low
closes_when: "A card with an active unimplemented cut is refused before diagram enumeration, with a message naming the cut and its value and no flavour-group prefix, by a test."
blocked_by: []
opened: 2026-10-20
tags: [cuts, refusal, error-messages, cli, cms]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-20}
sources:
  - {id: survey, resource: "../../validation/cms-gridpack-survey-2026-10.md", title: "CMS gridpack survey, 88 production cards (e3f543f)"}
---
`detect_unimplemented` (`vibegraph-lib/src/cuts.rs:1035`) runs inside cut
compilation, which runs inside flavour-group decomposition. The CLI wraps that
error as "failed to decompose into flavour groups: …"
(`vibegraph-cli/src/integrate.rs:836`) around `HadronicError`'s
"cut compilation failed: …" (`vibegraph-lib/src/hadronic.rs:115`). So an
`htjmin` card enumerates every diagram first (up to 92 s in the
[CMS gridpack survey](../../validation/cms-gridpack-survey-2026-10.md)) and then
reports a flavour-group failure, though the run card alone decided the outcome.
