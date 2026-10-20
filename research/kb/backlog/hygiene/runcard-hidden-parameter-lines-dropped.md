---
type: Backlog Item
title: "Run-card lines tagged '# hidden_parameter' are silently dropped"
description: "split_line strips only '!' comments, so '<v> = <name> # hidden_parameter' keeps the comment in the name and is skipped as a placeholder; 66 of 88 CMS cards lose such lines."
area: hygiene
state: open
priority: high
closes_when: "A test card with '<v> = <name> # hidden_parameter' reads the same as without the comment, and 'False = pdfwgt # hidden_parameter' on an MLM card drops the PDF ratio in the matching reweight as MadGraph does."
blocked_by: []
opened: 2026-10-20
tags: [runcard, parser, silent-physics, madgraph-parity, cms]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-20}
sources:
  - {id: survey, resource: "../../validation/cms-gridpack-survey-2026-10.md", title: "CMS gridpack survey, 88 production cards (e3f543f)"}
---
`split_line` (`vibegraph-lib/src/runcard.rs:511-525`) strips the comment at
`!` only. A MadGraph run card writes hidden fields as
`<value> = <name> # hidden_parameter`, so the name keeps ` # hidden_parameter`,
contains a space, and the line is skipped as a template placeholder. MadGraph
strips `#` first (`banner.py:2900`). Nothing warns.

Measured in the [CMS gridpack survey](../../validation/cms-gridpack-survey-2026-10.md):
`50 = mmll # hidden_parameter` integrates identically to a card with no `mmll`
line. 66 of 88 CMS cards carry at least one such line off vibegraph's default
(277 lines; `pdfwgt` 37, `deltaeta` 24, `dral`/`draj` 22, `pta` 21, ...).
`False = pdfwgt # hidden_parameter` is on 20 of the 25 MLM cards, where MadGraph
then drops the PDF ratio from the matching weight (`reweight.f:1496`) and
vibegraph keeps it.

The same function also splits at the first `=` rather than the last:
[runcard-value-with-equals-dropped](runcard-value-with-equals-dropped.md).
