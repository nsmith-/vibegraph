---
type: Backlog Item
title: A colourless odd-momentum vertex of self-conjugate bosons takes −1 by derivation alone
description: "boson_momentum_sign gives −1 to any all-boson vertex of self-conjugate legs with odd momentum parity, colour or not; only coloured ones (ggg, hggg, the cG contact) are measured."
area: validation
state: open
priority: low
closes_when: "A standalone toy-UFO row with a colourless VVV vertex of three self-conjugate vectors is gated per graph against MadGraph AMP(), confirming the −1 that boson_momentum_sign gives it."
blocked_by: []
opened: 2026-10-16
tags: [amplitudes, sign-convention, toy-ufo, standalone-jamps]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-16}
sources:
  - {id: t1, resource: "../../sprints/amplitude-signs/sessions/T1-report.md", title: "T1 report, Found 3"}
---
T1 traced the triple-gluon −1 to a convention difference: MadGraph's HELAS
momenta are all-outgoing and vibegraph's all-incoming, so each momentum factor
of a vertex flips sign, and a conjugate pair of legs adds the slot-exchange
parity `ε_slot`. `boson_momentum_sign` (`root_diagram.rs`) now applies
`(−1)^{n_P}` to every all-boson vertex whose legs are all self-conjugate,
without a colour condition. Sprint H pinned it on coloured vertices only:
`gg_to_hg_chg`, `gg_to_hg_chgtil` and `gg_to_ggg_cg` at −1, `gg_to_hgg_chg` at
+1 ([vector-vertex-signs](../../amplitudes/vector-vertex-signs.md)). The SM and
SMEFTsim have no colourless odd-momentum vertex of self-conjugate bosons (an
anomalous ZZγ coupling would be one), so for that case the −1 is derived, not
measured.
