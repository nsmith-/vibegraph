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
which breaks the hard-error rule.

All 7 are one family, `g2HDM_charged_higgs_cgbh_M{300,450,900,1000,1250,1500,1750}_rhotu00_rhott06_rhotc04`.
They are under
`/cvmfs/cms-griddata.cern.ch/phys_generator/gridpacks_tarball/pp/13p6TeV/madgraph/g2HDM_charged_higgs/`.
Each panicked 375–970 s into the run (seed 1, 4 cores).

The repro is M300. MadGraph's σ is 10.04 ± 0.039 pb. Its process card:

```
import model g2HDM_UFO
define p = p b b~
define j = p
generate p p > h+ QCD=99, (h+ > t b~, (t > w+ b , W+ > l+ vl))
add process p p > h+ j QCD=99, (h+ > t b~, (t > w+ b , W+ > l+ vl))
add process p p > h+ j j QCD=99, (h+ > t b~, (t > w+ b , W+ > l+ vl))
```

The same three lines follow for h−. The run card is the LO template with:
- `ickkw = 1`, `xqcut = 30`, `maxjetflavor = 5`;
- `drjj = drjl = 0`, `ebeam = 6800`;
- `lhaid = 325300`.

The bundled `g2HDM_UFO` and its param card are used. The command:

```
vibegraph integrate proc_card.dat --run-card run_card.dat --param-card param_card.dat \
  --ufo-dir models --neval 120000 --target-rel 0.01 --seed 1 -j 4 --no-tui
```

The CERN host has the bundle (`mlm-panic-repro.tar.gz`, sha256 `f24c1f74…`).
It suspects that the five-flavour `p` with b in the h± → t b decay lets a merge
join a decay-product b with a radiated parton. That is not yet checked.

Two readings to separate before fixing:
- MadEvent never reaches this merge, because its clustering picks a different
  history. Then vibegraph's clustering differs, and the fix is in the cluster
  choice.
- MadEvent reaches it and stops. Then the card cannot run in MadGraph either,
  and the right answer is a named refusal before sampling.
