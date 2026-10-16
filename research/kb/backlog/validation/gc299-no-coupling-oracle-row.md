---
type: Backlog Item
title: GC_299's seven-digit rounding has no coupling-oracle row
description: "GC_299 (h-γ-γ, from gHaa) is off by 2.8e-8 in MadGraph's Fortran, measured only per graph on standalone rows; no MadEvent coupling table carries it, so coupling_oracle cannot list it."
area: validation
state: open
priority: low
closes_when: "A coupling table for a MadEvent row with a photon-photon-Higgs vertex is banked and GC_299 is listed in KNOWN_FORTRAN_DEVIATIONS with its measured deviation."
blocked_by: []
opened: 2026-10-15
tags: [couplings, smeftsim, gc299, registered-deviation]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-15}
sources:
  - {id: f2, resource: "../../sprints/amplitude-signs/sessions/F2-report.md", title: "F2 report, Found"}
---
Today `GC_299`'s registered value rests on 25 per-graph measurements (spread 4e-15).
