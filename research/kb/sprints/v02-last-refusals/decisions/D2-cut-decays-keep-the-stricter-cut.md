---
type: Design Decision
title: "Sprint I: keep the stricter cut_decays rule for v0.2; the window rule is IR-divergent"
description: "D2: the kinematic window rule diverges on the every-pairing |M|² through an exempt quark's beam-collinear limit, so v0.2 keeps cutting every leg that is not a decay product in every pairing, the registered −26%/−6.8% deviation."
decided: 2026-11-08
decided_by: human:nsmith-
status: stable
tags: [cuts, decay-chains, identical-particles, ir-safety, sprint-i]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-08}
verified: [{by: "human:nsmith-", at: 2026-11-08}]
sources:
  - {id: ic1, resource: "../sessions/I-C1-report.md", title: "I-C1 report"}
  - {id: d5, resource: "../../cms-gridpack-coverage/decisions/D5-cut-decays-exemption-and-b-cut-cards.md", title: "Sprint H D5"}
---
**The premise that failed.** Sprint H's D5 named the window rule as the later
fix: exempt the pairing whose pair mass lies inside the forced window, IR-safe
"because a pair holding a soft quark has m → 0". The user brought it into v0.2.
I-C1 found the argument covers the soft limit only.

**The evidence** ([I-C1](../sessions/I-C1-report.md) §2). On
`g g > z u u~, z > u u~`, keep pair {0,1} exactly on the pole and drive leg 0
collinear with a beam (pT = ε), with legs 2 and 3 passing every cut. Then
|M|²·ε² is constant to 1e-4 between ε = 1e-2 and 1e-3, in three
configurations, including one where every other pairing is outside its
window. The distinct-flavour control (`z > d d~`) has no pole. The 1/pT² comes
from the other pairings, in which leg 0 is a core quark of a `g → u ū`
splitting. With the measure going as pT dpT, σ diverges, and the channels'
floor would cap it silently.

**The options.**
- (a) Keep today's rule (every leg that is not a decay product in every pairing
  is cut). It is IR-safe and registered.
- (b) Refuse such cards.
- (c) Per-pairing |M|² now.

**Decided (a)** (the user, 2026-11-08). The deviation stays registered (D5).
The item `cut-decays-identical-quarks-cut-a-stricter-observable` is rewritten
toward the per-pairing route and released from sprint I. H-C3's Found 2
("rebuild the guard over the exempting pairing's diagrams") is unsound for any
rule that exempts on the full |M|², because the pole comes from the other
pairings. Under the pairing-weighted cut of [D3](D3-identical-decays-behind-madgraph-compat.md),
where each pairing's exemption multiplies that pairing's share of |M|², it is
the right guard.
