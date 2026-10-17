---
type: Backlog Item
title: "The t-channel map is bounded by the pT cut alone, so a binding rapidity cut leaves a step VEGAS misses"
description: "Cuts::spacelike_floor is pT_min² alone; on e+ e- > e+ e- $$ z the binding |η| < 2.5 edge (|t| ≥ 1673 GeV²) sits at the sampling peak, so low-budget seeds miss a thin strip."
area: performance
state: open
priority: medium
closes_when: "The t-channel map's |t| bound includes the rapidity cut wherever the bound is provable, with a unit test that it never cuts an accepted point, ee_to_ee_nsz's ×¼ and ×1 rungs at χ²/dof near 1 on 20 seeds, and the fixed-beam σ gate's affected rows re-read."
blocked_by: []
opened: 2026-10-17
tags: [phase-space, t-channel, vegas, cuts, heavy-tail, bhabha]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-17}
sources:
  - {id: v1, resource: "../../sprints/ee-ee-nsz-low-budget/sessions/V1-report.md", title: "ee-ee-nsz-low-budget V1 report, §2, §4, §5, Found"}
---
## What V1 found

On `ee_to_ee_nsz`, the t-channel map starts at |t| ≥ ptl² = 100 GeV², but the
cut that binds is |η| < 2.5, which is |t| ≥ 1673.3 GeV². So 36 % of the map
coordinate carries zero integrand, and the edge sits at x = 0.640, where the
sampling weight is largest. A strip of cos θ ∈ [0.9863, 0.9866) holds about
2.2 % of σ. On low-budget seeds it gets about 1.5 draws an iteration, with
weights up to 396× the mean. Those seeds read low and quote errors 1.6–8.4×
too small
([V1](../../sprints/ee-ee-nsz-low-budget/sessions/V1-report.md)).

The falsifier moved only the map bound, through `ptl`, keeping the accepted
region unchanged:

| card | ×¼ χ²/dof | ×1 χ²/dof |
|---|---|---|
| ptl 10 (the row) | 3.84 | 2.01 |
| ptl 20 | 1.43 | 1.53 |
| ptl 40 | 0.77 | 1.12 |

At ptl 40 the CLI default takes 1.8× fewer evaluations at the median, at half
the error. σ is unchanged.

## The fix design (V1 §5)

For one massless leg, symmetric fixed beams and a 2 → 2 one-rung map, bound
|t| ≥ max(pT_min², (s/2)(1 − tanh η_max)). The bound sits beside
`Cuts::spacelike_floor` and is passed per rung through
`DiagramChannel::with_fiducial_t_max`. It moves every fixed-beam t-channel
integration, so the fixed-beam σ gate's readings and calibrations (PR #45)
follow it.

## Watch items

- **A possible low-budget offset on the ptl 40 card.** ×¼ reads +1.9σ and ×1
  +1.4σ against ×4 over 20 seeds, which cannot calibrate a rung difference.
  Read 40 seeds on ×¼ and ×4.
- **MadEvent may share the mechanism.** One default-survey MadEvent seed of
  this row reads 156.48 and is unattributed. A diagnostic run on the same
  card with ptl = 40 would show whether the low seed goes away.
