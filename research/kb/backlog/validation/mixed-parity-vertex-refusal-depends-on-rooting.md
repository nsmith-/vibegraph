---
type: Backlog Item
title: A vertex with mixed reversed-bilinear parities is refused at one rooting and compiles at another
description: "A Yukawa with Identity beside a derivative P·Gamma term is refused for lt~ lt > st but compiles for st > lt lt~, where nothing checks the parity its gamma term is evaluated with."
area: validation
state: open
priority: medium
closes_when: "Either the vertex is refused at every rooting, or the compiled case is gated against MadGraph AMP() (or against its split form) and shown correct, with the refusal's rooting dependence explained in VertexInfo::reversed_sign's doc."
blocked_by: []
opened: 2026-10-16
tags: [amplitudes, sign-convention, rooting, toy-ufo]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-16}
sources:
  - {id: pr30, resource: "https://github.com/nsmith-/vibegraph/pull/30", title: "PR #30's merge of main, which re-measured C-H's refusal test under the root-free convention"}
  - {id: ch, resource: "../../sprints/amplitude-signs/sessions/C-H-report.md", title: "C-H report"}
---
`toy_models.rs::mixed_sign_vertex_is_refused_at_compile` builds the vertex on an
edited toy copy. With the fermion pair sinking into the scalar, the gamma term
is a `GammaVout` sink and takes the reversed parity while `Identity` does not,
so the vertex is refused. With the scalar on leg 0 neither term is a
fermion→vector sink and the vertex compiles. The root-free convention reads each
vertex's parity "as the amplitude root", but for this vertex the reading depends
on the root leg. No model in reach has such a vertex.
