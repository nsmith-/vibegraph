---
type: Backlog Item
title: "On 496-group VBS cards a point costs ~9 ms CPU, mostly not in helicities"
description: "VBS cards 133 and 710 share their structure but differ 9× in helicities, yet their per-point survey cost differs only 2.3×; where the ~9 ms CPU per point goes is unmeasured."
area: performance
state: open
priority: medium
closes_when: "A profile of the integrand point on a VBS_WPMZTo4Q-shaped card attributes its cost (matrix elements, kinematics, cuts, PDF weights, channel densities), and the dominant part is filed or fixed."
blocked_by: []
opened: 2026-11-23
tags: [vbs, profiling, integrand, cms-gridpacks]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-23}
---
133 and 710 share 528 subprocesses, 496 flavour groups and 298 channels.
Their helicities are 8136 against 904, yet their survey costs per point are
about 6.3 against 2.7 ms wall at 4 threads: ~9 ms CPU per point that does
not scale with helicities. 528 subprocesses group into 496 flavour groups,
so the |M|² grouping barely merges anything there.

Related: [`mlm-every-group-matrix-element-evaluated-per-point`](../performance/mlm-every-group-matrix-element-evaluated-per-point.md).
Evidence: [L-T2 report](../../sprints/v023-orders-majorana-timeouts/sessions/L-T2-report.md), [phase sample](../../sprints/v023-orders-majorana-timeouts/v022-1009-phase-sample.md).
