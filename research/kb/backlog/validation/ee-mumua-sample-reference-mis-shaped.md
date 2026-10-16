---
type: Backlog Item
title: "ee_to_mumua's MadEvent sample is mis-shaped in pt(γ)"
description: "MadEvent's unweighted ee_to_mumua sample puts 8.73% of σ in pt(γ) ∈ [10, 20) against 9.40% from its own windowed runs; the samples cell measures that defect, so it stays info."
area: validation
state: open
priority: low
closes_when: "ee_to_mumua's samples cell compares against a sample reference shown to have the right pt(γ) shape (a non-MadEvent reference, or MadEvent at a budget and survey setting that fixes it), and the cell is gated."
blocked_by: []
opened: 2026-10-16
tags: [ee-to-mumua, samples, madevent-reference]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-16}
---
The σ side of the row is attributed and gated (`sprints/sigma-ee-mumua`). The
unweighted sample is a separate defect: relative to the true σ it is
mis-shaped by about 7.5% in `pt(γ) ∈ [10, 20)`, in MadGraph 3.5.7 and 3.7.1
alike, so a `samples` gate against it measures MadEvent
([MadGraph reference runs](../../validation/madgraph-reference-runs.md)). Whether
`hard_survey = 2`, which fixed the σ, also fixes the sample shape is
untested.
