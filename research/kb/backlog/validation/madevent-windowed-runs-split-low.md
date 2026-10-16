---
type: Backlog Item
title: Windowed MadEvent runs split into two populations at low pT
description: On gg_to_gg_cg, ptj/ptjmax windows below 50 GeV leave 3-4 of 5 fresh seeds 0.4-0.7 % low with errors 10-20x smaller than the gap; windowed references may carry the same split.
area: validation
state: open
priority: medium
closes_when: Each banked windowed MadEvent reference (pta_window_reference.json, higgs_window_reference.json and similar) has a fresh-directory seed sweep checked for a bimodal split, and any affected row is re-banked or flagged.
blocked_by: []
opened: 2026-10-15
tags: [madevent-reference, windows, seeds]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-15}
sources:
  - {id: v1, resource: "../../sprints/sigma-gg-cg/sessions/V1-report.md", title: "sigma-gg-cg V1, Found 1"}
---
On fixed-beam `gg_to_gg_cg`, five fresh pinned-3.7.1 seeds per `ptj`/`ptjmax`
window below 50 GeV split two ways. Three or four land 0.4–0.7 % low with
quoted errors near 30 pb, and the rest land on this side's deterministic
quadrature (seed χ²/dof 20–123). Above 50 GeV every window agrees to about
1e-4. The windows sum to 0.33 % below MadEvent's own unwindowed σ. Banked
windowed references were built the same way and have not been checked for
it.
