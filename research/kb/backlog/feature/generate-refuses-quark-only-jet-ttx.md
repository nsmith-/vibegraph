---
type: Backlog Item
title: generate refuses p p > t t~ j with a quark-only jet
description: "vibegraph integrate succeeds on p p > t t~ j with quark-only j, but generate stops: the resonance mapping in a mixed-flavour group looks up the wrong member."
area: feature
state: open
priority: medium
closes_when: vibegraph generate writes events for p p > t t~ j with quark-only j on the ttx MLM row's card, with a test that fails before the fix.
blocked_by: []
opened: 2026-10-15
tags: [generate, resonance, flavour-groups, ttx]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-15}
sources:
  - {id: v2, resource: "../../sprints/sigma-ttx-mlm-at1/sessions/V2-report.md", title: "sigma-ttx-mlm-at1 V2, Found 2"}
---
`vibegraph integrate` gives 148.64 ± 0.26 pb on the `gq` card, but
`vibegraph generate` refuses it: "the resonance 2 of g u > t t~ u has no
flavour in the subprocess with outgoing [6, -6, 1] that shares its matrix
element". The V2 report has the card and the log.
