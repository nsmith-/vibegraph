---
type: Backlog Item
title: The mixed-sign vertex refusal names no UFO vertex, structure, coupling or process
description: "The mixed-sign refusal prints internal coupling ids and the rooted rendering, so a user with an arbitrary UFO cannot tell which vertex, structures or process to look at."
area: validation
state: open
priority: low
closes_when: "The refusal names the UFO vertex (e.g. V_ltltst), its Lorentz structures (e.g. FFS1/FFS5), its couplings and orders, and the subprocess being compiled, and a test asserts those names appear."
blocked_by: []
opened: 2026-10-15
tags: [errors, usability, ufo, refusal]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-15}
sources:
  - {id: ch, resource: "../../sprints/amplitude-signs/sessions/C-H-report.md", title: "C-H report, Found"}
---
Today: "the structures of `CouplingId(5)*(1*ScalarProduct(IdentityAmp(Leg(1),Leg(0)),Leg(2))) + …`
carry mixed rooting-convention signs". `reject_cyclic_structure` already names
its structure. Needs the UFO vertex id carried on `VertexInfo` (about 12
constructions in `diagram_eval.rs` and `run.rs`) and `compile` adding the
subprocess.
