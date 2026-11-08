---
type: Backlog Item
title: "With cut_decays = F, identical quarks across a decay are all cut, a stricter observable than the card asks for"
description: "vibegraph cuts every quark that is a core particle in some pairing, so g g > z u u~, z > u u~ reads −26% below MadGraph; no kinematic rule fixes it on the every-pairing |M|², only a per-pairing one."
area: validation
state: open
priority: high
closes_when: "Under cut_decays = F each pairing's decay products are exempt only in that pairing's share of |M|² (a per-pairing weight or the single-pairing |M|² of compat site (d)), a test pins that no exempt leg leaves a massless limit uncut in the |M|² it multiplies, and σ on g g > z u u~, z > u u~ and g g > z j j, z > j j matches local MadGraph by the multi-seed criterion."
blocked_by: []
opened: 2026-11-06
tags: [cuts, decay-chains, identical-particles, madgraph-parity, cms]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-06}
sources:
  - {id: ic1, resource: "../../sprints/v02-last-refusals/sessions/I-C1-report.md", title: "I-C1: the window rule diverges through an exempt quark's beam-collinear limit"}
  - {id: d2, resource: "../../sprints/v02-last-refusals/decisions/D2-cut-decays-keep-the-stricter-cut.md", title: "Sprint I D2: keep the stricter cut for v0.2"}
  - {id: hk1, resource: "../../sprints/cms-gridpack-coverage/decisions/D5-cut-decays-exemption-and-b-cut-cards.md", title: "D5 (revised) and its sources"}
  - {id: hc2, resource: "../../sprints/cms-gridpack-coverage/sessions/H-C2-report.md", title: "H-C2: why the configuration-1 exemption diverges on an every-pairing |M|²"}
---
`cut_decays = F` is the default in MadGraph (`banner.py:4306`) and in vibegraph,
and 10 of the 11 CMS decay-chain survey cards use it.

MadGraph keeps one pairing of identical particles across a decay and exempts its
decay products (`check_decay`, configuration 1). vibegraph keeps every pairing and
cuts every leg that is a core particle in some pairing: all four quarks of
`g g > z u u~, z > u u~`. That is a stricter observable, 5.348 against MadGraph's
7.211 pb (−26%), −6.8% on the full `g g > z j j, z > j j` card. It is a registered
deviation ([D5](../../sprints/cms-gridpack-coverage/decisions/D5-cut-decays-exemption-and-b-cut-cards.md)),
kept for v0.2 ([sprint I D2](../../sprints/v02-last-refusals/decisions/D2-cut-decays-keep-the-stricter-cut.md)).

**No kinematic exemption is IR-safe here.** Sprint H showed that exempting MadGraph's
configuration-1 legs diverges (H-C2). Sprint I showed the same for the window rule
(exempt the pairing whose pair mass is inside the forced window): an exempt quark
can go collinear with a beam while its pair stays on the pole, and in the other
pairings it is a core quark of a `g → q q̄` splitting, so |M|² ∝ 1/pT² (I-C1, a
matrix-element probe). Any rule that leaves an identical massless quark uncut on
the every-pairing |M|² diverges.

The fix needs the exemption to act per pairing: weight the cut indicator by each
pairing's share, w_p = |M_p|²/Σ|M_q|² (H-C2's option 2), or evaluate MadGraph's
single pairing (compat site (d) of
[madgraph-compat-sites-unconditional](../feature/madgraph-compat-sites-unconditional.md)).
Either needs the evaluator split per pairing. I-C1's probe
(`ic1_window_probe.rs`, in its report) is the regression oracle for any rule.
