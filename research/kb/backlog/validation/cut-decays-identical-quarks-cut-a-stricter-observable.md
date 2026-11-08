---
type: Backlog Item
title: "With cut_decays = F, identical quarks across a decay are all cut; the default should be a pairing-weighted cut"
description: "vibegraph cuts every quark that is a core particle in some pairing (−26% on g g > z u u~, z > u u~); D3 makes the default a pairing-weighted cut, Σ_p w_p Θ_p |M|² with w_p = |M_p|²/Σ_q |M_q|²."
area: validation
state: open
priority: high
closes_when: "With --madgraph-compat off, cut_decays = F is applied as Σ_p w_p Θ_p |M|²; I-C1's probe on the weighted integrand gives w_p Θ_p |M|² ε² → 0 in its three configurations; the limit guard runs per pairing with that pairing's exemptions; and σ on g g > z u u~, z > u u~ and g g > z j j, z > j j is recorded against MadGraph by the multi-seed criterion, with its difference attributed."
blocked_by: []
opened: 2026-11-06
tags: [cuts, decay-chains, identical-particles, madgraph-parity, cms]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-06}
sources:
  - {id: d3, resource: "../../sprints/v02-last-refusals/decisions/D3-identical-decays-behind-madgraph-compat.md", title: "Sprint I D3: the pairing-weighted default and the compat flag"}
  - {id: ic1, resource: "../../sprints/v02-last-refusals/sessions/I-C1-report.md", title: "I-C1: the window rule diverges through an exempt quark's beam-collinear limit"}
  - {id: d2, resource: "../../sprints/v02-last-refusals/decisions/D2-cut-decays-keep-the-stricter-cut.md", title: "Sprint I D2: keep the stricter cut for v0.2"}
  - {id: hk1, resource: "../../sprints/cms-gridpack-coverage/decisions/D5-cut-decays-exemption-and-b-cut-cards.md", title: "D5 (revised) and its sources"}
  - {id: hc2, resource: "../../sprints/cms-gridpack-coverage/sessions/H-C2-report.md", title: "H-C2: why the configuration-1 exemption diverges on an every-pairing |M|²"}
---
`cut_decays = F` is the default in MadGraph (`banner.py:4306`) and in vibegraph,
and 10 of the 11 CMS decay-chain survey cards use it.

MadGraph keeps one pairing of identical particles across a decay and exempts its
decay products (`check_decay`, configuration 1). vibegraph keeps every pairing and
today cuts every leg that is a core particle in some pairing: all four quarks of
`g g > z u u~, z > u u~`. That is a stricter observable, 5.348 against MadGraph's
7.211 pb (−26%), −6.8% on the full `g g > z j j, z > j j` card. It ships in v0.2
([D2](../../sprints/v02-last-refusals/decisions/D2-cut-decays-keep-the-stricter-cut.md)).

**No exemption on the full every-pairing |M|² is IR-safe.** MadGraph's
configuration-1 legs (H-C2) and the kinematic window rule (I-C1) both leave an exempt
quark uncut where, in another pairing, it is a core quark of a `g → q q̄` splitting,
so |M|² ∝ 1/pT².

**The default to build
([D3](../../sprints/v02-last-refusals/decisions/D3-identical-decays-behind-madgraph-compat.md)):**
`Σ_p w_p Θ_p |M|²` with `w_p = |M_p|²/Σ_q |M_q|²`, where Θ_p exempts pairing p's decay
products and cuts its core legs. MadGraph's single pairing becomes
`--madgraph-compat` site (d) of
[madgraph-compat-sites-unconditional](../feature/madgraph-compat-sites-unconditional.md).

- **IR argument (derived, not measured).** |M|² ≤ N Σ_q |M_q|² gives
  w_p Θ_p |M|² ≤ N Θ_p |M_p|², so σ is finite when each pairing's own |M_p|² is
  integrable where Θ_p is open. That holds for two-body decays of a massive
  resonance: in I-C1's limit w_p ~ ε² cancels q's 1/ε², and Θ_q cuts the leg in
  q's term. It fails for multi-body decays with massless products.
- **The guard there:** run the limit check of #113 over each pairing's own
  diagrams with that pairing's exemptions (H-C3's Found 2, sound for this rule).
- **Oracle:** I-C1's probe (`ic1_window_probe.rs`, in its report) on the
  weighted integrand: w_p Θ_p |M|² ε² → 0.
- **Caveat:** w_p uses diagram subsets that are not gauge invariant off the pole.
  It moves only the interference share (~0.2%), and it is a fixed choice.
