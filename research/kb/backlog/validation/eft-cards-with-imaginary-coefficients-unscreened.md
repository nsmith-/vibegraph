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
