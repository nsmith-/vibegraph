---
type: Backlog Item
title: "With cut_decays = F, identical quarks across a decay are all cut, a stricter observable than the card asks for"
description: "vibegraph cuts every quark that is a core particle in some pairing, so g g > z u u~, z > u u~ reads −26% below MadGraph; exempt the pair that reconstructs the resonance instead."
area: validation
state: open
priority: high
closes_when: "Under cut_decays = F the pairing whose pair mass lies inside the forced window (closest to the pole when several do) is exempted, a test pins that a soft quark is never exempted, and σ on g g > z u u~, z > u u~ and on g g > z j j, z > j j matches local MadGraph by the multi-seed criterion within the interference scale."
blocked_by: []
opened: 2026-11-06
tags: [cuts, decay-chains, identical-particles, madgraph-parity, cms]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-06}
sources:
  - {id: hk1, resource: "../../sprints/cms-gridpack-coverage/decisions/D5-cut-decays-exemption-and-b-cut-cards.md", title: "D5 (revised) and its sources"}
  - {id: hc2, resource: "../../sprints/cms-gridpack-coverage/sessions/H-C2-report.md", title: "H-C2: why the configuration-1 exemption diverges on an every-pairing |M|²"}
---
`cut_decays = F` is the default in MadGraph (`banner.py:4306`) and in vibegraph,
and 10 of the 11 CMS decay-chain survey cards use it.

MadGraph keeps one pairing of identical particles across a decay, so for it "the
decay products" are well defined. It exempts them (`check_decay`, configuration 1).
vibegraph keeps every pairing, so it falls back to cutting every leg that is a
core particle in any pairing: all four quarks of `g g > z u u~, z > u u~`. That is
a stricter observable. It reads 5.348 against MadGraph's 7.211 pb (−26%), and −6.8%
on the full `g g > z j j, z > j j` card. The interference MadGraph drops is the
small part, about 0.2% (the identical-particles registered deviation).

**The fix is kinematic and needs no amplitudes.** Exempt the pairing whose pair
invariant mass lies inside the forced window, |m − M| < bwcutoff·Γ, choosing the one
closest to the pole if several do. It is IR-safe: a pair containing a soft quark has
m → 0 and is never in the window. It should match MadGraph except where two
pairings are in the window at once, which is the interference region. Exempting
MadGraph's configuration-1 legs on vibegraph's every-pairing |M|² instead makes σ
diverge logarithmically (H-C2).

Read `setcuts.f`'s other `from_decay` readers before implementing. The exemption
changes per point, so it cannot use the fixed per-subprocess `uncut_products` mask.
The IR-unsafe-exemption guard (`refuse_ir_unsafe_exemption`, `cuts.rs`) must also
be checked against the exempting pairing's own diagrams: as written it ORs over
every kept diagram and would refuse this rule (H-C3, Found 2). Exact
single-pairing reproduction is compat site (d), a separate item.
