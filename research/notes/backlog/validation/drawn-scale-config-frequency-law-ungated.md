---
type: Backlog Item
title: The scale-configuration draw's ∝ AMP2_c law and zero-spread census are not standing gates
description: No accessor exposes the drawn scale configuration, so the AMP2_c/ΣAMP2 frequency law is gated only in pieces and the inert-row zero-spread census was a one-off diff.
area: validation
state: open
priority: medium
closes_when: The drawn configuration is observable, one test asserts its frequency ∝ AMP2_c/ΣAMP2 end-to-end, and a banked assertion fails if a declared-inert row's scale becomes configuration-dependent.
blocked_by: []
opened: 2026-08-03
tags: [scales, chain-b, amp2, oracle, hadronic]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-06}
sources:
  - {id: todo, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L446-L453", title: "TODO.md entry T030"}
---
Chain B draws one scale configuration per point, `∝ AMP2_c` of the event's
flavour group. Two of its accepted gaps are still open.

**1. The frequency law is asserted only by factorisation.** Four independently
gated pieces cover it:
- `select.rs`'s binomial test;
- the order pin;
- the colour-draw χ²;
- σ.

`madevents_scale_configuration_is_drawn_from_its_own_matrix_elements_amp2`
(`vibegraph-lib/tests/validate_hadronic.rs`) checks *MadEvent's* draw against
the rule, not this integrand's use of it. An end-to-end assertion first needs
the drawn configuration exposed from the integrand.

**2. The zero-spread census is not banked.** Chain B-0 found every declared-inert
row's scale configuration-independent, but by a one-time manual diff. Promote it
to a banked assertion on the cheapest inert rows, so that a change making a
scale configuration-dependent fails a standing gate. Runtime cost is the only
obstacle.

Related unobserved counter: `scale_draw_fallbacks()`
(`vibegraph-lib/src/hadronic.rs:2700`).

Detail: [note 29, Chain B design and results](../../29-v01-validation-sprint-plan.md).
