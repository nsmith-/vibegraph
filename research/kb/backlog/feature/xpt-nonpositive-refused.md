---
type: Backlog Item
title: "A non-positive xptj, xptb, xpta or xptl is refused, though MadGraph reads it as off"
description: "detect_unimplemented refuses any xpt* off its default, but every MadGraph use is guarded by xpt* > 0 or max(xpt*, 0); 2 CMS gridpacks (ZtoAlpGto3G) stop on xptb < 0."
area: feature
state: open
priority: medium
closes_when: "A run card with xptj, xptb, xpta or xptl at or below zero compiles to the same cuts as the default, a positive value is still refused, and both are pinned by tests."
blocked_by: []
opened: 2026-11-04
tags: [cuts, run-card, cms]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-04}
sources:
  - {id: cuts-f, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/Template/LO/SubProcesses/cuts.f#L1005-L1015", title: "cuts.f: the xptb requirement, guarded by xptb > 0"}
  - {id: setcuts-f, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/Template/LO/SubProcesses/setcuts.f#L535-L665", title: "setcuts.f: smin from max(…, xpt*, 0)"}
---
`cuts::detect_unimplemented` refuses `xptj`, `xptb`, `xpta` and `xptl` whenever
they differ from the default 0 (`UNIMPLEMENTED_CUTS`, `vibegraph-lib/src/cuts.rs`).
In MadGraph every read of them is guarded: the cut itself by `xptb.gt.0d0`
(`cuts.f:1007`), the `smin` hint by `max(…, xptb, 0d0)` (`setcuts.f:584,591`),
and the failure report by `xptb.gt.0d0` (`setcuts.f:760`); the other three are
read the same way. So a value at or below zero acts as the default. In the
campaign `main-23d3a76-200-pc` on #71 this refuses both ZtoAlpGto3G gridpacks.

A positive value is a real cut ("at least one b with pT above xptb") and stays
refused until it is implemented.
