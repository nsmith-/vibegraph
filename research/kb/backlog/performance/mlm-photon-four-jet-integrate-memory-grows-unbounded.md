---
type: Backlog Item
title: "integrate's memory grows past 20 GB on an MLM γ + 4-jet card"
description: "GJ-4Jets_dRGJ-0p25_PTG-200_HT-40to100 reached 23.9 GB RSS by VEGAS iteration 45 of 180 (4 cores); a QCD-5Jets card earlier hit a 4.5 GB cap. What holds the memory is unmeasured."
area: performance
state: open
priority: high
closes_when: "A heap profile of integrate on the GJ-4Jets card names what grows with iterations (per-channel grids, cached matching histories, event buffers or the like). The growth is bounded, or justified and documented, with a test or measurement showing peak RSS flat in the iteration count. The card completes under a 12 GB limit."
blocked_by: []
opened: 2026-11-17
tags: [performance, memory, mlm, multijet, cms-gridpacks]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-17}
---
Reported by the CERN host from its v0.2.1 campaign `v021-1009-s1`, run with the
v0.2.1 musl binary at seed 1 on 4 cores.

`GJ-4Jets_dRGJ-0p25_PTG-200_HT-40to100` (MLM γ + 4 jets) reached 23.9 GB
resident by VEGAS iteration 45 of 180. The 12 GB cgroup limit then held it.
Growth that tracks the iteration count rather than the channel count points at
something kept per iteration. That is **inferred** from one run.

An earlier campaign's census follow-up had a smaller case:
`QCD-5Jets_Bin-HT-40` hit a 4.5 GB cap
([`full-cms-campaign-after-v02-cut`](../validation/full-cms-campaign-after-v02-cut.md)).

A global OOM once took down a whole host session (systemd `OOMPolicy=stop`), so
an unbounded integrate is a risk to batch hosts beyond the one card. The
multijet timeouts are a separate item:
[`multijet-mlm-cards-time-out-in-the-gridpack-campaign`](multijet-mlm-cards-time-out-in-the-gridpack-campaign.md).
