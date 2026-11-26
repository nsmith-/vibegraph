---
type: Backlog Item
title: "Card 6 (tZq EFT CPV, dim6top_LO_UFO) reads 1.655 ± 0.074 × MadGraph, and the matrix element does not explain it"
description: "Five seeds, every one ≥ 1.44×; provenance, census, couplings and |M|² at real coefficients all match MG. Same run card on both sides: only ptj 0.1 and mmll 30 cut a massless spectator b~."
area: validation
state: open
priority: high
closes_when: "The 1.66× is localised and explained with a recorded measurement: the cut application (ptj on b under maxjetflavor 5, mmll on core pairs only under cut_decays False), the collinear b~ region, or the reference (MG 2.9.18 against 3.7.1 on the same cards). It is fixed, or shown to be a reference effect. Card 6 then reads within 2σ over 5+ seeds, with the spread read, and card 7 (ttZ EFT) is read on the same footing."
blocked_by: []
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
