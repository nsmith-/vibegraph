---
type: Backlog Item
title: "A two-body decay with a fixed-helicity daughter is not declared constant"
description: "t > w+{0} b is isotropic once the mother's spin is summed, but the constant-integrand condition excludes every polarized leg."
area: feature
state: open
priority: low
closes_when: "The constant-integrand condition admits final-state polarizations of a two-body decay, pinned against a polarized-width oracle, or the record says why it must not."
blocked_by: []
opened: 2026-10-17
tags: [decay-width, polarization, constant-integrand]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-17}
sources:
  - {id: p1, resource: "../../sprints/vegas-constant-integrand/sessions/P1-report.md", title: "vegas-constant-integrand P1 report, Found 4"}
---
`FixedBeamIntegrand::has_constant_integrand` refuses any polarized leg. A
decay mother's polarization is already refused upstream
(`DecayedPolarization`), so only daughters can be polarized. With the
mother's spin summed, a two-body decay into a fixed-helicity daughter is
still isotropic
([P1](../../sprints/vegas-constant-integrand/sessions/P1-report.md), Found 4).
Relaxing the condition needs a polarized-width oracle, which none of the
current gates provides. Decays with three or more bodies are never constant
and must not be admitted by analogy.
