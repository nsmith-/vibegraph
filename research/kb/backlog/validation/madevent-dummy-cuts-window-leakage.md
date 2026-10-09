---
type: Backlog Item
title: MadEvent events violate windows its dummy_cuts should enforce
description: 27 of 500k MadEvent events in the 4e study fell outside a window its dummy_cuts should enforce; the cause is unexplained.
area: validation
state: open
priority: low
closes_when: The 27/500k window violations are explained (a MadEvent defect, a precision edge, or a misreading of the cut), recorded in a note.
blocked_by: []
opened: 2026-09-26
tags: [madevent, reference-quality, decay-chain, cuts]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-06}
sources:
  - {id: todo, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L305-L308", title: "TODO.md entry T016"}
---
In the MadGraph-only study of identical particles across decays
(`p p > z z, z > e+ e-` at 13 TeV, fixed μ = M_Z, bwcutoff 15), 27 of 500 000
MadEvent events violated a window that its `dummy_cuts` should enforce. The
study's scripts and numbers were kept outside the repository
(`/home/user/mg-4e-study/RESULTS.md`, not present in this container), so the
reproduction starts from the card in the note.

Nothing here is gated on it. It matters before any reference reads a windowed
MadEvent sample event by event, since such a comparison would count these
events as a disagreement. Detail:
[note 38 §5](../../history/notes/38-process-grammar-sprint-plan.md) (the 2026-09-26 entry).
