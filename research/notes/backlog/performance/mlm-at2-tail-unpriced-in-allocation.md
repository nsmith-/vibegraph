---
type: Backlog Item
title: The budget allocation does not price @2's heavy tail
description: pp_to_ll_0j2j_mlm's two-jet part is sized from quoted spreads that its heavy tail understates, so --neval must be over-asked for @2's precision.
area: performance
state: open
priority: medium
closes_when: The part split (sₖ) and the Neyman re-split account for tail-understated spreads, and at fixed --neval the seed-spread gain in variance × time matches the quoted-error gain within the seed scatter (≥10 seeds).
blocked_by: []
opened: 2026-09-30
tags: [mlm, allocation, neyman, weight-tail, budget]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-06}
sources:
  - {id: todo, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L952-L960", title: "TODO.md entry T086"}
---
With channel merging, the redundant per-channel floors are gone, so
`pp_to_ll_0j2j_mlm`'s `@2` part gets only what the part split (`sₖ`) and the
Neyman re-split give it. Both are computed from quoted spreads. `@2`'s heavy
tail makes its quoted error an underestimate: seed χ²/dof is 1.89 over ten seeds
at `--neval 200000`, and its seed spread is 4× the pre-merge base's at 3.5× less
CPU.

- At the same `--neval`, the total gains 1.5× in variance × time measured by
  seed spread, where the quoted errors promise 3×.
- At the same point count (`--neval 600000`, three seeds), every part gains:
  8.6× in rel²·CPU by quoted error.

The merge works as a budget knob today: `--neval` sets how many points `@2`
gets. Pricing the tail in the allocation would remove the need to over-ask
`--neval`. One option is to widen a part's spread by its iteration scatter, as
`stop_scale` does for the stop.

Detail: [note 41 §4, "F-B Landed"](../../41-mlm-feature-sprint-plan.md).
