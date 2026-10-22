---
type: Backlog Item
title: ℓℓj draws outside its jet-pT window given (τ, m_ll²)
description: "The ℓℓj jet pT edge is pinned by (τ, m_ll²) and is two intervals (±cos θ) in the all-timelike channels; a conditional window would take ~42% of rejections, ceiling about 1.5×."
area: performance
state: open
priority: low
closes_when: "A map option (off by default, recorded in MapChoices) narrows the ℓℓj polar coordinate to the jet-pT window given (τ, m_ll²), as one interval in the spine channels and a union of two in the all-timelike ones; the walk–density reciprocity test passes with it on; a point-level test finds zero accepted window-off points outside it in ≥10⁶ draws per channel; σ agrees on and off at ≥5 seeds per arm on pp_to_llj_fixed and pp_to_llj_mlm, and the gain is measured at 20+ seeds on the deciding pair."
blocked_by: [jj-two-to-two-cut-window-missing]
opened: 2026-10-22
tags: [phase-space-map, cut-edge, window, pp_to_llj]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-22}
sources:
  - {id: design, resource: "../../phase-space/sampler-and-map-seams.md", title: "The importance sampler and the phase-space map, designed together (D1)"}
  - {id: m1, resource: "../../sprints/vegas-absolute-grid-coords/sessions/M1-report.md", title: "M1 report, M2 table"}
---
M1's M2 table pins the ℓℓj jet `pT` edge by `(τ, m_ll²)` at `R = 0.035–0.048`
in the spine channels; in the all-timelike channels (0 and 1) it sits on two
branches, `±cos θ` (`R = 0.20`), so the window is a union of two intervals. The
edge causes 40–45% of ℓℓj's rejections and 9–17% of its top-0.1% second
moment; M5's ceiling is 1.45–1.50× in the Z channels.

The union is mapped by concatenating the intervals in order, each taking a share
of `u` proportional to its length
([joint seam design](../../phase-space/sampler-and-map-seams.md), D1 point 4).
After the `pp_to_jj` window, which builds the option and its oracles on an exact
case.
