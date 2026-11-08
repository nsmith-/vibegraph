---
type: Backlog Item
title: "A card with no ŝ floor panics at the first point with x = NaN instead of being refused"
description: "A massless final state with a leg left uncut gives τ_min = 0 and the first PDF lookup panics at x = NaN (p p > b b~ at ptb = 0 on main); MadGraph refuses such a card."
area: feature
state: open
priority: medium
closes_when: "A card whose τ floor is zero is refused at integrand construction with a typed refusal naming the uncut leg, with a test on p p > b b~ under sm-no_b_mass at ptb = 0."
blocked_by: []
opened: 2026-11-07
tags: [panic, refusal, cuts, tau-floor, pdf]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-07}
sources:
  - {id: hk1, resource: "../../sprints/cms-gridpack-coverage/sessions/H-K1-report.md", title: "H-K1 report, Found: a card with no ŝ floor"}
---
Found by H-K1. A massless final state with a leg left uncut gives `τ_min = 0`,
and the first point stops with `PDF evaluation point (x=NaN, …)` (the message
of `vibegraph-lib/src/pdf/extrap.rs:85`). It holds on main for `p p > b b~`
under `sm-no_b_mass` at `ptb = 0`. Item 8's literal card (template `ptb = 0`,
`etab = −1`) was a clean refusal before per-group cut indicators and now reaches
this panic.

MadGraph refuses that card (`setcuts.f:808`), and its σ is collinear-divergent
anyway. The fix is a refusal when the τ floor is zero, before any point is drawn.
