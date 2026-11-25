---
type: Backlog Item
title: "No hermetic test reaches the signed-|M|² sites of the split amplitude"
description: "The split JAMP2's |Re J_x J_y*|, helicity pruning on |T_c| (live in integrate) and the signed helicity draw are reached by no hermetic test; mutations M10–M12 and N12 survive."
area: validation
state: open
priority: medium
closes_when: "A JAMP2 check against MadGraph's own split JAMP2, and a pruning case with a combination whose T_c is negative at every probe point, each failing on its mutation."
blocked_by: []
opened: 2026-11-24
tags: [amplitudes, squared-orders, tests, mutation]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-24}
sources:
  - {id: report, resource: "../../sprints/v023-orders-majorana-timeouts/sessions/L-Z1-report.md", title: "L-Z1 report: Found 1"}
---
Found by sprint L's hygiene pass L-Z1 (Found 1); the evidence is in
[its report](../../sprints/v023-orders-majorana-timeouts/sessions/L-Z1-report.md).

Sites: `helas/eval/run.rs:462` (the split `JAMP2` that picks the colour flow), `run.rs:719` (MadGraph's `DABS(TS(I))` in helicity pruning, live in `integrate` on a signed selection) and `compile.rs:346` (the signed draw, unreachable while `generate` refuses a signed selection). The banked tables carry JAMPs, not `JAMP2`.
