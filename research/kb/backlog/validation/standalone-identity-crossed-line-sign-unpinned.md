---
type: Backlog Item
title: A bare Identity bilinear rooted on a crossed fermion line takes no −1
description: "Rooted as a fermion current on a crossed line, Identity(2,1) takes no −1 while ProjM+ProjP, the same structure, takes one per term; SMEFTsim's top and bottom Yukawas are bare Identity."
area: validation
state: open
priority: high
closes_when: "A toy or SMEFTsim row that roots a bare Identity Yukawa as a fermion current on a crossed line (e.g. e+ e- > t t~ h under SMEFTsim) is gated per diagram against MadGraph, and the Identity arm signs like ProjM+ProjP or the difference is explained."
blocked_by: []
opened: 2026-10-09
tags: [amplitude, sign-convention, fermion-flow, smeftsim, falsifier]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: code, resource: "../../../../vibegraph-lib/src/helas/eval/root_lorentz.rs", title: "root_lorentz.rs: the Identity arm (~:484-492) vs the ProjM/ProjP/Gamma5 arms (~:426-465, :497-509)"}
  - {id: smeftsim, resource: "../../../../validation/ufo/SMEFTsim_topU3l_MwScheme_UFO/lorentz.py", title: "SMEFTsim FFS2 = Identity(2,1), used by the b b~ H and t t~ H vertices"}
---
In `build_child` (`vibegraph-lib/src/helas/eval/root_lorentz.rs`), the ProjM,
ProjP and Gamma5 arms flip the term's sign when `standalone_projector_crossed`
holds, i.e. a standalone bilinear rooted at a fermion output whose wrapped leg
sits on a crossed line. The `Identity` arm (~:484-492) recurses with no such
check. `Identity = ProjM + ProjP`, and the Gamma5 arm's own reason for the −1
(`C Γᵀ C⁻¹ = Γ`) holds for the identity too. So the SM spelling (`FFS4`,
ProjM+ProjP) and a bare-Identity spelling of one Yukawa give opposite signs
when H is emitted off a crossed line. The amplitude-root arm (~:786-790)
does apply `pair_crossed` to Identity, which makes the gap specific to the
current-rooted case.

No gated row reaches it. `bbx_to_h_identity` and the toy Identity/Gamma5 row
(manifest ~:991) use Identity only at amplitude or scalar-sink roots. The
ProjM/ProjP flip was pinned by `e+ e- > ta+ ta- h` against MadGraph's `AMP()`.
SMEFTsim writes its SM top and bottom Yukawas as `FFS2 = Identity(2,1)`, next to
`FFS1 = Gamma5(2,1)`, which does flip. So `e+ e- > t t~ h` under SMEFTsim would
give a wrong relative sign between diagrams, and between its scalar and
pseudoscalar parts, if the gap is real.

Falsifier first, as AGENTS.md asks: a per-diagram × per-helicity dump against
MadGraph standalone on such a row, before any change to the arm. Found by
Phase 3 verifier V8 and confirmed against the code on 2026-10-09.
