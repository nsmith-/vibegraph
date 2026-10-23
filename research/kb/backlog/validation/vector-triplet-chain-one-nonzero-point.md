---
type: Backlog Item
title: "A vector-triplet decay chain integrates to one nonzero point with μF = 0"
description: "CMS card 048 (p p > vz, vz > h z) exits 0 with σ = 0.093 pb from a single nonzero point and logs μF = 0; the cause is undiagnosed."
area: validation
state: open
priority: medium
closes_when: "Card 048's run is diagnosed (why one point passes and why μF is 0) and either its σ agrees with MadGraph at the gridpack's param card and PDF, or the defect is fixed or refused with a test."
blocked_by: []
opened: 2026-10-20
tags: [decay-chains, scales, non-sm-ufo, triage, cms]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-20}
sources:
  - {id: survey, resource: "../../validation/cms-gridpack-survey-2026-10.md", title: "CMS gridpack survey, 88 production cards (e3f543f)"}
---
In pass 2 of the [CMS gridpack survey](../../validation/cms-gridpack-survey-2026-10.md),
card 048 (heavy vector triplet, `p p > vz, vz > h z`) produced σ = 0.09296 pb
from one nonzero phase-space point, and the run logged a factorisation scale
of 0. A single point and a zero scale both say the result means nothing, yet
nothing warned. MadGraph's own survey line for this gridpack is implausible,
so it is no oracle until rerun.

Start from the per-event record: the scale, the forced-window cut and the
point's weight on the one accepted event, against MadGraph's event record at
the same card.
