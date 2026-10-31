---
type: Backlog Item
title: The 2→6 integrals cells stay info because their MadEvent references scatter
description: "Both 20-seed MadEvent references flag SCATTER, in channels carrying 0.0% of σ, so the cells stay info per D1's third amendment; bbx on main reads +2.4σ vs reported, +1.3σ vs equal-weight."
area: validation
state: open
priority: medium
closes_when: "Either the SCATTER rule is amended for flagged channels that carry a negligible share of σ, and both cells gate against their 20-seed equal-weight references, or the cells are recorded as info for good with that reason."
blocked_by: []
opened: 2026-10-23
tags: [two-to-six, madevent-reference, gate-promotion, scatter]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-23}
sources:
  - {id: log, resource: "../../sprints/two-to-six-gate/log.md", title: "two-to-six-gate log, 2026-10-23"}
---
The 20-seed records live only in the M3's local refdata-11 candidate bundle.

| row | reported (20) | equal-weight (20) |
|---|---|---|
| uux | 7.175795e-7 ± 5.84e-10 | 7.180603e-7 ± 7.14e-10 |
| bbx | 1.686455e-6 ± 1.31e-9 | 1.687758e-6 ± 1.52e-9 |

vibegraph `uux` (`784582d`) reads +0.29σ / +0.67σ at 600k / 1.2M against
reported. `bbx` on main (600k, 1.689996e-6 ± 6.94e-10) reads +2.39σ
against reported and +1.34σ against equal-weight. Gating waits on the
allocation fix, so the cells are read on a budget that reaches every channel.

Filed at the close-out of [two-to-six-gate](../../sprints/two-to-six-gate/sprint.md).
