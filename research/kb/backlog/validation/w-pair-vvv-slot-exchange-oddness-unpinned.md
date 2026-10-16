---
type: Backlog Item
title: The oddness of every W-pair triple-vector structure under the slot exchange is unpinned
description: "triple_gluon_sign's derivation rests on every SM and SMEFTsim triple-vector structure on a W pair being odd under W⁺/W⁻ slot exchange; only a scratch script checked it."
area: validation
state: open
priority: low
closes_when: "A committed test evaluates each triple-vector Lorentz structure on a W pair in the SM and SMEFTsim with the two W slots exchanged, and fails if one is not odd."
blocked_by: []
opened: 2026-10-16
tags: [amplitudes, sign-convention, ufo, derivation]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-16}
sources:
  - {id: t1, resource: "../../sprints/amplitude-signs/sessions/T1-report.md", title: "T1 report, the derivation"}
  - {id: th, resource: "../../sprints/amplitude-signs/sessions/T-H-report.md", title: "T-H report, H12"}
---
T1's scratch `vvv_swap.py` found VVV1–VVV7 and VVV9 odd under the exchange
(VVV3 vanishes). `triple_gluon_sign`'s doc and
[vector-vertex-signs](../../amplitudes/vector-vertex-signs.md) state the result.
A structure that were even would carry no −1 inside its kernel, and the
convention would give it the wrong sign.
