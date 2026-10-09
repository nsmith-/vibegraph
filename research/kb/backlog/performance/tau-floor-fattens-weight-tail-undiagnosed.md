---
type: Backlog Item
title: An exact higher τ floor fattens the weight tail, cause undiagnosed
description: The provably implied bound √ŝ ≥ 50 + 20·n_j cut pp_to_llj_fixed's ε_unw from 5.45% to 3.18%; why a tighter exact floor makes the tail heavier is unknown.
area: performance
state: open
priority: low
closes_when: The mechanism by which the higher τ floor raises pp_to_llj_fixed's summed w_max is identified (e.g. by a binned weight-tail decomposition), and the floor is re-adopted or its rejection recorded with that cause.
blocked_by: []
opened: 2026-09-29
tags: [tau-floor, weight-tail, unweighting, phase-space-map, mlm]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-06}
sources:
  - {id: todo, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L970-L974", title: "TODO.md entry T089"}
---
The partition bound `√ŝ ≥ max over partitions Σ max(Σ p_T,min, √timelike_floor)`
is `50 + 20·n_j` on the matched cards, against today's `max(mmll, Σ p_T,min)`.
It is provably implied: a probe counted 0 accepted points below it. Measured,
then reverted:
- it removes 6–7.5 % of the draws in the survey and first iteration, then
  0.8–1.1 % per iteration;
- `pp_to_llj_mlm`'s rel²·CPU moved from 516 ± 95 to 460 ± 99 µs, which is not
  significant;
- `pp_to_llj_fixed` (`xqcut = 0`, `mmll = 50`): the summed `w_max` rose
  7.80e3 → 1.33e4 and the predicted ε_unw fell 5.45 % → 3.18 %.
  `cli_generate_proton`'s sample-against-integration bound failed at −1.80 %
  (bound 1.5 %), and the five-seed headroom spread widened from ±0.43 % to
  −1.80…+1.57 %. σ held (pulls −0.00, −0.04).

A tighter floor that excludes no accepted point should not by itself make the
weights worse. Diagnose this before tightening any future floor.

Detail: [note 41 §4, "M6 Landed"](../../history/notes/41-mlm-feature-sprint-plan.md).
