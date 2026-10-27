---
type: Backlog Item
title: "A self-conjugate vertex with a complex chiral coefficient is unpinned"
description: "The all-incoming reading binds a self-conjugate vertex with its fermion slots swapped; whether c_L ↔ c_R follow as MadGraph does is untested, since no banked row has a complex coefficient there."
area: validation
state: open
priority: low
closes_when: "One MadGraph row with an imaginary dipole or CP-odd Yukawa on a self-conjugate vertex (h t t~ or t t~ γ) gates per diagram."
blocked_by: []
opened: 2026-10-26
tags: [coupling, convention, smeft, chirality]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-26}
sources:
  - {id: v3, resource: "../../sprints/ckm-no-b-mass/sessions/V3-report.md", title: "ckm-no-b-mass V3 report, Found 3"}
---
For a self-conjugate vertex the all-incoming reading binds the same interaction
with its fermion slots swapped, and the chiral correction puts `P_χ̄` on the
swapped slots. Whether the coefficients follow (`c_L ↔ c_R` against `−conj`)
is a convention claim no banked row can see: every banked self-conjugate
fermion vertex carries a real or `i·real` coefficient.

Found by the [ckm-no-b-mass V3 session](../../sprints/ckm-no-b-mass/sessions/V3-report.md)
while scoping the CKM conjugation fix. The oracle is a MadGraph row with an
imaginary dipole or a CP-odd Yukawa, e.g. SMEFTsim at a restriction that keeps
the coefficient complex.
