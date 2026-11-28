---
type: Backlog Item
title: "The hadronic integrand jumps by up to 8e-3 when x₁, x₂ move by one ulp, on 15 of 230 accepted γ+4j points"
description: "Card 252's γ+4j weight changes by 1e-7–8.4e-3 relative under a ≤4e-16 change of the momentum fractions; some changes are smooth, some are jumps. Not bisected."
area: validation
state: open
priority: medium
closes_when: "The step that jumps (cuts, MLM veto, clustering scales, αs reweighting or PDF interpolation) is identified on the recorded points, and either shown to be a genuine boundary crossing (a cut edge within an ulp) or reformulated so the weight is continuous at the scale of the change."
blocked_by: []
opened: 2026-11-28
tags: [numerics, stability, mlm, scales, pdf, cuts]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-28}
---
Found by `vg-validator`'s coverage harness on CMS card 252 (γ+4j, build
59045b57). The harness checks that its weight equals production `value(u)` to
2.2e-16 at 230 accepted points. Perturbing x₁ and x₂ by at most 4e-16 moves
the integrand shape (jacobian × flux × Σ lumi × rwgt × |M|²) by 1e-7–1e-6
smoothly on some points, and by jumps up to 8.4e-3 on others: 15 of 230 in all.
Evidence: the `D6 rw` lines of the validator's d6 diagnostic log.

A jump of 8e-3 under an ulp is far above the algorithm's own error scale, so it
is either a boundary the point sits on within an ulp (a cut, the MLM veto, a
clustering decision), which happens but should be rare, or a numerical
instability in a scale or reweighting step. 15 of 230 is not rare.
