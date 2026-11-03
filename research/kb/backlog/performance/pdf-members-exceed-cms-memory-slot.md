---
type: Backlog Item
title: The CMS systematics PDF list needs 3.8 GB resident, over CMS's 2 GB job slot
description: "generate with CMS's 880-member systematics request peaks at 3.76 GB RSS, about 4.2 MB per loaded member; CMS production slots allow 2 GB."
area: performance
state: open
priority: high
closes_when: "generate with the CMS runcmsgrid.sh systematics request (880 members) on pp_to_ll at lhaid 325300 peaks below 2 GB resident, measured, with the CMS-arguments weights gate still exact."
blocked_by: []
opened: 2026-10-30
tags: [systematics, pdf, memory, cms, lhapdf]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-30}
---
Measured in sprint E's S-V1 session (`research/kb/sprints/systematics/sessions/S-V1-report.md`).
The run was `generate` on `pp_to_ll` at lhaid 325300 with 10,000 events and CMS's 43-set,
880-member `--pdf` list (926 weights per event). It peaked at 3.76 GB resident at `-j 1` and at
`-j 4`, against 0.44 GB for the card's own 147-weight request. The test process peaked at
4.83 GB. The design estimated 3.1–3.5 GB ([systematics weights](../../events/systematics-weights.md) §4.4).

A loaded member takes 3.5–4 MB, while its raw knots are about 0.56 MB, so most of the
footprint is derived interpolation data. What would close it, cheapest first:
- store each member nearer to LHAPDF's own layout (knots, with coefficients computed on use);
- or keep the derived coefficients as `f32` where the weights gate stays exact at MadGraph's
  printed eight digits.

Either change must be measured against the CMS-arguments weights gate
(`validate_systematics`, `SYSTEMATICS_CMS_PDFS`), which is exact today.
