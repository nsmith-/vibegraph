---
type: Backlog Item
title: MadEvent's unweighted samples under-populate high-weight regions
description: MadEvent's events put 5.4 % less of σ above m(ll) = 150 than its own sliced σ on fixed-beam u u~ > e+ e- g g; references built as σ × event fraction inherit it.
area: validation
state: open
priority: medium
closes_when: The deficit is located (per-channel event scaling to a biased channel σ, or elsewhere) with a run that keeps the per-channel event files, and every banked reference built from a MadEvent event fraction is listed with its exposure.
blocked_by: []
opened: 2026-10-15
tags: [madevent-reference, unweighting, event-fraction]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-15}
sources:
  - {id: v1, resource: "../../sprints/sigma-2to4-offset/sessions/V1-report.md", title: "sigma-2to4-offset V1, Found 2"}
---
MadEvent's unweighted sample puts 0.07374 ± 0.00036 of σ above m(ll) = 150,
against 0.07798 from its own sliced σ (−5.4 %, 12σ). Its final unweighting
truncates 0.13–0.43 % of σ. With the truncation off (35 runs, 90k events), the
sample is still 4σ short. The untested candidate is that each channel's events
are scaled to that channel's
[biased reported σ](../../validation/madevent-iteration-weighting-bias.md).
vibegraph's weighted events agree with its own slice. Any reference formed as
inclusive σ × event fraction under-reads high-weight regions by this much. The
`ee_to_mumua` windows are one candidate.
