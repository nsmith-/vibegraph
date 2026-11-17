---
type: Backlog Item
title: "Older run-card templates' 'off' sentinels are refused as unimplemented cuts"
description: "ht2max/ht3max/ht4max = 100000 (24 cards) and misset, ptheavy, ptl1min–ptl4min = −1 (7 cards) are refused; cuts.f treats these values as no cut."
area: feature
state: open
priority: medium
closes_when: "Each sentinel value that MadGraph's cuts.f / setcuts.f provably leaves inert (cited per parameter) compiles to the default cuts, a value that would cut is still refused until implemented, both with tests, and the 31 campaign cards get past this refusal."
blocked_by: []
opened: 2026-11-17
tags: [cuts, run-card, refusal, cms-gridpacks, simshop]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-17}
---
The CERN host's v0.2.1 campaign `v021-1009-s1` found 31 cards refused this way.
Older run-card templates write a parameter's "off" value instead of leaving the
line out:
- `ht2max`, `ht3max` and `ht4max` at 100000 (24 cards);
- `misset`, `ptheavy` and `ptl1min`–`ptl4min` at −1 (7 cards).

vibegraph lists these in `UNIMPLEMENTED_CUTS` and refuses them whatever their
value. simshop ranks this third among its development blockers.

The same pattern for `ptj1min`–`ptj4min` is
[`ptj-n-min-nonpositive-refused`](ptj-n-min-nonpositive-refused.md). Its fix
(`NONPOSITIVE_IS_OFF`, `vibegraph-lib/src/cuts.rs`) is the model for minimum
cuts. Maximum cuts need an "at or above the template's off value" rule, read
from cuts.f's guards for each parameter.
