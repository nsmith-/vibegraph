---
type: Backlog Item
title: Closed-form scale choice 1 is evaluated on lab momenta at hadron beams
description: E_T = E·p_T/|p| is not boost-invariant, and MadEvent's setscales.f evaluates it on partonic-CM momenta; the proton integrand hands it lab momenta.
area: validation
state: open
priority: low
closes_when: Choice 1 reads partonic-CM momenta in the hadronic integrand, pinned by a test on a point with nonzero partonic rapidity that fails on the lab-frame code, and the Frames docs on ProtonIntegrand and hadronic.rs say so.
blocked_by: []
opened: 2026-10-15
tags: [scales, frame, hadronic]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-15}
sources:
  - {id: f1, resource: "../../sprints/sigma-ttx-mlm-at1/log.md", title: "sigma-ttx-mlm-at1 F1 and HY1 Found"}
---
`ProtonIntegrand::shared_scale_sum` → `event_scales_in` → `scales_at(lab)`
evaluates the closed-form choices on lab momenta (`proton.rs` near 2402).
Choices 2 and 3 and √ŝ are longitudinally boost-invariant. Choice 1's E_T is
not. No banked row uses choice 1 at hadron beams, so nothing gates it. The
clustering path was moved to the CM for the same reason
([sigma-ttx-mlm-at1](../../sprints/sigma-ttx-mlm-at1/closeout.md)).
