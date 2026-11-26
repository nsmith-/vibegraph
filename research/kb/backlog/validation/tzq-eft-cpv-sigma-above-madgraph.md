---
type: Backlog Item
title: "Card 6 (tZq EFT CPV, dim6top_LO_UFO) reads 1.655 ± 0.074 × MadGraph, and the matrix element does not explain it"
description: "Five seeds, every one ≥ 1.44×; provenance, census, couplings and |M|² at real coefficients all match MG. Same run card on both sides: only ptj 0.1 and mmll 30 cut a massless spectator b~."
area: validation
state: open
priority: high
closes_when: "With the imaginary-coupling fix, card 6 reads within 2σ of 0.04754 pb over 5 seeds at campaign budget (lhaid 325300), with the spread read, and card 7 is read on the same footing."
blocked_by: [imaginary-couplings-on-contact-and-self-conjugate-vertices-wrong-phase]
opened: 2026-11-26
tags: [eft, dim6top, tzq, sigma, madgraph-parity, cuts, cms, release-gate]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-26}
---
Found by `vg-validator`'s pre-tag stage B on dev build 54cecfac.
- **vibegraph, 5 seeds:** 0.07589, 0.07616, 0.08884, 0.06852 and 0.08400 pb.
  Each seed's error is 6–15%.
- **MadGraph:** 2.9.18's base-point survey, 0.04754 ± 0.000055 pb.

The ratio is about 1.45× in every lepton group. The ττ group converges poorly
(0.0136 ± 0.0074 pb on seed 1, against about 0.008 for the others).

Sprint L's L-D1 probe (branch `sprintl-dim6top-sigma`) excluded these as causes:
- **Model provenance:** the amplitude-bearing files are identical to the
  public dim6top_LO_UFO.
- **The restriction:** it keeps exactly the 8 ctX operators.
- **The diagram census:** zero mismatches. `/ h` and `$$ w+ w-` reach the
  diagram set.
- **All 66 couplings:** they match to 1e-12.
- **|M|² with ctXI = 0:** it matches MG 3.7.1 exactly.
- **The decay normalisation:** 0.1158, against an expected 0.116.

The imaginary-coefficient defect
(`imaginary-couplings-on-contact-and-self-conjugate-vertices-wrong-phase`) moves
|M|² by only a few percent.

vibegraph ran the gridpack's own run card byte for byte (sha256 404b5ce0…).
Its only cuts are:
- `ptj = 0.1`;
- `mmll = 30`, with `cut_decays = False`.

`ptb`, `etab` and every ΔR are 0, and `maxjetflavor = 5`. With MB = 0, the
spectator b~ is regulated at most by ptj = 0.1 GeV, so the g → b b̄ collinear
region is nearly unregulated on both sides. The next oracle is how each side
reads the same card in that region: whether b counts as a jet for ptj, and
which lepton pairs mmll cuts.

Card 7 (ttZ_3l_4l_EFT_CPV_point_0_13p6, the same bundled dim6top UFO) reads high
too. On one seed it gives vg/MG 1.215 ± 0.034 (pull +6.4), with a 2.8% error.
It ran its gridpack's own run card. The cuts follow card 6's pattern, with
mmll = 10. Its imaginary coefficients are large (|ctXI| 0.67–0.98), so the
class-B dipole defect is fully in play there.

The two cards discriminate between the hypotheses in two ways:
- **Per event:** check the core m(l+l−) against mmll, with the W-decay leptons
  exempt.
- **Single seed:** compare a reduced σ at ctXI = 0 against card 7's ctXI.

**Cause found.** L-D1's follow-up compared vibegraph against MG 3.7.1 on one
subprocess, `g u > e+ ve b b~ d mu+ mu-`, at card 6's own cuts:
- with card 6's coefficients, vg/MG is 1.668;
- with ctXI = 0 on both sides, it is 1.011.

So the imaginary-coupling defect accounts for the whole gap. The cuts match:
- mmll applies to the core OSSF pair only, on both sides; the minimum core
  m(ll) is 32.2 and 30.07 GeV.
- ptj = 0.1 applies to the b~ at maxjetflavor 5 on both sides.

That run used lhaid 247000 on both sides.

**Full-card control.** `vg-validator` ran card 6 with ctXI = 0 on 54cecfac,
5 seeds at campaign budget, with Γt kept at 1.3129:

| seed | 1 | 2 | 3 | 4 | 5 |
|---|---|---|---|---|---|
| vg/MG | 0.989 | 0.902 | 0.991 | 0.982 | 0.976 |

- The unweighted mean is 0.968 ± 0.017, with χ²/dof 3.39.
- The reference is MG at the CPV point. L-D1's pair puts MG's own ctXI → 0
  shift at about −1.1%, so against a matching reference the control reads
  about 0.98.
- Seed 2 is confidently low: 0.902 with a quoted 2.0% error, the smallest of
  the five. That is a missed-region signature on a collinear-dominated
  integrand. If the fix build's 5 seeds show it again, it becomes its own
  sampling item.
