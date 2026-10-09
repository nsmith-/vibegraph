---
type: Backlog Item
title: wpwm_to_wpwmz_cw's configuration partition is not compared with MadGraph's
description: The row's 21-group configuration partition matches MadGraph only in group-size multiset; no diagram pairing exists, so the partition itself is unchecked.
area: validation
state: open
priority: medium
closes_when: A diagram pairing derived from MadGraph's own per-diagram data (not fitted to the check) lets config_groups compare wpwm_to_wpwmz_cw's partition, and KNOWN_CONFIG_PAIRING_UNAVAILABLE no longer lists the row.
blocked_by: []
opened: 2026-09-07
tags: [non-sm-ufo, five-vector, config-groups, amplitude-oracle]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-06}
sources:
  - {id: todo, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L400-L412", title: "TODO.md entry T028"}
---
`config_groups` compares partitions as sets of MadGraph graph indices. For this
222-diagram row no per-diagram pairing is banked, so
`KNOWN_CONFIG_PAIRING_UNAVAILABLE` prints both partitions and lets the
pairing-free comparisons (|M|², per-flow JAMPs, JAMP2) run.

The two partitions agree only on the multiset of group sizes: 21 groups,
`{3×8, 7×8, 12, 17×4}`. They agree on nothing finer:
- Ours are the contiguous diagrams 62–221.
- MadGraph's run 2–186 with three gaps.

No index shift maps one onto the other. A pairing written now would be fitted to
the only check that reads it.

The honest route is MadGraph's own per-diagram cluster trees in
`validation/madgraph/output/wpwm_to_wpwmz_cw.json`. The exemption is two-way: it
fails the day the row banks per-diagram amplitudes. This is best taken by the
same session as the |M|² residual
([wpwmz-cw-ow-five-vector-residual](wpwmz-cw-ow-five-vector-residual.md)), since
that disagreement is the reason the pairing matters.

Detail: [note 35 §10.5](../../history/notes/35-ufo-lorentz-sprint-plan.md).
