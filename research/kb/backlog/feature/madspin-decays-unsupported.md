---
type: Backlog Item
title: MadSpin decays are unsupported
description: "11 of 65 LO CMS cards decay resonances with MadSpin; in scope only as a phase-space plugin, whose seam does not exist yet and needs a design session."
area: feature
state: blocked
priority: medium
closes_when: "MadSpin-style decays (spin-correlated, on-shell-window decays of production events) are implemented as a phase-space plugin behind a seam agreed with the user, and decayed distributions and σ × BR match MadSpin on a banked CMS card such as tt̄ with leptonic decays."
blocked_by: [sampler-and-phase-space-seams-designed-apart]
opened: 2026-10-20
tags: [madspin, decays, phase-space, plugin, design, cms]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-20}
sources:
  - {id: survey, resource: "../../validation/cms-gridpack-survey-2026-10.md", title: "CMS gridpack survey, 88 production cards (e3f543f)"}
---
In the [CMS gridpack survey](../../validation/cms-gridpack-survey-2026-10.md)
11 of 65 LO cards ship a `madspin_card.dat`. vibegraph reads none; the
nearest it has is a full decay chain
([decay chains](../../process/decay-chains.md)), which is a different
calculation (the full matrix element in a Breit–Wigner window, at a cost
recorded in [decay chains without MadSpin](../../performance/decay-chains-without-madspin.md)).

User decision (2026-10-20): MadSpin is in scope **only as a phase-space
plugin**, not as a separate post-processing pass. No such seam exists. The
[importance-sampler seam](../../phase-space/importance-sampler-seam.md) is a
density on the unit hypercube *in front of* a fixed phase-space map, while a
MadSpin decay extends the map itself: it adds decay variables to an
undecayed production point and weights by the decayed matrix element.
State is needs-user because the seam's shape has to be designed with the
user first:

- where the decay step sits relative to the map, the sampler and the
  channel combiner;
- how the decayed matrix element, the undecayed one and the spin-correlation
  accept–reject (MadSpin's maximum-weight bound) divide between plugin and core;
- what the artifact records so `generate` replays it under the `-j` contract.

Related: [decay-angles-drawn-flat-in-decay-chains](../performance/decay-angles-drawn-flat-in-decay-chains.md).
