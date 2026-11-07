---
type: Backlog Item
title: "The full Zto2Q 4-jet card is too costly to run: 674 680 diagrams, then 11421 channels"
description: "Zto2Q-4Jets_Bin-HT-100to400 passes grouping but does not finish in 2400 s: 11421 channels at the 512-point floor make one iteration 5.8M points before any --neval is spent."
area: performance
state: open
priority: low
closes_when: "Zto2Q-4Jets_Bin-HT-100to400 integrates to its target within a stated budget on the CERN host, by a cheaper compile and a channel allocation whose first iteration does not scale with the channel count, with the cost recorded before and after."
blocked_by: []
opened: 2026-11-07
tags: [cost, channels, compile, decay-chains, cms]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-07}
sources:
  - {id: hd3, resource: "../../sprints/cms-gridpack-coverage/sessions/H-D3-report.md", title: "H-D3 report, Found: the full Zto2Q card did not finish in 900 s"}
  - {id: hk1, resource: "../../sprints/cms-gridpack-coverage/sessions/H-K1-report.md", title: "H-K1 report: the Zto2Q HT-100to400 stop is cost"}
---
Two measurements of the same card:

- H-D3: it took 222 s to enumerate 674 680 diagrams and was still compiling
  3700 subprocesses' evaluators when killed at 900 s.
- H-K1, after per-group cut indicators: past grouping on every multiplicity
  (13 / 93 / 165 / 675 flavour groups cut by 3 / 8 / 14 / 30 distinct filters),
  but the 4-jet integration does not finish in 2400 s. 11421 channels, each at
  the 512-point floor, make one iteration 5.8 M points before any `--neval` is
  spent.

The `HT-800to1500` variant was not measured (a container restart cut it).
Two costs to attack: the compile, and a per-channel point floor that grows the
first iteration with the channel count.
