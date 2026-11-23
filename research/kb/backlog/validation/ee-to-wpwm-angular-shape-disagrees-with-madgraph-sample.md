---
type: Backlog Item
title: "ee_to_wpwm y(w+)/cos(w+) disagree with MadGraph's banked sample on most seeds; σ agrees"
description: "Across 20 seeds the KS p of y(w+), cos(w+) and pt(w+) against MadGraph's banked sample is below 0.05 on 15, median ~1e-2, under both v0.2.2's serial sampler and the blocked pass."
area: validation
state: open
priority: high
closes_when: "The cause is found (vibegraph's sampler or amplitudes, the banked MadGraph sample's provenance or statistics, or the KS comparison itself, e.g. sample sizes or weighting), and the ee_to_wpwm and ee_to_wpwm_cw shape cells are back to gating with a pass rate measured across at least 20 seeds."
blocked_by: []
opened: 2026-11-22
tags: [samples-gate, ee-to-ww, madgraph-parity, shapes]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-22}
sources:
  - {id: ls1, resource: "../../sprints/v023-orders-majorana-timeouts/sessions/L-S1-report.md", title: "L-S1 report: sampler census and Found 5"}
---
Found by sprint L's PR 13 (multicore `generate`) when the changed generation
stream moved the samples gate's committed seed. The probe
`probe_wpwm_sampler_census` (`validate_samples.rs`) gives 20 seeds per sampler,
with the same grids and maxima:

| Sampler | y(w+)/cos(w+) KS p: min | median | seeds < 0.05 |
|---|---|---|---|
| v0.2.2 serial loop | 1.48e-4 | 9.2e-3 | 15/20 |
| blocked pass | 5.74e-5 | 1.30e-2 | 15/20 |

pt(w+) behaves the same way. The manager re-ran the census at 55b32dbd and
reproduced the summary exactly. The committed seeds had passed by luck,
1.6× above the 1e-4 floor. The rows' σ cells, χ², beams and scales all pass.

**Ruled out:** the blocked sampler, since the serial loop shows the same
distribution, and seed choice.

**Open:** which side owns the shape. A per-event comparison of the cos θ_W
spectrum against MadGraph's own σ in angular windows would decide it, as
`ee_to_mumua`'s pt(a) was decided. A KS test on weighted or unequal-size
samples would also produce this.

Meanwhile the two shape cells are `info` in `validation/manifest.toml`, by the
user's ruling (2026-11-22). They still run and print their KS p every run.

This replaces `ee-wpwm-samples-gate-thin-headroom` (low, 2026-10-31), which
recorded the thin headroom of the old seed without a diagnosis.
