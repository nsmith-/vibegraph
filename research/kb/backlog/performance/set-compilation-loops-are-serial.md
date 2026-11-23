---
type: Backlog Item
title: "The amplitude probe and the compile phase compile every diagram set one at a time"
description: "amplitude_probe::measure and derive_flavor_groups compile diagram sets in serial loops; card 65 spent 1077 s of its 1200 s cap in the probe on one core of four."
area: performance
state: open
priority: high
closes_when: "Set compilation (`derive_flavor_groups`, and `amplitude_probe::measure` while it compiles) runs across the rayon pool, results in set order. σ, grids and probe values are bit for bit on the banked layer. The wall time of both phases is measured on a Z(→jj)+0–4j card and a VBS ssWW card."
blocked_by: []
opened: 2026-11-23
tags: [compile, amplitude-probe, parallelism, cms-gridpacks]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-23}
---
Card 65 (Zto2Q-4Jets) spent 1077 s in the amplitude probe with 1225
user-s over 1200 wall-s: one core of four. The probe and compile loops
(`amplitude_probe.rs:262`, `proton.rs:1107`) are serial iterators. Card 14
(VBS): probe 167 s, compile 178 s.

Pairs with [`amplitude-probe-compiles-each-set-twice`](../performance/amplitude-probe-compiles-each-set-twice.md),
which removes the probe's compile altogether. Evidence:
[L-T2 report](../../sprints/v023-orders-majorana-timeouts/sessions/L-T2-report.md), [phase sample](../../sprints/v023-orders-majorana-timeouts/v022-1009-phase-sample.md).
