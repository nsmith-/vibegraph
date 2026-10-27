---
type: Backlog Item
title: "Charged scalar and tensor vertices still carry their own coupling"
description: "bound_coupling exchanges couplings only for charged vector currents; a charged Higgs/Goldstone Yukawa or a W–t–b dipole with a complex coefficient would evaluate the CP conjugate."
area: validation
state: open
priority: medium
closes_when: "Each Lorentz structure of a charged fermion–fermion–boson vertex maps to its Hermitian image and the vertex carries the conjugate interaction's coupling on that image, pinned by a MadGraph row at a complex coefficient."
blocked_by: []
opened: 2026-10-26
tags: [ckm, coupling, convention, smeft, crossing]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-26}
sources:
  - {id: v3, resource: "../../sprints/ckm-no-b-mass/sessions/V3-report.md", title: "ckm-no-b-mass V3 report, Found 2"}
---
Diagram enumeration binds a vertex by its particles read all-incoming, while the
UFO lists them all-outgoing. For a charged vector current
(`γ^μ` with `P_L`, `P_R`, `γ⁵` or the identity), `UFOModel::bound_coupling`
(`vibegraph-lib/src/ufo/mod.rs`) now takes the Hermitian-conjugate interaction's
coupling, which is what MadGraph evaluates.

Every other charged vertex keeps its own coupling: chiral scalars (`FFS`, a
charged Higgs or Goldstone) and tensor or derivative structures (SMEFTsim's
W–t–b dipoles). The same all-incoming reading conjugates their Lagrangian
coefficients, and a name-keyed exchange would be wrong because the conjugate's
same-name term is the opposite chirality (the SM `[c~, b, G+]` / `[b~, c, G-]`
pair lists `FFS1, FFS3` crosswise). With a complex coefficient (a CP-odd Wilson
coefficient, a complex charged-Higgs Yukawa) `|M|²` would be the CP conjugate.

The SM's charged scalars carry real couplings at every banked restriction, so
nothing banked moves today.
