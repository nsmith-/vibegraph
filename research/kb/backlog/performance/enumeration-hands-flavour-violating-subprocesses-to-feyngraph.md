---
type: Backlog Item
title: "Enumeration builds topologies for every charge-conserving subprocess, ~97% of which the model gives no diagram"
description: "Enumeration hands every charge-conserving assignment to feyngraph; on W/Z+4j lines ~97% violate a conserved flavour number and have no diagram, and three campaign cards died there."
area: performance
state: open
priority: high
closes_when: "Before topology assignment, an assignment whose total of any additive quantum number conserved by every vertex of the model is nonzero is skipped. The numbers are derived from the vertex table, not listed by hand. The diagram sets and enumeration census are identical on the banked layer and on a CKM model card. The enumerate wall time on a W(→ℓν)+0–4j MLM card and on Z(→νν)+4j is measured before and after."
blocked_by: []
opened: 2026-11-23
tags: [enumerate, mlm, multijet, cms-gridpacks]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-23}
---
`generate_sets_inner` calls `assign_topologies` on every surviving
(deduplicated, charge-conserving) assignment. On a W(→ℓν)+0–4j MLM card the
3j line handed 13 734 subprocesses to feyngraph and 289 had diagrams; 150 of
the line's 154 s went to the empty ones. The 288 lepton-flavour- and
generation-conserving subprocesses are exactly the non-empty ones. On the 4j
lines: W 45 720 survivors, 1 284 conserving; Z→νν 60 318 survivors, 1 878
conserving.

Campaign cards 58, 15 and 1009 (release, `-j 4 --parallel-diagrams`) died
in their last process line at the cap. 1009's 4j line projects to ~2900 s
of feyngraph work against ~90 s for its conserving subprocesses
(extrapolated). Separately, the candidate loop is serial:
`--parallel-diagrams` only parallelises inside one subprocess.

Evidence: [L-T2 report](../../sprints/v023-orders-majorana-timeouts/sessions/L-T2-report.md), [phase sample](../../sprints/v023-orders-majorana-timeouts/v022-1009-phase-sample.md). This answers the first
question of [`diagram-enumeration-cost-unmeasured`](../feature/diagram-enumeration-cost-unmeasured.md).
