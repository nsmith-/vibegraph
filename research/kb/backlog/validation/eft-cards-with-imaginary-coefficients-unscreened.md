---
type: Backlog Item
title: "CMS EFT cards with imaginary Wilson coefficients have not been screened for the imaginary-coupling defect"
description: "Every dim6top CPV point and SMEFTsim *Im card may carry the phase defect that put card 6 at 1.67×. A one-subprocess σ pair, with and without the imaginary coefficients, screens a card in minutes."
area: validation
state: open
priority: medium
closes_when: "Every campaign card with a nonzero imaginary Wilson coefficient is listed with its model. On the fixed build, each family has one card read against MG within 2σ, and the list records which cards ran."
blocked_by: []
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

**Neighbours, scanned on the UFO text.** `vg-validator`'s scan looked for
self-conjugate vertices with two or more Lorentz structures on distinct
couplings, at least one chiral. Its positive control on dim6top found the known
class-B vertices.
- **SM_alp_UFO: clear.** Every ALP vertex has a single structure, so a
  conjugation sign cancels in |M|². The TCP cards' 0.7–2% offset (174, 175,
  965–969) stays with `half-percent-residual-after-hard-survey-134-967`.
- **HC_NLO_X0_UFO: implicated by shape.** X0 t t̄ (V_3010) carries FFS1 with
  coupling i·kHtt and FFS5 (γ⁵) with a real kAtt, with no factor i. That is the
  same relative phase as dim6top's V_110 with an imaginary ctpI. The same holds
  for the X0 couplings to τ, b, μ and c (V_3008, V_3009, V_3011 and V_3012).
  Either main is wrong on HC with any nonzero kAff, or a fix that closes
  dim6top breaks HC. The per-diagram x per-helicity check against MadGraph is
  required of the fix. It must run at a point with cosa ≠ ±1 (e.g. cosa = 1/√2,
  kAtt = kHtt = 1).

**Card 4's base point cannot see the defect.** It sets cosa = 1, so
sina = 0 and the γ⁵ coupling GC_3010a vanishes, and its 0.992 reads only
CP-even physics. Its CP mixing lives in its 344-line reweight card: 19 points
in cosa, kAtt, kHtt and others. Any reweighting of such a sample to cosa ≠ ±1,
including vibegraph's own reweight path, inherits the defect until the fix
lands. HC_NLO_X0 is the standard ttH CP model, so that reweight case is the
exposure that matters most downstream.
