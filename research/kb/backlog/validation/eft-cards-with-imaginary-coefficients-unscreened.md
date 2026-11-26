---
type: Backlog Item
title: "CMS EFT cards with imaginary Wilson coefficients have not been screened for the imaginary-coupling defect"
description: "Every dim6top CPV point and SMEFTsim *Im card may carry the phase defect that put card 6 at 1.67×. A one-subprocess σ pair, with and without the imaginary coefficients, screens a card in minutes."
area: validation
state: open
priority: medium
closes_when: "Every campaign card with a nonzero imaginary Wilson coefficient is listed with its model. On the fixed build, each family has one card read against MG within 2σ, and the list records which cards ran."
blocked_by: [imaginary-couplings-on-contact-and-self-conjugate-vertices-wrong-phase]
opened: 2026-11-26
tags: [eft, cpv, dim6top, smeftsim, campaign, screen]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-26}
---
Cards 6 (tZq, 1.655×) and 7 (ttZ, 1.215× on one seed) are the two found so far,
both dim6top. The screen is cheap: one subprocess's σ, once with the card's
imaginary coefficients and once with them zeroed. MadGraph's σ barely moves on a
CP-even observable, so any vibegraph shift is the defect.

**Census.** `vg-validator`'s gpdb query read the param card MadGraph ran for
each of the 1000 v022-1009 cards. Exactly two set a nonzero imaginary Wilson
coefficient: cards 6 and 7, both dim6top. Both were refused at model load in
v022.

These models set only real parts:
- SMEFTsim general (96, 97, 468–472) and topU3l (832–838);
- TopFCNC_model_ATLAS (948–954) and TopFCNC_UFO (590–595);
- WEff_UFO, QuarticCKM_5, SM_HEL_UFO_noLightYukawa.

**Unscreened neighbours.** These cards have real CP-odd parameters that build
an explicit i·γ⁵ coupling. They are implicated only if their model puts two
interfering structures on one self-conjugate vertex:
- **HC_NLO_X0_UFO card 4** (tth, kAtt and kHtt): 0.992 in v022, with a 5-seed
  mean pull of −2.19.
- **SM_alp_UFO TCP cards 174, 175 and 965–969**: all 1.0065–1.0199 high. Cards
  967–969 are on the v022 fail list
  (`half-percent-residual-after-hard-survey-134-967`).

The sign test on these models is a per-vertex check on the UFO: whether any
self-conjugate vertex carries both a scalar-like and a γ⁵ structure, with
different couplings.
