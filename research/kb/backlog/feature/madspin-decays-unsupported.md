---
type: Backlog Item
title: MadSpin decays are unsupported
description: "11 of 65 LO CMS cards decay resonances with MadSpin; the agreed form is an opt-in decay stage after the map, with the in-process decay chain as the default."
area: feature
state: open
priority: medium
closes_when: "A decay request (a madspin_card's decays) is read at integrate and recorded in the artifact; by default it runs as the equivalent in-process decay chain; opted in, production is integrated undecayed and generate applies a factorised decay stage (a trailing stage after the map, on its own substreams, outside Σαg, accepted against a per-event maximum as MadSpin does); σ × BR and decayed distributions match MadSpin on a banked tt̄ leptonic CMS card, and the two modes agree to O(Γ/M) in a running informational comparison."
blocked_by: []
opened: 2026-10-20
tags: [madspin, decays, phase-space, plugin, cms]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-22}
sources:
  - {id: survey, resource: "../../validation/cms-gridpack-survey-2026-10.md", title: "CMS gridpack survey, 88 production cards (e3f543f)"}
  - {id: design, resource: "../../phase-space/sampler-and-map-seams.md", title: "The importance sampler and the phase-space map, designed together (D5)"}
---
In the [CMS gridpack survey](../../validation/cms-gridpack-survey-2026-10.md)
11 of 65 LO cards ship a `madspin_card.dat`; vibegraph reads none.

The seam is settled in [the joint design](../../phase-space/sampler-and-map-seams.md)
(D5, agreed with the user 2026-10-22):

- **Default:** the decays are rewritten into the equivalent decay-chain process
  and trained in-process, as [decay chains](../../process/decay-chains.md) are
  today.
- **Opt-in, factorised:** σ and the grids are the undecayed production's, times
  the recorded BR; a decay stage after the map draws each kept event's decay
  from a trailing substream and accepts it on MadSpin's weight against a
  per-event maximum from trial decays.

What to build, in order:

1. Generalise `ChannelIntegrand::scale_draw_ndim` into a list of trailing
   stages, each with its own substream base; byte-identical on the default
   path.
2. Read the decay request at integrate (an integrate input under PR #60's
   gridpack `generate`) and record it, with its mode, in the artifact; a build
   that does not implement the mode refuses it by name.
3. The chain mode, then the factorised stage. Read the decay weight, the
   trial count, the Breit–Wigner cutoff and the `spinmode` values from
   MadSpin's source (`research/refs/mg5amcnlo`, `MadSpin/`), not from memory.

Sequencing: after PR #60's H1 and H2. Related:
[decay-angles-drawn-flat-in-decay-chains](../performance/decay-angles-drawn-flat-in-decay-chains.md).
