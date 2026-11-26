---
type: Backlog Item
title: "A complex coupling on a two-fermion contact vertex or a self-conjugate dipole binds with the wrong phase or sign"
description: "conjugate_couplings returns early for vertices other than 2 fermions + 1 vector, and for self-conjugate ones. Imaginary Wilson coefficients (dim6top CPV) then mismatch per diagram."
area: validation
state: open
priority: high
closes_when: "Every two-fermion vertex binds its Hermitian partner's coupling on the conjugated structure, times the conjugation sign; a self-conjugate vertex is its own partner, and bound_coupling returns the sign. A committed per-diagram x per-helicity row with nonzero imaginary coefficients on both vertex classes fails on main and passes with the fix, on a subprocess and its charge conjugate. A test pins the sign. Real-coefficient rows are unchanged. If artifacts embed bound couplings, ENUMERATION_VERSION is bumped."
blocked_by: []
opened: 2026-11-26
tags: [ufo, couplings, eft, cpv, dim6top, hermiticity, amplitude, silent-wrong-answer]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-26}
---
Found by sprint L's L-D1 probe on card 6 (tZq_EFT_CPV_point_1, dim6top_LO_UFO).
Its report is on branch `sprintl-dim6top-sigma` (be45128c). The probe compared
75 diagrams of `g u > t b~ d mu+ mu-` with the decay at 10 points against MG
3.7.1 standalone:
- **ctXI = 0:** every diagram matches (ratio −i to 6.7e-14).
- **Card 6's imaginary coefficients:** 23 of 75 diagrams disagree, in two
  classes:
  - **(A) contacts.** The t-b-W-γ and t-b-W-Z contact vertices take their own
    coupling where MG takes the Hermitian partner's (GC_796 = conj(GC_795)).
    The phase is off by −c/c* = e^{−i·46.4°}.
  - **(B) self-conjugate dipoles.** On t-t-g/γ/Z, the σ·q·γ⁵ term (FFV9,
    carrying ctGI/ctWI/ctZI) has the wrong sign relative to σ·q (FFV11).

`UFOModel::conjugate_couplings` (`ufo/mod.rs`) returns early for a vertex that
is not exactly two fermions plus one vector (A), and when `own == anti` (B).

The candidate rule, checked arithmetically on the dumps, closes the fitted
subprocess to 8e-14 and the charge conjugate `g d > t~ b u mu+ mu-` to 1.9e-14.
Fixing only A or only B leaves 4–15%.

On card 6, summed |M|² moves by 0.989 at √ŝ = 400 GeV and 1.089 at 800 GeV, so
this is a percent-level effect there. It does not explain the card's 1.66×
(`tzq-eft-cpv-sigma-above-madgraph`). Any model with imaginary couplings on
these vertex classes is exposed: every dim6top CPV point, and probably
SMEFTsim's CP-odd operators. No enforced row has imaginary coefficients on
these classes, which is why no gate caught it.

The sign convention is the project's (match MadGraph). An independent analytic
CP-odd check would confirm it without MadGraph; that would be a separate item if
wanted.
