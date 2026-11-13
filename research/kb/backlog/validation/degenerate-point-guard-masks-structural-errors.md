---
type: Backlog Item
title: "The degenerate-point guard can present a structural integrand error as 'the cuts reject every point'"
description: "Before ZeroTauFloor, a card with no ŝ floor zeroed every point as degenerate and ended in a cut-rejection refusal; other structural errors may still surface that way."
area: validation
state: open
priority: medium
closes_when: "Every path where the degenerate-point guard zeroes all points is traced, and each structural cause has its own typed refusal before integration, with a test."
blocked_by: []
opened: 2026-11-09
tags: [guards, refusal, diagnostics]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-09}
sources:
  - {id: ir2, resource: "../../sprints/v02-last-refusals/sessions/I-R2-report.md", title: "I-R2 report"}
---
Found by I-R2. On d87ad3a, `p p > b b~` at `ptb = 0` printed "8323 of 8323
integration points were zeroed as degenerate", then "the cuts reject every
point". The cause was a zero τ floor, not the cuts. Sprint I's `ZeroTauFloor`
fixes that case only.
