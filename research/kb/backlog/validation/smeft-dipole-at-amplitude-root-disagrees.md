---
type: Backlog Item
title: "SMEFTsim dipole operators disagree with MadGraph when the dipole vertex is the amplitude root"
description: "ctWRe, cbGRe, cbWRe, cbBRe miss MadGraph by 2.5e-5 to 8.7e-3 per flow on b b~ > t t~ and b b~ > e+ e-; the mirrored e+ e- > b b~ and the banked dipole gate agree. Silent."
area: validation
state: open
priority: high
closes_when: "b b~ > e+ e- NP<=1 (cbBRe only), t t~ > e+ e- NP<=1 (ctBRe only) and b b~ > t t~ NP<=1 with the full dipole set agree with MadGraph standalone per flow × helicity as enforced rows that fail before the fix; every banked row unchanged."
blocked_by: []
opened: 2026-11-11
tags: [amplitudes, smeft, dipoles, lorentz, silent-wrong-answer, release]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-11}
sources:
  - {id: sprint, resource: "../../sprints/v021-fixes/sprint.md", title: "Sprint J plan, PR 6"}
---
Found by J-E1 while validating the SMEFTsim four-quark panic fix, with
per-coefficient restrict cards from `restrict_massless`. Failing: ctWRe 2.1e-3,
cbGRe 2.0e-3, cbWRe 8.7e-3, cbBRe 2.5e-5 on `b b~ > t t~ NP<=1`. ctGRe and
ctBRe agree there. Minimal: `b b~ > e+ e-` with cbBRe (2.9e-4) and
`t t~ > e+ e-` with ctBRe (4.6e-3). The mirror `e+ e- > b b~` and the banked
`ee_to_ttx_dipole` agree, so every banked dipole gate has the dipole rooted
as a current. The working hypothesis, unverified, is a momentum-sign
convention for `P` when the dipole vertex is the amplitude root. Session J-D1
diagnoses and fixes it.
