---
type: Backlog Item
title: A colourless triple-vector vertex of three self-conjugate vectors is predicted to need −1 and gets none
description: "triple_gluon_sign counts coloured VVV vertices as a proxy for 'no conjugate pair among the legs'. The derivation predicts −1 for a colourless vertex of three self-conjugate vectors too."
area: validation
state: open
priority: low
closes_when: "A standalone toy-UFO row with a colourless VVV vertex of three self-conjugate vectors is gated per graph against MadGraph AMP(), and triple_gluon_sign's predicate is whichever rule that row confirms."
blocked_by: []
opened: 2026-10-16
tags: [amplitudes, sign-convention, toy-ufo, standalone-jamps]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-16}
sources:
  - {id: t1, resource: "../../sprints/amplitude-signs/sessions/T1-report.md", title: "T1 report, Found 3"}
---
T1 traced the triple-gluon −1 to the slot exchange the all-incoming crossing
gives every triple-vector vertex with a conjugate pair of legs, which a gluon
vertex does not get. The code counts `is_yang_mills_vvv && is_coloured`. The two
rules differ only on a colourless vertex of three self-conjugate vectors (none in
the SM or SMEFTsim), where the derivation predicts −1 and the code gives +1.
