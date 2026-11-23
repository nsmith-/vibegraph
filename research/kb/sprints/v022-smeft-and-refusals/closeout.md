---
type: Sprint Record
title: "Sprint K: v0.2.2, SMEFT models and the largest refusal classes"
description: "Twelve feature PRs and a version-bump PR before v0.2.2, four hygiene passes, and the release. The post-release campaign on the released binary read 0 panics and no card that passed on v0.2.1 fails (PASS 219 → 303). Its open ends are filed as items for sprint L."
closed: 2026-11-22
status: draft
tags: [release, v0.2.2, smeft, smeftsim, ufo, refusals, cms-gridpacks, simshop, sprint-k, sprint-record]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-22}
sources:
  - {id: plan, resource: "sprint.md", title: "Sprint K plan"}
  - {id: log, resource: "log.md", title: "Sprint K log"}
  - {id: notes, resource: "../../../../docs/release-notes/v0.2.2.md", title: "v0.2.2 release notes"}
  - {id: defects, resource: "../../validation/madgraph-defects.md", title: "MadGraph defects (the 2.9.18 leading-jet-window deviation)"}
---
The sprint session ran the [plan](sprint.md), dispatching one dev agent per PR
and verifying each one itself. The day-by-day account, with every command and
measurement, is in the [log](log.md). Each session's report is in `sessions/`.

## What landed

| PR | Squash | What it established |
|---|---|---|
| #141 | 54ded749 | Signed UFO coupling orders, spent vertex by vertex as MadGraph does, and the gluon-free WEIGHTED trial. SMEFT_alpha matches MadGraph per diagram and per helicity ([K-O1](sessions/K-O1-report.md)). |
| #142 | 7b46311a | One-body (2 → 1) phase space at proton beams. The fixed-beam case stays a named refusal, because MadGraph returns zero ([K-P1](sessions/K-P1-report.md)). |
| #143 | cce68116 | Batch-safe `integrate`: SIGTERM stops a run, the result records a stop reason, plus `--check`, atomic output, enumerate totals, and default threads from CPU limits ([K-B1](sessions/K-B1-report.md)). |
| #144 | 09490121 | Restriction cards applied as MadGraph applies them ([K-R1](sessions/K-R1-report.md)). |
| #145 | f65d1bb7 | Seven UFO value-form classes and the `import model` options ([K-U1](sessions/K-U1-report.md)). |
| #146 | 82ed311c | The coloured-scalar seagull colour structure ([K-S1](sessions/K-S1-report.md)). |
| #147 | 15de3a8f | Subprocess groups split by outgoing masses ([K-X1](sessions/K-X1-report.md)). |
| #150 | 0340a897 | Repeated identical decays stitched once; σ had been exactly 4× ([K-D1](sessions/K-D1-report.md)). |
| #152 | f78fb60b | Run-card off sentinels, the resolved-card report, the classification table, and the jet and lepton rank cuts ([K-C1](sessions/K-C1-report.md), [K-C2](sessions/K-C2-report.md)). |
| #153 | 74596ba3 | No panic in MLM reweighting on the g2HDM cards. Groups that vanish at the card are not clustered ([K-M1](sessions/K-M1-report.md)). |
| #155 | c1a9531b | `integrate` memory flat on multijet MLM cards ([K-G1](sessions/K-G1-report.md)). |
| #156 | b3bc5399 | The fixed-beam sampled-point fault and a binding `misset < 0` become named refusals ([K-N1](sessions/K-N1-report.md)). |
| #157–#160 | 4d408579, 0f53755b, 92dee0d7, 43d2429e | Four hygiene passes, one per code area ([K-H1](sessions/K-H1-report.md) to [K-H4](sessions/K-H4-report.md)). |
| #161 | acd09813 | `ENUMERATION_VERSION` 3: v0.2.1 artifacts were read silently and gave the 4× σ again. Found while preparing the release candidate. |
| #148 | c26980f1 | The record and release preparation. v0.2.2 is tagged on it. |
| #167 | 8093df47 | The post-release campaign's close-out. |

## Release

v0.2.2 is tagged on c26980f1. Its code is the release candidate 400c88f9 that
the CERN host checked.

The release gate changed mid-sprint, by Nick on 2026-11-20. The CERN host and
simshop run only pre-built binaries:
- **Before the tag:** CI on main, plus the CERN host's targeted check of the
  cards that failed in v0.2.1. The check showed no panics, and each failing case
  was deliberate or filed.
- **After the tag, on the released binary:** the full campaign `v022-1009`.

| verdict | v0.2.1 | v0.2.2 |
|---|---|---|
| PASS | 219 | 303 |
| FAIL | 77 | 78 |
| panics | 7 | 0 |

No card that passed on v0.2.1 fails, and wall time is unchanged.

## Carried forward

These are filed, and most are in sprint L's plan:
- **`g2hdm-charged-higgs-high-mass-sigma-reads-high`** (high). Sprint L takes it
  as a stretch goal.
- **`vbs-stable-boson-sigma-reads-high`** (high).
- **`tptp-integrand-zero-on-every-point`**, raised to high: zero |M|² while
  MadGraph gives a σ.
- **`sigma-reads-high-on-mlm-and-wbq-cards`.**
- **`stitch-set-sign-assert-panics`** (low; never reached in the campaign).
- **The refusal classes behind 435 refusals:** NLO, Majorana and
  fermion-number violation, spin-2/3/2, squared orders, polarized.
- **178 cards that time out on every seed.**
  Correction 2026-11-23: 6 of these 178 (VBS 136, 724, 726, 727, 728; VBF 290) carry a batch-slot stall signature: 290 refuses in 16.6 s on the released binary, and the VBS_WZ cards are refused at parse (local check pending), so they are not timeouts; see the [sprint L log](../v023-orders-majorana-timeouts/log.md) (2026-11-23).

## Lessons

- **An enumeration change needs a version bump and a known-answer card.**
  #150 fixed a 4× σ, but did not bump `ENUMERATION_VERSION`. Its known-answer
  set had no card that reached the change, so v0.2.1 artifacts brought the 4×
  back silently. Only a v0.2.1-built artifact read with the candidate showed it.
  #161 added the missing card, and a mutation confirmed that the card catches
  the old stitching.
- **A reference σ from an older MadGraph is a hypothesis.** The DY1j window
  failed by up to 16 standard errors against the gridpacks' MadGraph 2.9.18.
  MadGraph 3.7.1 on the same cards agreed with vibegraph to 0.16%. Before
  filing a vibegraph bug against an old gridpack, regenerate the reference with
  a current MadGraph.
- **Counts relayed across sessions need checking.** Two checkpoint counts did
  not add up, and one breakdown was revised from interim to final figures. Each
  was caught by summing the parts before relaying.
- **Commit identity.** One merge commit used the harness identity because the
  `-c` flags were left off. The repository's local git config now carries
  Nick's identity.
