---
type: Backlog Item
title: "Matched reweighting panics on a merge whose colour structure ipartupdate does not name"
description: "7 gridpacks in the CERN host's v0.2.1 campaign panic at proton.rs:3409: rewgt's UnnamedColourStructure (merge 4 joins lines [60, 64]) reaches an unwrap_or_else(panic!) on a sampled point."
area: validation
state: open
priority: high
closes_when: "The 7 cards never panic. The cause is named against reweight.f (is the merge one MadEvent reaches, and does MadEvent stop there or cluster differently?), then it is fixed with σ and the per-event merge sequence against MadGraph's record, or refused by name before sampling, with a failing-first test either way."
blocked_by: []
opened: 2026-11-17
tags: [mlm, panic, clustering, rewgt, madgraph-parity, cms-gridpacks]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-17}
---
Reported by the CERN host from its stage-1 campaign `v021-1009-s1`. That
campaign ran the v0.2.1 musl binary on 1005 CMS gridpacks, one seed each. All 7
panics in it are this one:

> matched reweighting on a sampled point: the clustering's merge 4 joins lines
> [60, 64] with a colour structure reweight.f's ipartupdate does not name,
> where MadEvent stops the run

`RewgtError::UnnamedColourStructure`
(`vibegraph-lib/src/coupling/cluster/rewgt.rs:292`) mirrors `ipartupdate`'s
`stop`. At v0.2.1, `proton.rs:3409` turns it into a panic on a sampled point,
which breaks the hard-error rule. The gridpack names were not in the tally; ask
the CERN host for them.

Two readings to separate before fixing:
- MadEvent never reaches this merge, because its clustering picks a different
  history. Then vibegraph's clustering differs, and the fix is in the cluster
  choice.
- MadEvent reaches it and stops. Then the card cannot run in MadGraph either,
  and the right answer is a named refusal before sampling.
