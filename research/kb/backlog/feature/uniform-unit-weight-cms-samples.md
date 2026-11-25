---
type: Backlog Item
title: "CMS samples need one unit event weight across channels and multiplicities, with nothing above it"
description: "Downstream CMS processing costs ~27 core-s per LHE event against ~0.007 s for vibegraph, so any weight spread wastes paid-for statistics; flat weights are worth a much lower unweighting efficiency."
area: feature
state: open
priority: high
closes_when: "generate can write a sample whose every event carries one weight across all parts and flavour channels, with no weight above it, selected per gridpack and recorded in gridpack.json; its σ and per-channel shares agree with the buffered sample within a statistical gate; and the efficiency cost per extra factor of scan points is measured."
blocked_by: []
opened: 2026-11-24
tags: [generate, unweighting, lhef, cms, gridpack, v0.3]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-24}
sources:
  - {id: unweighting, resource: "../../events/unweighting.md", title: "Unweighting: channel draw ∝ w_max, overweights kept"}
  - {id: strategy, resource: "../../events/lhef-weight-strategy.md", title: "LHEF weight strategy: Buffer or StochasticRounding"}
---
The user decided on 2026-11-24 that CMS samples should have weights as
uniform as possible, targeting v0.3. On simshop's v0.2.2 Drell-Yan MLM job the
full CMS chain cost about 27 core-seconds per LHE event (44 per event kept
after matching), against about 0.007 s for vibegraph, so unweighting can afford
to discard far more to buy flat weights.

That job (`p p > e+ e-` + `p p > e+ e- j`, buffer, 1000 events) has a Kish
`N_eff` of 97.6% of its kept events: one unit weight per flavour subprocess
(98.8% from those units alone) and overweights (98.0% alone). Richer processes
spread more: overweights have carried 4.6–7.4% of σ on mixed-multiplicity MLM
grids.

What has to be designed:

1. **One unit across parts and channels.** Unweight against a common scale
   instead of a per-part factor. Today buffer normalises each part
   separately, and `StochasticRounding` is refused on a sum over
   multiplicities.
2. **No weight above the unit.** Unit-weight copies (stochastic rounding: an
   event of weight `r` is written `floor(r)+Bernoulli(frac(r))` times, and the
   copies share parton-level kinematics), or a larger scan or safety factor
   that leaves too few overweights to matter. Measure both costs.
3. **A per-gridpack selector** recorded in `gridpack.json`, so a production
   tool can default to it.

Settle first whether the per-flavour unit weights simshop observed are intended
compensation or a bias; that is being investigated for v0.2.3 separately.

**Where the per-flavour units came from** (simshop's rerun of the same job,
2026-11-24). The card used `smeftsim_topu3l_mwscheme_ufo`, whose light quarks
are massive. Incoming masses split a multiplicity into mass parts
(`multiplicity.rs`, `split_by_external_masses`), so `p p > e+ e-` +
`p p > e+ e- j` became 12 parts, one per incoming-flavour signature. Buffer
normalises each part to its own integrated σ, so each flavour got its own
unit. Nothing is biased, but the scales are noisy: part 9 held 7 of 1000
events and was scaled up by 1.83, since its sample estimate was 45% below
its integration. On the `sm` model, where the light quarks are massless, the
same card has two parts and two units (L-X3). A common unit has to replace
per-part normalisation, not only per-multiplicity normalisation.
